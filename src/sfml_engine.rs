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
        FRCTL_CTX_CMPLX_PREC, FractalBackend, FractalContext, FractalEngine, FractalNotif, lodiv,
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

    fn set_rug_prec(&mut self, prec: u32) {
        let mut ctx = self.ctx_rwl.write().unwrap();
        ctx.window.set_prec(prec);
        ctx.center.set_prec(prec);
    }
}

impl FractalEngine for SfmlEngine {
    fn commence(&self) {
        self.ctx_rwl.write().unwrap().engine_enabled = true;
        self.notif_tx
            .send(FractalNotif::Commence)
            .expect("Cannot start the internal engine")
    }

    fn shutdown(&mut self) {
        self.ctx_rwl.write().unwrap().engine_enabled = false;
        self.notif_tx
            .send(FractalNotif::Shutdown)
            .expect("Cannot shutdown the internal engine")
    }

    fn reset_window(&mut self) {
        let mut ctx = self.ctx_rwl.write().unwrap();
        ctx.window = rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, (2.66, 2.0));
        let mut new_real = ctx.window.real().clone();
        new_real.mul_from(ctx.res.y as f32 / ctx.res.x as f32);
        ctx.window.mut_imag().assign(new_real);
        drop(ctx);
    }

    fn reset_view(&mut self) {
        let mut ctx = self.ctx_rwl.write().unwrap();
        ctx.center = rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, -0.5);
        ctx.window = rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, (2.66, 2.0));
        let mut new_real = ctx.window.real().clone();
        new_real.mul_from(ctx.res.y as f32 / ctx.res.x as f32);
        ctx.window.mut_imag().assign(new_real);
        drop(ctx);
    }

    fn reload(&mut self) {
        self.notif_tx
            .send(FractalNotif::Reload)
            .expect("Cannot reload the internal engine")
    }

    fn move_window(&mut self, trsln: Complex<f32>) {
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

    fn zoom_view(&mut self, zoom: f32) {
        self.ctx_rwl.write().unwrap().window *= zoom;
        self.reload()
    }

    fn set_lodiv(&mut self, lodiv: u32) {
        {
            let mut ctx = self.ctx_rwl.write().unwrap();
            ctx.lodiv = lodiv;
            ctx.has_resized = true;
        }
        self.reload()
    }

    fn set_backend(&mut self, backend: FractalBackend) {
        self.ctx_rwl.write().unwrap().backend = backend;
        self.reload()
    }

    fn set_seq_iter(&mut self, seq_iter: u32) {
        self.ctx_rwl.write().unwrap().seq_iter = seq_iter;
    }

    fn set_workers(&mut self, workers: usize) {
        self.ctx_rwl.write().unwrap().worker_count = workers;
    }

    fn set_converge_distance(&mut self, converge_distance: f64) {
        self.ctx_rwl.write().unwrap().converge_distance = converge_distance;
    }

    fn gui_central_panel(&mut self, ui: &mut Ui) {
        let mut ctx;
        {
            ctx = self.ctx_rwl.read().unwrap().clone();
        } // drops the mic
        let mut rug_prec = ctx.window.prec().0;

        ui.heading("SFML Engine");
        ui.separator();

        if ui.checkbox(&mut ctx.engine_enabled, "Enabled").clicked() {
            match ctx.engine_enabled {
                true => self.commence(),
                false => self.shutdown(),
            }
        }

        ui.add_space(7.0);

        if ui
            .radio_value(
                &mut ctx.backend,
                FractalBackend::F64,
                "64-bit floating point",
            )
            .clicked()
        {
            self.set_backend(FractalBackend::F64);
        }
        if ui
            .radio_value(
                &mut ctx.backend,
                FractalBackend::Rug,
                "Rug arbitrary precision",
            )
            .clicked()
        {
            self.set_backend(FractalBackend::Rug);
        }

        ui.add_space(7.0);

        ui.horizontal(|ui| {
            ui.label("Precision : ");
            let drag_value = ui.add(egui::DragValue::new(&mut rug_prec).range(64..=u32::MAX));
            if drag_value.changed() {
                self.set_rug_prec(rug_prec);
            }
            if drag_value.drag_stopped() {
                self.reload();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Sequence Iterations : ");
            let drag_value = ui.add(egui::DragValue::new(&mut ctx.seq_iter));
            if drag_value.changed() {
                self.set_seq_iter(ctx.seq_iter);
            }
            if drag_value.drag_stopped() {
                self.reload();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Converge Distance : ");
            let drag_value = ui.add(egui::DragValue::new(&mut ctx.converge_distance));
            if drag_value.changed() {
                self.set_converge_distance(ctx.converge_distance);
            }
            if drag_value.drag_stopped() {
                self.reload();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Workers : ");
            if ui.button(" - ").clicked() && ctx.worker_count >= 1 {
                self.set_workers(ctx.worker_count - 1);
            }
            if ui
                .add(
                    egui::DragValue::new(&mut ctx.worker_count)
                        .range(1..=256)
                        .speed(0.1),
                )
                .changed()
            {
                self.set_workers(ctx.worker_count);
            }
            if ui.button(" + ").clicked() && ctx.worker_count <= 256 {
                self.set_workers(ctx.worker_count + 1);
            }
        });

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
                self.set_lodiv(ctx.lodiv);
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::HIGHEST, "HIGHEST")
                .clicked()
            {
                self.set_lodiv(lodiv::HIGHEST);
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::FAST, "FAST")
                .clicked()
            {
                self.set_lodiv(lodiv::FAST);
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::FASTER, "FASTER")
                .clicked()
            {
                self.set_lodiv(lodiv::FASTER);
            }
            if ui
                .selectable_label(ctx.lodiv == lodiv::FASTEST, "FASTEST")
                .clicked()
            {
                self.set_lodiv(lodiv::FASTEST);
            }
        });

        ui.add_space(7.0);

        ui.horizontal(|ui| {
            ui.label("Movements :");
            if ui.button("Left").clicked() {
                self.move_window(Complex::new(-0.2, 0.0));
            }
            if ui.button("Down").clicked() {
                self.move_window(Complex::new(0.0, -0.2));
            }
            if ui.button("Up").clicked() {
                self.move_window(Complex::new(0.0, 0.2));
            }
            if ui.button("Right").clicked() {
                self.move_window(Complex::new(0.2, 0.0));
            }
        });

        ui.horizontal(|ui| {
            ui.label("Zoom : ");
            if ui.button("Outside").clicked() {
                self.zoom_view(2.0);
            }
            if ui.button("Inside").clicked() {
                self.zoom_view(0.5);
            }
        });

        ui.add_space(7.0);

        if ui
            .button(RichText::new("RELOAD").size(12.0).extra_letter_spacing(3.0))
            .clicked()
        {
            self.reload()
        }

        ui.add_space(7.0);

        if ui.button("RESET WINDOW").clicked() {
            self.reset_window();
            self.reload();
        }

        if ui.button("RESET VIEW").clicked() {
            self.reset_view();
            self.reload();
        }
    }

    fn gui_bottom_panel(&mut self, ui: &mut Ui) {
        let ctx = self.ctx_rwl.read().unwrap();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Reload Time :").strong());
            ui.label(format!("{:?}", ctx.reload_durs.iter().max().unwrap()));
        });

        ui.collapsing("Worker Specific :", |ui| {
            for (id, dur) in ctx.reload_durs.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Worker {} :", id)).strong());
                    ui.label(format!("{:?}", dur));
                });
            }
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Resolution :").strong());
            ui.label(format!(
                "{}x{}",
                ctx.res.x / ctx.lodiv,
                ctx.res.y / ctx.lodiv
            ));
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
