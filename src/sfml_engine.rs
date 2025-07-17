use std::{
    sync::{
        Arc, RwLock,
        mpsc::{self, Sender},
    },
    thread,
};

use egui::{RichText, Ui};
use rug::{
    Assign,
    ops::{AddFrom, MulFrom},
};

use crate::{
    fractal_complex::Complex,
    fractal_engine::{
        FRCTL_CTX_CMPLX_PREC, FractalBackend, FractalContext, FractalEngine, FractalEngineError,
        FractalNotif, lodiv,
    },
    sfml_engine_internal::SfmlEngineInternal,
};

pub struct SfmlEngine {
    notif_tx: Sender<FractalNotif>,
    ctx_rwl: Arc<RwLock<FractalContext>>,
}

impl SfmlEngine {
    pub fn new() -> SfmlEngine {
        let (ext_tx, in_rx) = mpsc::channel::<FractalNotif>();

        let ctx_rwl = Arc::<RwLock<FractalContext>>::default();

        let ctx_rwl_clone = Arc::clone(&ctx_rwl);

        thread::Builder::new()
            .name("SFML Engine".to_string())
            .spawn(|| -> ! { SfmlEngineInternal::run(ctx_rwl_clone, in_rx) })
            .unwrap();

        ext_tx.send(FractalNotif::Commence).unwrap();

        SfmlEngine {
            notif_tx: ext_tx,
            ctx_rwl,
        }
    }
}

impl FractalEngine for SfmlEngine {
    fn commence(&self) -> Result<(), FractalEngineError> {
        self.ctx_rwl.write().unwrap().engine_enabled = true;
        match self.notif_tx.send(FractalNotif::Commence) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Cannot start the internal engine : {}", e);
                Err(FractalEngineError::SendError)
            }
        }
    }

    fn shutdown(&mut self) -> Result<(), FractalEngineError> {
        self.ctx_rwl.write().unwrap().engine_enabled = false;
        match self.notif_tx.send(FractalNotif::Shutdown) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Cannot shutdown the internal engine : {}", e);
                Err(FractalEngineError::SendError)
            }
        }
    }

    fn reset_view(&mut self) -> Result<(), FractalEngineError> {
        let mut ctx = self.ctx_rwl.write().unwrap();
        ctx.center = rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, -0.5);
        ctx.window = rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, (2.66, 2.0));
        let mut new_real = ctx.window.real().clone();
        new_real.mul_from(ctx.res.y as f32 / ctx.res.x as f32);
        ctx.window.mut_imag().assign(new_real);
        drop(ctx);

        self.reload()
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
        let mut ctx = self.ctx_rwl.write().unwrap();

        let mut real_offset = ctx.window.real().clone();
        real_offset.mul_from(0.5 * trsln.re);
        let mut imag_offset = ctx.window.imag().clone();
        imag_offset.mul_from(-0.5 * trsln.im); // why minus ??
        ctx.center.add_from(real_offset);
        ctx.center.mut_imag().add_from(imag_offset);

        drop(ctx);
        self.reload()
    }

    fn zoom_view(&mut self, zoom: f32) -> Result<(), FractalEngineError> {
        {
            self.ctx_rwl.write().unwrap().window *= zoom;
        }
        self.reload()
    }

    fn set_lodiv(&mut self, lodiv: u32) -> Result<(), FractalEngineError> {
        {
            self.ctx_rwl.write().unwrap().lodiv = lodiv;
        }
        self.reload()
    }

    fn set_backend(&mut self, backend: FractalBackend) -> Result<(), FractalEngineError> {
        {
            self.ctx_rwl.write().unwrap().backend = backend;
        }
        self.reload()
    }

    fn set_seq_iter(&mut self, seq_iter: u32) -> Result<(), FractalEngineError> {
        // TODO it would be cool if I could reload here...
        // nevermind, although it is squechy...
        {
            self.ctx_rwl.write().unwrap().seq_iter = seq_iter;
        }
        self.reload()
    }

    fn gui_central_panel(&mut self, ui: &mut Ui) {
        let mut ctx;
        {
            ctx = self.ctx_rwl.read().unwrap().clone();
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
            .radio_value(&mut ctx.backend, FractalBackend::F64, "64-bit float")
            .clicked()
        {
            self.set_backend(FractalBackend::F64).unwrap();
        }

        ui.horizontal(|ui| {
            ui.label("Sequence Iterations : ");
            if ui.add(egui::DragValue::new(&mut ctx.seq_iter)).changed() {
                self.set_seq_iter(ctx.seq_iter).unwrap();
            }
        });

        ui.horizontal(|ui| {});

        ui.horizontal(|ui| {
            ui.label("Quality : ");
            if ui
                .add(
                    egui::DragValue::new(&mut ctx.lodiv)
                        .range(1..=25)
                        .speed(0.04),
                )
                .changed()
            {
                self.set_lodiv(ctx.lodiv).unwrap();
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::HIGHEST, "HIGHEST")
                .clicked()
            {
                self.set_lodiv(lodiv::HIGHEST).unwrap();
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::FAST, "FAST")
                .clicked()
            {
                self.set_lodiv(lodiv::FAST).unwrap();
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::FASTER, "FASTER")
                .clicked()
            {
                self.set_lodiv(lodiv::FASTER).unwrap();
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::FASTEST, "FASTEST")
                .clicked()
            {
                self.set_lodiv(lodiv::FASTEST).unwrap();
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

        if ui
            .button(RichText::new("RELOAD").size(12.0).extra_letter_spacing(3.0))
            .clicked()
        {
            self.reload().unwrap()
        }

        if ui.button("RESET VIEW").clicked() {
            self.reset_view().unwrap();
        }
    }

    fn gui_bottom_panel(&mut self, ui: &mut Ui) {
        let ctx = self.ctx_rwl.read().unwrap();

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
