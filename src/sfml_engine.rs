use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
    time::Instant,
};

use egui::{RichText, Ui};
use rug::{
    Assign,
    ops::{AddFrom, MulFrom},
};
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
        FRCTL_CTX_CMPLX_PREC, FractalBackend, FractalContext, FractalEngine, FractalEngineError,
        FractalNotif, lodiv,
    },
};

pub struct SfmlEngine {
    notif_tx: Sender<FractalNotif>,
    ctx_mx: Arc<Mutex<FractalContext>>,
}

pub struct SfmlEngineInternal<'a> {
    notif_rx: &'a Receiver<FractalNotif>,
    ctx_mx: Arc<Mutex<FractalContext>>,

    win: FBox<RenderWindow>,
    texture: FBox<Texture>,
}

impl SfmlEngine {
    pub fn new() -> SfmlEngine {
        let (ext_tx, in_rx) = mpsc::channel::<FractalNotif>();

        let ctx_mx = Arc::<Mutex<FractalContext>>::default();

        let ctx_mx_clone = Arc::clone(&ctx_mx);

        thread::spawn(|| -> ! { SfmlEngine::engine_internal(ctx_mx_clone, in_rx) });

        ext_tx.send(FractalNotif::Commence).unwrap();

        SfmlEngine {
            notif_tx: ext_tx,
            ctx_mx: ctx_mx,
        }
    }

    pub fn engine_internal(ctx_mx: Arc<Mutex<FractalContext>>, rx: Receiver<FractalNotif>) -> ! {
        loop {
            match rx.recv().unwrap() {
                FractalNotif::Commence => (), // Time to start...
                _ => {
                    println!("IDGAF I'M NOT UP YET !");
                    continue;
                }
            };

            let ctx = ctx_mx.lock().unwrap();

            let mut win = RenderWindow::new(
                (ctx.res.x, ctx.res.y),
                "Mandelbread SFML Engine",
                Style::DEFAULT,
                &ContextSettings::default(),
            )
            .unwrap();

            win.set_framerate_limit(60);

            let image = Image::new_solid(ctx.res.x, ctx.res.y, Color::MAGENTA).unwrap();
            let texture = Texture::from_image(&image, IntRect::default()).unwrap();

            drop(ctx);

            let internal_engine = SfmlEngineInternal {
                notif_rx: &rx,
                ctx_mx: Arc::clone(&ctx_mx),
                win,
                texture,
            };

            internal_engine.run_until_end();
        }
    }
}

impl FractalEngine for SfmlEngine {
    fn commence(&self) -> Result<(), FractalEngineError> {
        match self.notif_tx.send(FractalNotif::Commence) {
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

    fn reset_view(&mut self) -> Result<(), FractalEngineError> {
        let mut ctx = self.ctx_mx.lock().unwrap();
        ctx.center = rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, -0.5);
        ctx.window = rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, (2.66, 2.0));
        Ok(())
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

    fn move_window(&mut self, trsln: Complex<f32>) -> Result<(), FractalEngineError> {
        let mut ctx = self.ctx_mx.lock().unwrap();

        // WARNING CODE REPETITION
        let mut real_offset = ctx.window.real().clone();
        real_offset.mul_from(0.5 * trsln.re);
        let mut imag_offset = ctx.window.imag().clone();
        imag_offset.mul_from(-0.5 * trsln.im); // why minus ??
        ctx.center.add_from(real_offset);
        ctx.center.mut_imag().add_from(imag_offset);

        Ok(())
    }

    fn zoom_view(&mut self, zoom: f32) -> Result<(), FractalEngineError> {
        {
            self.ctx_mx.lock().unwrap().window *= zoom;
        }
        self.reload()
    }

    fn change_lodiv(&mut self, lodiv: u32) -> Result<(), FractalEngineError> {
        {
            self.ctx_mx.lock().unwrap().lodiv = lodiv;
        }
        self.reload()
    }

    fn change_backend(&mut self, backend: FractalBackend) -> Result<(), FractalEngineError> {
        {
            self.ctx_mx.lock().unwrap().backend = backend;
        }
        self.reload()
    }

    fn gui_central_panel(&mut self, ui: &mut Ui) {
        let mut ctx;
        {
            ctx = self.ctx_mx.lock().unwrap().clone();
        } // drops the mic

        ui.heading("SFML Engine");
        ui.separator();

        if ui.checkbox(&mut ctx.engine_enabled, "Enabled").clicked() {
            match ctx.engine_enabled {
                true => self.commence(),
                false => self.shutdown(),
            }
            .unwrap()
        }

        if ui
            .radio_value(&mut ctx.backend, FractalBackend::F32, "32-bit float")
            .clicked()
        {
            self.change_backend(FractalBackend::F32).unwrap();
        }

        if ui
            .radio_value(&mut ctx.backend, FractalBackend::F64, "64-bit float")
            .clicked()
        {
            self.change_backend(FractalBackend::F64).unwrap();
        }

        if ui
            .button(RichText::new("RELOAD").size(12.0).extra_letter_spacing(3.0))
            .clicked()
        {
            self.reload().unwrap()
        }

        ui.horizontal(|ui| {
            ui.label("Quality : ");
            if ui
                .add(
                    egui::DragValue::new(&mut ctx.lodiv)
                        .range(1..=25)
                        .speed(0.04),
                )
                .dragged()
            {
                self.change_lodiv(ctx.lodiv).unwrap();
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::HIGHEST, "HIGHEST")
                .clicked()
            {
                self.change_lodiv(lodiv::HIGHEST).unwrap();
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::FAST, "FAST")
                .clicked()
            {
                self.change_lodiv(lodiv::FAST).unwrap();
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::FASTER, "FASTER")
                .clicked()
            {
                self.change_lodiv(lodiv::FASTER).unwrap();
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::FASTEST, "FASTEST")
                .clicked()
            {
                self.change_lodiv(lodiv::FASTEST).unwrap();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Movements :");
            if ui.button("Left").clicked() {
                self.move_window(Complex::new(-0.2, 0.0)).unwrap();
            }
            if ui.button("Down").clicked() {
                self.move_window(Complex::new(0.0, -0.2)).unwrap();
            }
            if ui.button("Up").clicked() {
                self.move_window(Complex::new(0.0, 0.2)).unwrap();
            }
            if ui.button("Right").clicked() {
                self.move_window(Complex::new(0.2, 0.0)).unwrap();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Zoom : ");
            if ui.button("Outside").clicked() {
                self.zoom_view(1.1).unwrap();
            }
            if ui.button("Inside").clicked() {
                self.zoom_view(0.9).unwrap();
            }
        });

        if ui.button("RESET VIEW").clicked() {
            self.reset_view().unwrap();
        }
    }

    fn gui_bottom_panel(&mut self, ui: &mut Ui) {
        let ctx = self.ctx_mx.lock().unwrap();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Reload Time :").strong());
            ui.label(format!("{:?}", ctx.reload_dur));
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Resolution :").strong());
            ui.label(format!("{}x{}", ctx.res.x, ctx.res.y));
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Center : ").strong());
            ui.label(format!(
                "{:.5}{:+.5}i",
                ctx.center.real(),
                ctx.center.imag()
            ));
            ui.label(RichText::new("          Window : ").strong());
            ui.label(format!(
                "{:.5}{:+.5}i",
                ctx.window.real(),
                ctx.window.imag()
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
                FractalNotif::Commence => panic!("bah bro je roule déjà..."),
                FractalNotif::Shutdown => self.shutdown_internal(),
                FractalNotif::Reload => self.choose_reload_internal(),
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("The connexion shouldn't be disconnected"),
        }
    }

    fn render_internal(&mut self) {
        // TODO FIX THIS LATER
        let ctx = self.ctx_mx.lock().unwrap();
        let mut sprite = Sprite::with_texture(&self.texture);
        sprite.set_scale((
            ctx.res.x as f32 / sprite.texture_rect().width as f32,
            ctx.res.y as f32 / sprite.texture_rect().height as f32,
        ));

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
    }

    fn choose_reload_internal(&mut self) {
        let (lodiv, backend);
        {
            let ctx = self.ctx_mx.lock().unwrap();
            lodiv = ctx.lodiv;
            backend = ctx.backend;
        }

        let now = Instant::now();

        match backend {
            FractalBackend::F32 => self.reload_internal::<Complex<f32>>(lodiv),
            FractalBackend::F64 => self.reload_internal::<Complex<f64>>(lodiv),
        }

        // TODO FIX THERE ARE TWO MUTEXES UNWRAP IN THIS FUNCTION
        self.ctx_mx.lock().unwrap().reload_dur = now.elapsed();
    }

    fn reload_internal<T: FractalComplex>(&mut self, lodiv: u32) {
        let mut ctx = self.ctx_mx.lock().unwrap();

        if ctx.res != self.win.size() {
            ctx.res = self.win.size();

            let mut new_real = ctx.window.real().clone();
            new_real.mul_from(ctx.res.y as f32 / ctx.res.x as f32);
            ctx.window.mut_imag().assign(new_real);
        }

        let res_lodiv = (ctx.res.x / lodiv, ctx.res.y / lodiv);

        // FIX MAYBE SOMETHING TO INVESTIGATE
        let center_as_t = T::from_cmplx(&ctx.center);
        let window_as_t = T::from_cmplx(&ctx.window);

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
