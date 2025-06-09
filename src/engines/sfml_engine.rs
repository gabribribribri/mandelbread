use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
    time::Instant,
};

use ::num::{
    FromPrimitive, Zero,
    complex::{Complex, Complex32, Complex64, ComplexFloat},
};
use sfml::{
    cpp::FBox,
    graphics::{
        Color, FloatRect, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture, View,
    },
    window::{ContextSettings, Event, Key, Style},
};

use crate::{
    fractal_engine::{
        FractalAction, FractalContext, FractalEngine, FractalInfoNotif, FractalInfos,
        FractalPrecision,
    },
    utils::{self, distance_gradient, distance_gradient_f32},
};
macro_rules! generate_reload_float {
    (
        $float_type:ident,
        $fn_name: ident
    ) => {
        fn $fn_name(&mut self) {
            let ctx_val = self.fractal_ctx.lock().unwrap();
            let start_t = Complex::new(
                ctx_val.start.re as $float_type,
                ctx_val.start.im as $float_type,
            );
            let end_t = Complex::new(ctx_val.end.re as $float_type, ctx_val.end.im as $float_type);

            let mut new_image =
                Image::new_solid(ctx_val.res.0, ctx_val.res.1, Color::rgb(32, 0, 0)).unwrap();

            for x in 0..ctx_val.res.0 {
                for y in 0..ctx_val.res.1 {
                    let c = utils::map_between_f32(ctx_val.res, start_t, end_t, (x, y));
                    let mut n = c;
                    let mut distance = 0.0;
                    for _ in 1..=99 {
                        utils::fsq_add_f32(&mut n, c);
                        distance = n.re.abs() + n.im.abs();
                        if distance >= 100.0 {
                            break;
                        }
                    }
                    if distance <= 100.0 {
                        new_image.set_pixel(x, y, Color::BLACK).unwrap();
                    } else {
                        new_image
                            .set_pixel(x, y, distance_gradient_f32::<100, 1500>(distance).into())
                            .unwrap();
                    }
                }
            }

            // Send image to the GPU
            self.texture
                .load_from_image(&new_image, IntRect::default())
                .unwrap();
        }
    };
}

pub struct SfmlEngine {
    action_sender: Sender<FractalAction>,
    info_receiver: Receiver<FractalInfoNotif>,
    fractal_ctx: Arc<Mutex<FractalContext<f64>>>,
    fractal_infos: FractalInfos,
}

pub struct SfmlEngineInternal<'a> {
    win: FBox<RenderWindow>,
    texture: FBox<Texture>,
    action_receiver: &'a Receiver<FractalAction>,
    info_sender: &'a Sender<FractalInfoNotif>,
    fractal_ctx: Arc<Mutex<FractalContext<f64>>>,
}

impl SfmlEngine {
    pub fn new() -> SfmlEngine {
        let (action_sender, action_receiver) = mpsc::channel::<FractalAction>();

        let (info_sender, info_receiver) = mpsc::channel::<FractalInfoNotif>();

        let fractal_ctx = Arc::new(Mutex::new(FractalContext::default()));

        let fractal_ctx_clone = Arc::clone(&fractal_ctx);

        thread::spawn(|| -> ! {
            SfmlEngine::thread_internal(fractal_ctx_clone, action_receiver, info_sender)
        });

        // Start the internal engine
        action_sender.send(FractalAction::Commence).unwrap();

        SfmlEngine {
            action_sender,
            info_receiver,
            fractal_ctx,
            fractal_infos: FractalInfos::default(),
        }
    }

    fn thread_internal(
        // The parameters are owned by the function and must not be dropped
        fractal_ctx: Arc<Mutex<FractalContext<f64>>>,
        action_receiver: Receiver<FractalAction>,
        info_sender: Sender<FractalInfoNotif>,
    ) -> ! {
        loop {
            // WAIT FOR SIGNAL TO START
            match action_receiver.recv().unwrap() {
                FractalAction::Commence => (), // It starts
                _ => continue,
            }

            // CONSTRUCT THE ENGINE
            let ctx_val = fractal_ctx.lock().unwrap();

            let mut win = RenderWindow::new(
                ctx_val.res,
                "Mandelbread",
                Style::DEFAULT,
                &ContextSettings::default(),
            )
            .unwrap();

            win.set_framerate_limit(60);

            let image =
                Image::new_solid(ctx_val.res.0, ctx_val.res.1, Color::rgb(32, 0, 0)).unwrap();

            drop(ctx_val);

            let texture = Texture::from_image(&image, IntRect::default()).unwrap();

            let mut internal_engine = SfmlEngineInternal {
                win,
                texture,
                action_receiver: &action_receiver,
                info_sender: &info_sender,
                fractal_ctx: Arc::clone(&fractal_ctx),
            };

            // MAKE INTERNAL ENGINE WORK
            internal_engine.run_until_end();

            // DROP THE VALUES
            drop(internal_engine);
        }
    }
}

impl FractalEngine for SfmlEngine {
    fn commence(&mut self) {
        match self.action_sender.send(FractalAction::Commence) {
            Err(e) => println!("Could not start the internal engine : {}", e.to_string()),
            _ => (),
        }
    }

    fn shutdown(&mut self) {
        match self.action_sender.send(FractalAction::Shutdown) {
            Err(e) => println!("Cannot shutdown : {}", e.to_string()),
            _ => (),
        }
    }

    fn reload(&mut self) {
        match self.action_sender.send(FractalAction::Reload) {
            Err(e) => println!("SFML Engine will not reload : {}", e.to_string()),
            _ => (),
        }
    }

    fn render(&mut self) {
        // bats les couilles
    }

    fn get_ctx(&self) -> FractalContext<f64> {
        let ctx = self.fractal_ctx.lock().unwrap();
        FractalContext {
            res: ctx.res,
            start: ctx.start.into(),
            end: ctx.end.into(),
            precision: ctx.precision,
        }
    }

    fn get_infos(&mut self) -> FractalInfos {
        while let Ok(notif) = self.info_receiver.try_recv() {
            match notif {
                FractalInfoNotif::ReloadTime(dur) => self.fractal_infos.reload_time = Some(dur),
            }
        }
        self.fractal_infos
    }

    fn move_view(&mut self, c: Complex64) {
        match self.action_sender.send(FractalAction::Move(c)) {
            Err(e) => println!("Cannot move : {}", e.to_string()),
            _ => (),
        }
    }
}

impl<'a> SfmlEngineInternal<'a> {
    fn shutdown(&mut self) {
        self.win.close();
    }

    fn run_until_end(&mut self) {
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
                Event::KeyPressed {
                    code,
                    scan,
                    alt,
                    ctrl,
                    shift,
                    system,
                } => match code {
                    Key::W => self.move_view(Complex64::new(0.0, 1.0)),
                    Key::A => self.move_view(Complex64::new(-1.0, 0.0)),
                    Key::S => self.move_view(Complex64::new(0.0, -1.0)),
                    Key::D => self.move_view(Complex64::new(1.0, 0.0)),
                    Key::R => self.choose_and_reload(),

                    _ => (),
                },
                _ => (),
            }
        }
    }

    fn handle_messages_received(&mut self) {
        match self.action_receiver.try_recv() {
            Ok(action) => match action {
                FractalAction::Shutdown => self.shutdown(),
                FractalAction::Reload => self.choose_and_reload(),
                FractalAction::Move(c) => self.move_view(c),
                FractalAction::Commence => println!("[SFML THREAD]: Bro wtf I'm already running"),
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

    fn choose_and_reload(&mut self) {
        let now = Instant::now();

        let precision = self.fractal_ctx.lock().unwrap().precision;
        match precision {
            FractalPrecision::F32 => self.reload_f32(),
            FractalPrecision::F64 => panic!(),
        }

        self.info_sender
            .send(FractalInfoNotif::ReloadTime(now.elapsed()))
            .unwrap()
    }

    generate_reload_float!(f32, reload_f32);
    // generate_reload_float!(f64, reload_f64);

    // Reload and send reload time
    fn render(&mut self) {
        let sprite = Sprite::with_texture(&self.texture);
        self.win.clear(Color::rgb(0, 32, 0));
        self.win.draw(&sprite);
        self.win.display();
    }

    fn move_view(&mut self, c: Complex64) {
        let mut ctx = self.fractal_ctx.lock().unwrap();
        ctx.start += c;
        ctx.end += c;
    }
}
