use std::{
    error::Error,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, SendError, Sender, TryRecvError},
    },
    thread,
    time::{Duration, Instant},
};

use sfml::{
    cpp::FBox,
    graphics::{
        Color, FloatRect, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture, View,
    },
    window::{ContextSettings, Event, Style},
};

use crate::{
    complex::Complex,
    fractal_engine::{
        FractalAction, FractalContext, FractalEngine, FractalInfoNotif, FractalInfos,
    },
    utils::distance_gradient,
};

pub struct SfmlEngine {
    action_sender: Sender<FractalAction>,
    info_receiver: Receiver<FractalInfoNotif>,
    fractal_ctx: Arc<Mutex<FractalContext<f32>>>,
}

pub struct SfmlEngineInternal {
    win: FBox<RenderWindow>,
    texture: FBox<Texture>,
    action_receiver: Receiver<FractalAction>,
    info_sender: Sender<FractalInfoNotif>,
    fractal_ctx: Arc<Mutex<FractalContext<f32>>>,
}

impl SfmlEngine {
    pub fn spawn() -> Box<SfmlEngine> {
        let (action_tx, action_rx) = mpsc::channel::<FractalAction>();

        let (info_tx, info_rx) = mpsc::channel::<FractalInfoNotif>();

        let ctx = Arc::new(Mutex::new(FractalContext::default()));

        let ctx_clone = Arc::clone(&ctx);

        thread::spawn(|| {
            // CONSTRUCT INTERNAL ENGINE

            println!("The engine is constructing...");

            let ctx_clone_locked = ctx_clone.lock().unwrap();

            let mut win = RenderWindow::new(
                ctx_clone_locked.res,
                "Mandelbread",
                Style::DEFAULT,
                &ContextSettings::default(),
            )
            .unwrap();

            win.set_framerate_limit(60);

            let image = Image::new_solid(
                ctx_clone_locked.res.0,
                ctx_clone_locked.res.1,
                Color::rgb(32, 0, 0),
            )
            .unwrap();

            drop(ctx_clone_locked);

            let texture = Texture::from_image(&image, IntRect::default()).unwrap();

            let mut internal_engine = SfmlEngineInternal {
                win,
                texture,
                action_receiver: action_rx,
                info_sender: info_tx,
                fractal_ctx: ctx_clone,
            };

            println!("The engine is starting !!");

            // MAKE INTERNAL ENGINE WORK
            internal_engine.run();

            println!("The engine stopped running...")
        });

        Box::new(SfmlEngine {
            action_sender: action_tx,
            info_receiver: info_rx,
            fractal_ctx: ctx,
        })
    }
}

impl FractalEngine for SfmlEngine {
    fn reload(&mut self) {
        match self.action_sender.send(FractalAction::Reload) {
            Ok(_) => (),
            Err(e) => println!("SFML Engine will not reload : {}", e.to_string()),
        }
    }

    fn render(&mut self) {
        // bats les couilles
    }

    fn shutdown(&mut self) {
        self.action_sender.send(FractalAction::Shutdown).unwrap();
    }

    fn get_ctx(&self) -> FractalContext<f64> {
        let ctx = self.fractal_ctx.lock().unwrap();
        FractalContext {
            res: ctx.res,
            start: ctx.start.into(),
            end: ctx.end.into(),
        }
    }

    fn get_infos(&self) -> FractalInfos {
        let mut infos = FractalInfos::new();
        while let Ok(notif) = self.info_receiver.try_recv() {
            match notif {
                FractalInfoNotif::ReloadTime(dur) => infos.reload_time = Some(dur),
            }
        }
        return infos;
    }
}

impl SfmlEngineInternal {
    fn run(&mut self) {
        while self.win.is_open() {
            self.handle_events();
            self.handle_messages_received();
            self.render();
        }
    }

    fn handle_events(&mut self) {
        while let Some(event) = self.win.poll_event() {
            match event {
                Event::Closed => self.win.close(),
                Event::Resized { width, height } => self.resize(width, height),
                _ => (),
            }
        }
    }

    fn handle_messages_received(&mut self) {
        match self.action_receiver.try_recv() {
            Ok(action) => match action {
                FractalAction::Shutdown => self.win.close(),
                FractalAction::Reload => self.reload(),
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("Connexion shouldn't be disconnected"),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        let mut ctx_val = self.fractal_ctx.lock().unwrap();
        ctx_val.res.0 = width;
        ctx_val.res.1 = height;
        drop(ctx_val);
        self.win.set_view(
            &*View::from_rect(FloatRect::new(0.0, 0.0, width as f32, height as f32)).unwrap(),
        );
    }

    // Reload and send reload time
    fn reload(&mut self) {
        let now = Instant::now();

        let ctx_val;
        {
            ctx_val = self.fractal_ctx.lock().unwrap();
        }

        let mut new_image =
            Image::new_solid(ctx_val.res.0, ctx_val.res.1, Color::rgb(32, 0, 0)).unwrap();

        for x in 0..ctx_val.res.0 {
            for y in 0..ctx_val.res.1 {
                let c = Complex::map_between(ctx_val.res, ctx_val.start, ctx_val.end, (x, y));
                let mut n = c;
                let mut distance = 0.0;
                for _ in 1..=99 {
                    n.sq_add(c);
                    distance = n.re.abs() + n.im.abs();
                    if distance >= 100.0 {
                        break;
                    }
                }
                if distance <= 100.0 {
                    new_image.set_pixel(x, y, Color::BLACK).unwrap();
                } else {
                    new_image
                        .set_pixel(x, y, distance_gradient::<100, 1500>(distance).into())
                        .unwrap();
                }
            }
        }

        // Send image to the GPU
        self.texture
            .load_from_image(&new_image, IntRect::default())
            .unwrap();

        self.info_sender
            .send(FractalInfoNotif::ReloadTime(now.elapsed()))
            .unwrap()
    }

    fn render(&mut self) {
        let sprite = Sprite::with_texture(&self.texture);
        self.win.clear(Color::rgb(0, 32, 0));
        self.win.draw(&sprite);
        self.win.display();
    }
}
