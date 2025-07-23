use std::{
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
    time::Duration,
};

use rug::{Assign, ops::MulFrom};
use sfml::{
    cpp::FBox,
    graphics::{
        Color, FloatRect, Rect, RenderTarget, RenderWindow, Sprite, Texture, Transformable, View,
    },
    system::Vector2f,
    window::{ContextSettings, Event, Style, mouse::Button},
};

use crate::{
    fractal_complex,
    fractal_engine::{FractalContext, FractalNotif},
    sfml_engine_worker::SfmlEngineWorkerInternal,
};

pub struct SfmlEngineInternal<'a> {
    notif_rx: &'a Receiver<FractalNotif>,
    ctx_rwl: Arc<RwLock<FractalContext>>,
    workers: Vec<SfmlEngineWorkerExternal>,
    win: FBox<RenderWindow>,
    texture: FBox<Texture>,
}

struct SfmlEngineWorkerExternal {
    tx: Sender<WorkerNotif>,
    rx: Receiver<WorkerResult>,
}

pub enum WorkerNotif {
    SetRenderRect(Rect<u32>),
    Reload,
    Shutdown,
}

pub struct WorkerResult {
    pub pixels: Vec<u8>,
    pub rrect: Rect<u32>,
    pub reload_dur: Duration,
}

impl<'a> SfmlEngineInternal<'a> {
    pub fn run(ctx_rwl: Arc<RwLock<FractalContext>>, rx: Receiver<FractalNotif>) -> ! {
        loop {
            match rx.recv().unwrap() {
                FractalNotif::Commence => (), // Time to start...
                FractalNotif::Shutdown => {
                    println!("BRO SHUT UP I'M ALREADY ASLEEP");
                    continue;
                }
                _ => {
                    println!("IDGAF I'M NOT UP YET !");
                    continue;
                }
            };

            let ctx = ctx_rwl.read().unwrap();

            let mut win = RenderWindow::new(
                (ctx.res.x, ctx.res.y),
                "Mandelbread SFML Engine",
                Style::DEFAULT,
                &ContextSettings::default(),
            )
            .unwrap();

            win.set_framerate_limit(60);
            win.set_vertical_sync_enabled(true);

            let mut workers = vec![];

            for id in 0..ctx.worker_count {
                let (worker_tx, internal_rx) = mpsc::channel();
                let (internal_tx, worker_rx) = mpsc::channel();
                let ctx_rwl_clone = ctx_rwl.clone();

                thread::Builder::new()
                    .name(format!("SFML Worker {}", id))
                    .spawn(|| {
                        SfmlEngineWorkerInternal::build_and_run(
                            internal_rx,
                            internal_tx,
                            ctx_rwl_clone,
                        )
                    })
                    .unwrap();

                workers.push(SfmlEngineWorkerExternal {
                    tx: worker_tx,
                    rx: worker_rx,
                });
            }

            let mut texture = Texture::new().unwrap();
            texture
                .create(ctx.res.x / ctx.lodiv, ctx.res.y / ctx.lodiv)
                .unwrap();
            drop(ctx);

            let internal_engine = SfmlEngineInternal {
                notif_rx: &rx,
                ctx_rwl: Arc::clone(&ctx_rwl),
                win,
                texture,
                workers,
            };

            internal_engine.run_until_stop();
        }
    }

    pub fn run_until_stop(mut self) {
        while self.win.is_open() {
            self.handle_events_internal();
            self.handle_notifs_internal();
            self.render_internal();
        }
    }

    fn handle_events_internal(&mut self) {
        while let Some(event) = self.win.poll_event() {
            match event {
                Event::Closed => self.shutdown_internal(),
                Event::Resized { width, height } => self.resize_internal(width, height),
                Event::MouseButtonReleased { x, y, button } => {
                    if button == Button::Left {
                        self.move_window_from_mouse_pos(x, y);
                    }
                }
                Event::MouseWheelScrolled {
                    wheel: _,
                    delta,
                    x,
                    y,
                } => {
                    if delta > 0.0 {
                        self.zoom_view_scrollwheel(1.0 / 1.1, x, y);
                    } else {
                        self.zoom_view_scrollwheel(1.1, x, y);
                    }
                }
                _ => (),
            }
        }
    }

    fn handle_notifs_internal(&mut self) {
        match self.notif_rx.try_recv() {
            Ok(notif) => match notif {
                FractalNotif::Commence => panic!("bah bro je roule déjà..."),
                FractalNotif::Shutdown => self.shutdown_internal(),
                FractalNotif::Reload => self.reload_internal(),
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("The connexion shouldn't be disconnected"),
        }
    }

    fn render_internal(&mut self) {
        // TODO FIX THIS LATER
        let mut sprite = Sprite::with_texture(&self.texture);
        sprite.set_scale((
            self.win.size().x as f32 / sprite.texture_rect().width as f32,
            self.win.size().y as f32 / sprite.texture_rect().height as f32,
        ));

        self.win.clear(Color::CYAN);
        self.win.draw(&sprite);
        self.win.display();
    }

    fn shutdown_internal(&mut self) {
        self.win.close();
        for worker in &self.workers {
            worker.tx.send(WorkerNotif::Shutdown).unwrap();
        }
    }

    fn resize_internal(&mut self, width: u32, height: u32) {
        let mut ctx = self.ctx_rwl.write().unwrap();

        ctx.res = self.win.size();
        ctx.has_resized = true;

        let mut new_real = ctx.window.real().clone();
        new_real.mul_from(height as f64 / width as f64);
        ctx.window.mut_imag().assign(new_real);

        self.win.set_view(
            &*View::from_rect(FloatRect::new(0.0, 0.0, width as f32, height as f32)).unwrap(),
        );
    }

    fn move_window_from_mouse_pos(&mut self, x: i32, y: i32) {
        let mut ctx = self.ctx_rwl.write().unwrap();

        ctx.center =
            fractal_complex::map_pixel_value_rug(self.win.size(), &ctx.center, &ctx.window, (x, y));

        drop(ctx);
        self.reload_internal();
    }

    fn zoom_view_scrollwheel(&mut self, zoom: f32, x: i32, y: i32) {
        let ctr_pxl = Vector2f::new(self.win.size().x as f32, self.win.size().y as f32) / 2.0;
        let factor = 1.0 - 1.0 / zoom;
        let ctr_offset = Vector2f::new(ctr_pxl.x - x as f32, ctr_pxl.y - y as f32) * factor;
        let new_ctr_pxl = ctr_offset + ctr_pxl;

        let mut ctx = self.ctx_rwl.write().unwrap();
        ctx.center = fractal_complex::map_pixel_value_rug(
            self.win.size(),
            &ctx.center,
            &ctx.window,
            (new_ctr_pxl.x as i32, new_ctr_pxl.y as i32),
        );
        ctx.window *= zoom;
        drop(ctx);

        self.reload_internal();
    }

    fn manage_workers(&mut self, new_worker_count: usize) {
        let mut ctx = self.ctx_rwl.write().unwrap();
        if new_worker_count == self.workers.len() {
            panic!("Bruh qu'est-ce que tu me call, y'a rien changé ô_o");
        } else if new_worker_count < self.workers.len() {
            for _ in 0..(self.workers.len() - new_worker_count) {
                self.workers
                    .last()
                    .unwrap()
                    .tx
                    .send(WorkerNotif::Shutdown)
                    .unwrap();
                self.workers.pop().unwrap();
                ctx.reload_durs.pop().unwrap();
            }
        } else if new_worker_count > self.workers.len() {
            for id in 0..(new_worker_count - self.workers.len()) {
                let (worker_tx, internal_rx) = mpsc::channel();
                let (internal_tx, worker_rx) = mpsc::channel();
                let ctx_rwl_clone = self.ctx_rwl.clone();

                thread::Builder::new()
                    .name(format!("SFML Worker {}", self.workers.len() + id))
                    .spawn(|| {
                        SfmlEngineWorkerInternal::build_and_run(
                            internal_rx,
                            internal_tx,
                            ctx_rwl_clone,
                        )
                    })
                    .unwrap();

                self.workers.push(SfmlEngineWorkerExternal {
                    tx: worker_tx,
                    rx: worker_rx,
                });
                ctx.reload_durs.push(Duration::ZERO);
            }
        }
    }

    fn send_render_rect_workers(&mut self) {
        let rrect_top = self.texture.size().y / self.workers.len() as u32;
        for i in 0..self.workers.len() {
            let rrect_height = if i == self.workers.len() - 1 {
                self.texture.size().y - rrect_top * (self.workers.len() as u32 - 1)
            } else {
                rrect_top
            };
            let new_rrect = Rect {
                left: 0,
                top: i as u32 * rrect_top,
                width: self.texture.size().x,
                height: rrect_height,
            };
            self.workers[i]
                .tx
                .send(WorkerNotif::SetRenderRect(new_rrect))
                .unwrap();
        }
    }

    fn reload_internal(&mut self) {
        let (ctx_worker_count, ctx_lodiv, ctx_has_resized);
        {
            let ctx = self.ctx_rwl.read().unwrap();
            ctx_worker_count = ctx.worker_count;
            ctx_lodiv = ctx.lodiv;
            ctx_has_resized = ctx.has_resized;
        }

        // Create or Remove workers if necessary
        if self.workers.len() != ctx_worker_count {
            self.manage_workers(ctx_worker_count);
            self.send_render_rect_workers();
        }

        // Resize self.texture and worker's RenderRect if necessary
        if ctx_has_resized {
            self.ctx_rwl.write().unwrap().has_resized = false;

            // Changing Texture Size
            self.texture
                .create(self.win.size().x / ctx_lodiv, self.win.size().y / ctx_lodiv)
                .unwrap();

            // Changing Workers RenderRect
            self.send_render_rect_workers();
        }

        // Send the start message to the workers !
        for worker in &self.workers {
            worker.tx.send(WorkerNotif::Reload).unwrap();
        }

        // Receive raw pixel data and upload it to GPU
        for (id, worker) in self.workers.iter().enumerate() {
            let WorkerResult {
                pixels,
                rrect,
                reload_dur,
            } = worker.rx.recv().unwrap();

            self.ctx_rwl.write().unwrap().reload_durs[id] = reload_dur;

            self.texture.update_from_pixels(
                &*pixels,
                rrect.width,
                rrect.height,
                rrect.left,
                rrect.top,
            );
        }
    }
}
