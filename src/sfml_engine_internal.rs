use std::{
    sync::{
        Arc, RwLock,
        mpsc::{Receiver, TryRecvError},
    },
    time::Instant,
};

use rug::{Assign, ops::MulFrom};
use sfml::{
    cpp::FBox,
    graphics::{
        Color, FloatRect, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture,
        Transformable, View,
    },
    window::{ContextSettings, Event, Style},
};

use crate::{
    fractal_complex::Complex,
    fractal_engine::{FractalBackend, FractalContext, FractalNotif},
};

pub struct SfmlEngineInternal<'a> {
    notif_rx: &'a Receiver<FractalNotif>,
    ctx_mx: Arc<RwLock<FractalContext>>,
    win: FBox<RenderWindow>,
    texture: FBox<Texture>,
}

impl<'a> SfmlEngineInternal<'a> {
    pub fn run(ctx_mx: Arc<RwLock<FractalContext>>, rx: Receiver<FractalNotif>) -> ! {
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

            let ctx = ctx_mx.read().unwrap();

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
            // let res = ctx.res;

            drop(ctx);

            let internal_engine = SfmlEngineInternal {
                notif_rx: &rx,
                ctx_mx: Arc::clone(&ctx_mx),
                // res,
                win,
                texture,
            };

            internal_engine.run_until_end();
        }
    }

    pub fn run_until_end(mut self) {
        while self.win.is_open() {
            self.handle_events_internal();
            self.handle_notifs_internal();
            self.render_internal();
            // No need to wait since the framerate is capped at 60
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
    }

    fn resize_internal(&mut self, width: u32, height: u32) {
        let mut ctx = self.ctx_mx.write().unwrap();
        ctx.res = self.win.size();

        let mut new_real = ctx.window.real().clone();
        new_real.mul_from(ctx.res.y as f32 / ctx.res.x as f32);
        ctx.window.mut_imag().assign(new_real);
        self.win.set_view(
            &*View::from_rect(FloatRect::new(0.0, 0.0, width as f32, height as f32)).unwrap(),
        );
    }

    fn choose_reload_internal(&mut self) {
        let backend;
        {
            let ctx = self.ctx_mx.read().unwrap();
            backend = ctx.backend;
        }

        let start = Instant::now();

        match backend {
            FractalBackend::F32 => self.reload_internal_f32(),
            FractalBackend::F64 => self.reload_internal_f64(),
        }

        self.ctx_mx.write().unwrap().reload_dur = start.elapsed();
    }

    // Drop f32 support next time you need to modify something here
    fn reload_internal_f32(&mut self) {
        let ctx = self.ctx_mx.read().unwrap().clone();
        let center_c32 = Complex::new(ctx.center.real().to_f32(), ctx.center.imag().to_f32());
        let window_c32 = Complex::new(ctx.window.real().to_f32(), ctx.window.imag().to_f32());
        let res_lodiv_u32 = (ctx.res.x / ctx.lodiv, ctx.res.y / ctx.lodiv);
        let res_lodiv_c32 = Complex::new(res_lodiv_u32.0 as f32, res_lodiv_u32.1 as f32);
        let seq_iter = ctx.seq_iter;
        let mut new_image =
            Image::new_solid(res_lodiv_u32.0, res_lodiv_u32.1, Color::MAGENTA).unwrap();
        drop(ctx);

        for x in 0..res_lodiv_u32.0 {
            for y in 0..res_lodiv_u32.1 {
                let c = Complex::map_pixel_value_f32(
                    res_lodiv_c32,
                    center_c32,
                    window_c32,
                    Complex::new(x as f32, y as f32),
                );
                let mut n = c;
                let mut distance = 0.0;
                for _i in 1..seq_iter {
                    n.fsq_add_f32(c);
                    distance = n.re.abs() + n.im.abs();
                    if distance >= 100.0 {
                        break;
                    }
                }
                if distance <= 100.0 {
                    new_image.set_pixel(x, y, Color::BLACK).unwrap()
                } else {
                    new_image
                        .set_pixel(x, y, Complex::distance_gradient_f32(distance))
                        .unwrap();
                }
            }
        }

        // Send image to the GPU
        self.texture
            .load_from_image(&new_image, IntRect::default())
            .unwrap();
    }

    fn reload_internal_f64(&mut self) {
        let ctx = self.ctx_mx.read().unwrap().clone();
        let center_c64 = Complex::new(ctx.center.real().to_f64(), ctx.center.imag().to_f64());
        let window_c64 = Complex::new(ctx.window.real().to_f64(), ctx.window.imag().to_f64());
        let res_lodiv_u32 = (ctx.res.x / ctx.lodiv, ctx.res.y / ctx.lodiv);
        let res_lodiv_c64 = Complex::new(res_lodiv_u32.0 as f64, res_lodiv_u32.1 as f64);
        let seq_iter = ctx.seq_iter;
        let mut new_image =
            Image::new_solid(res_lodiv_u32.0, res_lodiv_u32.1, Color::MAGENTA).unwrap();
        drop(ctx);

        for x in 0..res_lodiv_u32.0 {
            for y in 0..res_lodiv_u32.1 {
                let c = Complex::map_pixel_value_f64(
                    res_lodiv_c64,
                    center_c64,
                    window_c64,
                    Complex::new(x as f64, y as f64),
                );
                let mut n = c;
                let mut distance = 0.0;
                for _i in 1..seq_iter {
                    n.fsq_add_f64(c);
                    distance = n.re.abs() + n.im.abs();
                    if distance >= 100.0 {
                        break;
                    }
                }
                if distance <= 100.0 {
                    new_image.set_pixel(x, y, Color::BLACK).unwrap()
                } else {
                    new_image
                        .set_pixel(x, y, Complex::distance_gradient_f64(distance))
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
