use std::{
    sync::mpsc::{self, Receiver, SendError, Sender},
    thread::{self, spawn},
};

use egui::Ui;
use rug::Complex;
use sfml::{
    cpp::FBox,
    graphics::{Color, Image, IntRect, RenderWindow, Texture},
    window::{ContextSettings, Style},
};

use crate::fractal_engine::{FractalContext, FractalEngine, FractalEngineError, FractalNotif};

pub struct SfmlEngine {
    notif_tx: Sender<FractalNotif>,
    notif_rx: Receiver<FractalNotif>,
    fractal_ctx: FractalContext,
}

pub struct SfmlEngineInternal<'a> {
    notif_tx: &'a Sender<FractalNotif>,
    notif_rx: &'a Receiver<FractalNotif>,
    fractal_ctx: FractalContext,

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
            fractal_ctx,
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

            let image = Image::new_solid(ctx.res.0, ctx.res.1, Color::rgb(32, 0, 0)).unwrap();
            let texture = Texture::from_image(&image, IntRect::default()).unwrap();

            let mut internal_engine = SfmlEngineInternal {
                notif_tx: &tx,
                notif_rx: &rx,
                fractal_ctx: ctx,
                win,
                texture,
            };

            internal_engine.run_until_end();
        }
    }
}

impl FractalEngine for SfmlEngine {
    fn commence(&self) -> Result<(), FractalEngineError> {
        match self
            .notif_tx
            .send(FractalNotif::Commence(self.fractal_ctx.clone()))
        {
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

    fn move_view(&mut self, translation: Complex) -> Result<(), FractalEngineError> {
        match self.notif_tx.send(FractalNotif::Move(translation)) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Cannot move the view : {}", e);
                Err(FractalEngineError::SendError)
            }
        }
    }

    fn zoom_view(&mut self, zoom: f32) -> Result<(), FractalEngineError> {
        match self.notif_tx.send(FractalNotif::Zoom(zoom)) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Cannot zoom : {}", e);
                Err(FractalEngineError::SendError)
            }
        }
    }

    fn gui_central_panel(&mut self, ui: &mut Ui) {}

    fn gui_bottom_panel(&mut self, ui: &mut Ui) {}
}

impl<'a> SfmlEngineInternal<'a> {
    fn run_until_end(self) {}
}
