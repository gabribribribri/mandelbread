use std::{
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use egui::{RichText, Ui};
use rug::{self, Assign};
use sfml::{
    cpp::FBox,
    graphics::{
        Color, FloatRect, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture,
        Transformable, View,
    },
    window::{ContextSettings, Event, Style},
};

use crate::{
    fractal_complex::{Complex, FractalComplex},
    fractal_engine::{
        FractalBackend, FractalContext, FractalEngine, FractalEngineError, FractalNotif,
        lodiv::{self, FASTER},
    },
};

pub struct SfmlEngine {
    notif_tx: Sender<FractalNotif>,
    notif_rx: Receiver<FractalNotif>,
    ctx: FractalContext,

    reload_dur: Duration,
    engine_enabled: bool,
}

pub struct SfmlEngineInternal<'a> {
    notif_tx: &'a Sender<FractalNotif>,
    notif_rx: &'a Receiver<FractalNotif>,
    ctx: FractalContext,

    win: FBox<RenderWindow>,
    texture: FBox<Texture>,
}

impl SfmlEngine {
    pub fn new() -> SfmlEngine {
        let (ext_tx, in_rx) = mpsc::channel::<FractalNotif>();
        let (in_tx, ext_rx) = mpsc::channel::<FractalNotif>();

        thread::spawn(|| -> ! { SfmlEngine::engine_internal(in_tx, in_rx) });

        let fractal_ctx = FractalContext::default();

        ext_tx
            .send(FractalNotif::Commence(fractal_ctx.clone()))
            .unwrap();

        SfmlEngine {
            notif_tx: ext_tx,
            notif_rx: ext_rx,
            ctx: fractal_ctx,
            reload_dur: Duration::ZERO,
            engine_enabled: true,
        }
    }

    pub fn engine_internal(tx: Sender<FractalNotif>, rx: Receiver<FractalNotif>) -> ! {
        loop {
            let ctx = match rx.recv().unwrap() {
                FractalNotif::Commence(ctx) => ctx, // Time to start...
                _ => {
                    println!("IDGAF I'M NOT UP YET !");
                    continue;
                }
            };

            let mut win = RenderWindow::new(
                ctx.res,
                "Mandelbread SFML Engine",
                Style::DEFAULT,
                &ContextSettings::default(),
            )
            .unwrap();

            win.set_framerate_limit(60);

            let image = Image::new_solid(ctx.res.0, ctx.res.1, Color::MAGENTA).unwrap();
            let texture = Texture::from_image(&image, IntRect::default()).unwrap();

            let internal_engine = SfmlEngineInternal {
                notif_tx: &tx,
                notif_rx: &rx,
                ctx,
                win,
                texture,
            };

            internal_engine.run_until_end();
        }
    }

    fn handle_notifs(&mut self) {
        while let Ok(notif) = self.notif_rx.try_recv() {
            match notif {
                FractalNotif::ReloadTime(dur) => self.reload_dur = dur,
                FractalNotif::ChangeResolution(width, height) => self.ctx.res = (width, height),
                FractalNotif::Move(trsln) => self.ctx.center = trsln,
                FractalNotif::Zoom(zoom) => self.ctx.window *= zoom,
                FractalNotif::ChangeView(view) => self.ctx.window = view,
                _ => panic!("Shouldn't have received that"),
            }
        }
    }
}

impl FractalEngine for SfmlEngine {
    fn commence(&self) -> Result<(), FractalEngineError> {
        match self.notif_tx.send(FractalNotif::Commence(self.ctx.clone())) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Cannot start the internal engine : {}", e);
                Err(FractalEngineError::SendError)
            }
        }
    }

    fn shutdown(&mut self) -> Result<(), FractalEngineError> {
        match self.notif_tx.send(FractalNotif::Shutdown) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Cannot shutdown the internal engine : {}", e);
                Err(FractalEngineError::SendError)
            }
        }
    }

    fn reload(&mut self) -> Result<(), FractalEngineError> {
        match self.notif_tx.send(FractalNotif::Reload) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Cannot reload the internal engine : {}", e);
                Err(FractalEngineError::SendError)
            }
        }
    }

    fn move_view(&mut self, translation: rug::Complex) -> Result<(), FractalEngineError> {
        match self.notif_tx.send(FractalNotif::Move(translation)) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Cannot move the view : {}", e);
                Err(FractalEngineError::SendError)
            }
        }
    }

    fn zoom_view(&mut self, zoom: f32) -> Result<(), FractalEngineError> {
        self.ctx.window *= zoom;

        match self
            .notif_tx
            .send(FractalNotif::ChangeView(self.ctx.window.clone()))
        {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Cannot zoom : {}", e);
                Err(FractalEngineError::SendError)
            }
        }
    }

    fn gui_central_panel(&mut self, ui: &mut Ui) {
        ui.heading("SFML Engine");
        ui.separator();

        if ui.checkbox(&mut self.engine_enabled, "Enabled").clicked() {
            match self.engine_enabled {
                true => self.commence(),
                false => self.shutdown(),
            }
            .unwrap()
        }

        if ui
            .button(RichText::new("RELOAD").size(12.0).extra_letter_spacing(3.0))
            .clicked()
        {
            self.reload().unwrap()
        }
    }

    fn gui_bottom_panel(&mut self, ui: &mut Ui) {
        self.handle_notifs();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Reload Time :").strong());
            ui.label(format!("{:?}", self.reload_dur));
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Resolution :").strong());
            ui.label(format!("{}x{}", self.ctx.res.0, self.ctx.res.1));
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Center : ").strong());
            ui.label(format!(
                "{:.5}{:+.5}i",
                self.ctx.center.real(),
                self.ctx.center.imag()
            ));
            ui.label(RichText::new("          Window : ").strong());
            ui.label(format!(
                "{:.5}{:+.5}i",
                self.ctx.window.real(),
                self.ctx.window.imag()
            ));
        });
    }
}

impl<'a> SfmlEngineInternal<'a> {
    fn run_until_end(mut self) {
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
                FractalNotif::Shutdown => self.shutdown_internal(),
                FractalNotif::Reload => self.choose_reload_internal(lodiv::QUALITY),
                FractalNotif::Move(trsln) => self.move_view_internal(trsln),
                FractalNotif::Zoom(zoom) => self.zoom_view_internal(zoom),
                FractalNotif::ChangeResolution(width, height) => {
                    self.resize_internal(width, height)
                }
                FractalNotif::ChangeView(view) => self.set_view_internal(view),
                FractalNotif::Commence(_) => panic!("Uh bro I'm already running"),
                FractalNotif::ReloadTime(_) => {
                    panic!("I am not supposed to get back a reload time")
                }
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("The connexion shouldn't be disconnected"),
        }
    }

    fn render_internal(&mut self) {
        let mut sprite = Sprite::with_texture(&self.texture);

        self.win.clear(Color::CYAN);
        self.win.draw(&sprite);
        self.win.display();
    }

    fn shutdown_internal(&mut self) {
        self.win.close();
    }

    fn resize_internal(&mut self, width: u32, height: u32) {
        self.win.set_view(
            &*View::from_rect(FloatRect::new(0.0, 0.0, width as f32, height as f32)).unwrap(),
        );

        self.ctx.res = (width, height);

        let new_win_imag = self.ctx.window.real().clone() * (height as f32 / width as f32);
        self.ctx.window.mut_imag().assign(new_win_imag);

        self.notif_tx
            .send(FractalNotif::ChangeResolution(width, height))
            .unwrap();

        self.notif_tx
            .send(FractalNotif::ChangeView(self.ctx.window.clone()))
            .unwrap();

        // self.choose_reload_internal(lodiv::FAST);
    }

    fn move_view_internal(&mut self, translation: rug::Complex) {
        self.ctx.center += translation;
        // self.choose_reload_internal(lodiv::FAST);
    }

    fn zoom_view_internal(&mut self, factor: f32) {
        self.ctx.window *= factor;
        // self.choose_reload_internal(lodiv::FAST);
    }

    fn set_view_internal(&mut self, view: rug::Complex) {
        self.ctx.window = view;
        // self.choose_reload_internal(lodiv::FAST);
    }

    fn choose_reload_internal(&mut self, lodiv: u32) {
        let now = Instant::now();

        match self.ctx.backend {
            FractalBackend::F32 => self.reload_internal::<Complex<f32>>(lodiv),
            // FractalBackend::F64 => self.reload_internal::<f64>(),
            _ => panic!("Is not implemented yet !!"),
        }

        self.notif_tx
            .send(FractalNotif::ReloadTime(now.elapsed()))
            .unwrap()
    }

    fn reload_internal<T: FractalComplex>(&mut self, lodiv: u32) {
        let res_lodiv = (self.ctx.res.0 / lodiv, self.ctx.res.1 / lodiv);

        let center_as_t = T::from_cmplx(&self.ctx.center);
        let window_as_t = T::from_cmplx(&self.ctx.window);

        let res_as_t = T::from_u32_pair(res_lodiv);

        let mut new_image = Image::new_solid(res_lodiv.0, res_lodiv.1, Color::MAGENTA).unwrap();

        for x in 0..res_lodiv.0 {
            for y in 0..res_lodiv.1 {
                let c = T::map_pixel_value(
                    res_as_t,
                    center_as_t,
                    window_as_t,
                    T::from_u32_pair((x, y)),
                );
                let mut n = c;
                let mut distance = T::float_val_0();
                for _i in 1..=99 {
                    n.fsq_add(c);
                    distance = n.distance_origin();
                    if distance >= T::float_val_100() {
                        break;
                    }
                }
                if distance <= T::float_val_100() {
                    new_image.set_pixel(x, y, Color::BLACK).unwrap()
                } else {
                    new_image
                        .set_pixel(x, y, T::distance_gradient(distance))
                        .unwrap();
                }
            }
        }

        // Send image to the GPU
        self.texture
            .load_from_image(&new_image, IntRect::default())
            .unwrap();
    }
}
