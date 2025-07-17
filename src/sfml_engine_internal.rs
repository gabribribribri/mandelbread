use std::{
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
    time::Instant,
};

use rug::{Assign, ops::MulFrom};
use sfml::{
    cpp::FBox,
    graphics::{
        Color, FloatRect, Rect, RenderTarget, RenderWindow, Sprite, Texture, Transformable, View,
    },
    window::{ContextSettings, Event, Style},
};

use crate::{
    fractal_engine::{FractalContext, FractalNotif},
    sfml_engine_worker::SfmlEngineWorkerInternal,
};

pub struct SfmlEngineInternal<'a> {
    notif_rx: &'a Receiver<FractalNotif>,
    ctx_rwl: Arc<RwLock<FractalContext>>,
    win: FBox<RenderWindow>,
    texture: FBox<Texture>,
    workers: Vec<SfmlEngineWorkerExternal>,
}

struct SfmlEngineWorkerExternal {
    tx: Sender<WorkerNotif>,
    rx: Receiver<(Vec<u8>, Rect<u32>)>,
}

pub enum WorkerNotif {
    SetRenderRect(Rect<u32>),
    Reload,
    Shutdown,
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
            texture.create(ctx.res.x, ctx.res.y).unwrap();
            drop(ctx);

            let internal_engine = SfmlEngineInternal {
                notif_rx: &rx,
                ctx_rwl: Arc::clone(&ctx_rwl),
                win,
                texture,
                workers,
            };

            internal_engine.run_until_end();
        }
    }

    pub fn run_until_end(mut self) {
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

        assert!(width == self.win.size().x && height == self.win.size().y);

        let mut new_real = ctx.window.real().clone();
        new_real.mul_from(ctx.res.y as f32 / ctx.res.x as f32);
        ctx.window.mut_imag().assign(new_real);

        self.win.set_view(
            &*View::from_rect(FloatRect::new(0.0, 0.0, width as f32, height as f32)).unwrap(),
        );
    }

    fn reload_internal(&mut self) {
        // Start the chronometer
        let start = Instant::now();

        // Resize self.texture and worker's RenderRect if necessary
        if self.texture.size() != self.win.size() {
            // Changing Texture Size
            self.texture
                .create(self.win.size().x, self.win.size().y)
                .unwrap();

            // Changing Workers RenderRect
            let render_rect_width = self.texture.size().x / self.workers.len() as u32;
            for i in 0..self.workers.len() {
                self.workers[i]
                    .tx
                    .send(WorkerNotif::SetRenderRect(Rect {
                        left: i as u32 * render_rect_width,
                        top: 0,
                        width: render_rect_width,
                        height: self.texture.size().y,
                    }))
                    .unwrap();
            }
        }

        // Send the start message to the workers !
        for worker in &self.workers {
            worker.tx.send(WorkerNotif::Reload).unwrap();
        }

        // Receive raw pixel data and upload it to GPU
        for worker in &self.workers {
            let (pixels, rrect) = worker.rx.recv().unwrap();

            self.texture.update_from_pixels(
                &*pixels,
                rrect.width,
                rrect.height,
                rrect.left,
                rrect.top,
            );
        }

        // Stop the chronometer and report time
        self.ctx_rwl.write().unwrap().reload_dur = start.elapsed();
    }
}
