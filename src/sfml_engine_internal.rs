use std::{
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
    time::{Duration, Instant},
};

use rug::{Assign, ops::MulFrom};
use sfml::{
    cpp::FBox,
    graphics::{
        Color, FloatRect, Rect, RectangleShape, RenderTarget, RenderTexture, RenderWindow, Shader,
        Sprite, Texture, Transformable, View,
    },
    system::Vector2f,
    window::{ContextSettings, Event, Style, mouse::Button},
};

use crate::{
    fractal_complex,
    fractal_engine::{self, FractalBackend, FractalContext, FractalNotif},
    sfml_engine_worker::SfmlEngineWorkerInternal,
};

pub struct SfmlEngineInternal<'a> {
    notif_rx: &'a Receiver<FractalNotif>,
    ctx_rwl: Arc<RwLock<FractalContext>>,
    workers: Vec<SfmlEngineWorkerExternal>,
    win: FBox<RenderWindow>,
    texture: FBox<Texture>,
    render_texture: FBox<RenderTexture>,
    shader: FBox<Shader<'a>>,
    backend: FractalBackend,
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

            let mut texture = Texture::new().unwrap();
            texture
                .create(ctx.res.x / ctx.lodiv, ctx.res.y / ctx.lodiv)
                .unwrap();

            let render_texture =
                RenderTexture::new(ctx.res.x / ctx.lodiv, ctx.res.y / ctx.lodiv).unwrap();

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

            let shader = Shader::from_file_vert_frag("src/vertex.glsl", "src/fragment.glsl")
                .expect("Failed to load shaders");

            let backend = ctx.backend;

            drop(ctx);

            let internal_engine = SfmlEngineInternal {
                notif_rx: &rx,
                ctx_rwl: Arc::clone(&ctx_rwl),
                win,
                texture,
                render_texture,
                workers,
                shader,
                backend,
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
                FractalNotif::ChangeBackend(backend) => self.backend = backend,
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("The connexion shouldn't be disconnected"),
        }
    }

    fn render_internal(&mut self) {
        let mut sprite;

        match self.backend {
            FractalBackend::F64 | FractalBackend::Rug => {
                sprite = Sprite::with_texture(&self.texture);
            }
            FractalBackend::Shader => {
                sprite = Sprite::with_texture(self.render_texture.texture());
            }
        }

        sprite.set_scale((
            self.win.size().x as f32 / sprite.texture_rect().width as f32,
            self.win.size().y as f32 / sprite.texture_rect().height as f32,
        ));

        self.win.clear(Color::rgb(64, 0, 0));
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
        if ctx.auto_seq_iter {
            ctx.seq_iter = fractal_engine::seq_iters_formula(&ctx.window, ctx.auto_seq_iter_fact);
        }
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

    fn adjust_workers_if_needed(&mut self) {
        let ctx_worker_count = self.ctx_rwl.read().unwrap().worker_count;

        // Create or Remove workers if necessary
        if self.workers.len() != ctx_worker_count {
            self.manage_workers(ctx_worker_count);
            self.send_render_rect_workers();
        }
    }

    fn adjust_textures_if_needed(&mut self) {
        // Resize self.texture and worker's RenderRect if necessary
        if self.ctx_rwl.read().unwrap().has_resized {
            let mut ctx = self.ctx_rwl.write().unwrap();
            ctx.has_resized = false;

            // Changing Texture Size
            self.texture
                .create(ctx.res.x / ctx.lodiv, ctx.res.y / ctx.lodiv)
                .unwrap();

            // Changing RenderTexture Size
            self.render_texture
                .recreate(
                    ctx.res.x / ctx.lodiv,
                    ctx.res.y / ctx.lodiv,
                    &ContextSettings::default(),
                )
                .unwrap();

            // Changing Workers RenderRect
            drop(ctx);
            self.send_render_rect_workers();
        }
    }

    fn adjust_uniforms(&mut self) {
        // TODO Fix this ugliness
        for dur in &mut self.ctx_rwl.write().unwrap().reload_durs {
            *dur = Duration::ZERO;
        }

        let ctx = self.ctx_rwl.read().unwrap();

        self.shader
            .set_uniform_vec2(
                "u_Resolution",
                Vector2f::new(
                    self.render_texture.size().x as f32 / ctx.lodiv as f32,
                    self.render_texture.size().y as f32 / ctx.lodiv as f32,
                ),
            )
            .unwrap();

        self.shader
            .set_uniform_vec4(
                "u_Center",
                fractal_engine::two_f64_to_vec4(
                    ctx.center.real().to_f64(),
                    ctx.center.imag().to_f64(),
                ),
            )
            .unwrap();

        self.shader
            .set_uniform_vec4(
                "u_Window",
                fractal_engine::two_f64_to_vec4(
                    ctx.window.real().to_f64(),
                    ctx.window.imag().to_f64(),
                ),
            )
            .unwrap();

        self.shader
            .set_uniform_int("u_SeqIter", ctx.seq_iter as i32)
            .unwrap();

        self.shader
            .set_uniform_float("u_ConvergeDistance", ctx.converge_distance as f32)
            .unwrap();
    }

    fn reload_internal(&mut self) {
        match self.backend {
            FractalBackend::F64 | FractalBackend::Rug => self.prepare_and_reload_internal_cpu(),
            FractalBackend::Shader => self.prepare_and_reload_internal_gpu(),
        }
    }

    fn prepare_and_reload_internal_cpu(&mut self) {
        // Prepare
        self.adjust_workers_if_needed();
        self.adjust_textures_if_needed();

        // Reload
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

    fn prepare_and_reload_internal_gpu(&mut self) {
        // Prepare
        self.adjust_textures_if_needed();
        self.adjust_uniforms();

        // Render
        let start = Instant::now();
        let states = sfml::graphics::RenderStates {
            shader: Some(&self.shader),
            ..Default::default()
        };
        let rect_to_render = RectangleShape::with_size(Vector2f::new(
            self.render_texture.size().x as f32,
            self.render_texture.size().y as f32,
        ));
        self.render_texture
            .draw_with_renderstates(&rect_to_render, &states);
        self.render_texture.display();
        self.ctx_rwl.write().unwrap().reload_durs[0] = start.elapsed();
    }
}
