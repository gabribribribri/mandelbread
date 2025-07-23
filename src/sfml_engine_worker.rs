use std::{
    sync::{
        Arc, RwLock,
        mpsc::{Receiver, Sender},
    },
    time::Instant,
};

use sfml::graphics::Rect;

use crate::{
    fractal_complex::{self, Complex},
    fractal_engine::{FractalBackend, FractalContext},
    sfml_engine_internal::{WorkerNotif, WorkerResult},
};

pub struct SfmlEngineWorkerInternal {
    notif_rx: Receiver<WorkerNotif>,
    data_tx: Sender<WorkerResult>,
    ctx_rwl: Arc<RwLock<FractalContext>>,
    rrect: Rect<u32>,
}

impl SfmlEngineWorkerInternal {
    pub fn build_and_run(
        notif_rx: Receiver<WorkerNotif>,
        data_tx: Sender<WorkerResult>,
        ctx_rwl: Arc<RwLock<FractalContext>>,
    ) {
        let mut worker = SfmlEngineWorkerInternal {
            ctx_rwl,
            notif_rx,
            data_tx,
            rrect: Rect::<u32>::default(),
        };

        worker.run()
    }

    fn run(&mut self) {
        loop {
            match self.notif_rx.recv().unwrap() {
                WorkerNotif::Reload => {
                    let result = self.choose_compute_backend();
                    self.data_tx.send(result).unwrap();
                }
                WorkerNotif::Shutdown => break,
                WorkerNotif::SetRenderRect(render_rect) => self.rrect = render_rect,
            }
        }
    }

    fn choose_compute_backend(&mut self) -> WorkerResult {
        let backend = self.ctx_rwl.read().unwrap().backend;
        match backend {
            FractalBackend::F64 => self.compute_image_f64(),
            FractalBackend::Rug => self.compute_image_rug(),
        }
    }

    fn compute_image_f64(&mut self) -> WorkerResult {
        let start = Instant::now();

        let ctx = self.ctx_rwl.read().unwrap().clone();
        let center_c64 = Complex::new(ctx.center.real().to_f64(), ctx.center.imag().to_f64());
        let window_c64 = Complex::new(ctx.window.real().to_f64(), ctx.window.imag().to_f64());
        let res_lodiv_c64 = Complex::new(
            (ctx.res.x / ctx.lodiv) as f64,
            (ctx.res.y / ctx.lodiv) as f64,
        );
        let seq_iter = ctx.seq_iter;
        let converge_distance = ctx.converge_distance;
        drop(ctx);

        let mut pixels = Vec::with_capacity((self.rrect.width * self.rrect.height * 4) as usize);

        for y in 0..self.rrect.height {
            for x in 0..self.rrect.width {
                let c = Complex::map_pixel_value_f64(
                    res_lodiv_c64,
                    center_c64,
                    window_c64,
                    Complex::new((self.rrect.left + x) as f64, (self.rrect.top + y) as f64),
                );
                let mut n = c;
                let mut distance = 0.0;
                let mut iter = 0;
                while iter < seq_iter && distance <= converge_distance {
                    n.f_sq_add_f64(c);
                    distance = n.abs_sum_f64();
                    iter += 1;
                }
                if distance <= converge_distance {
                    pixels.extend_from_slice(&[0, 0, 0, 255]);
                } else {
                    let color = fractal_complex::iter_gradient(iter, seq_iter);
                    pixels.extend_from_slice(&color);
                }
            }
        }

        WorkerResult {
            pixels,
            rrect: self.rrect,
            reload_dur: start.elapsed(),
        }
    }

    fn compute_image_rug(&mut self) -> WorkerResult {
        let start = Instant::now();

        let ctx = self.ctx_rwl.read().unwrap().clone();
        let center = ctx.center.clone();
        let window = ctx.window.clone();
        let res = ctx.res / ctx.lodiv;
        let seq_iter = ctx.seq_iter;
        let converge_distance = ctx.converge_distance;
        drop(ctx);

        let mut pixels = Vec::with_capacity((self.rrect.width * self.rrect.height * 4) as usize);

        for y in 0..self.rrect.height {
            for x in 0..self.rrect.width {
                let c = fractal_complex::map_pixel_value_rug(
                    res,
                    &center,
                    &window,
                    ((self.rrect.left + x) as i32, (self.rrect.top + y) as i32),
                );
                let mut n = c.clone();
                let mut distance = 0.0;
                let mut iter = 0;
                while iter < seq_iter && distance <= converge_distance {
                    n = fractal_complex::f_sq_add_rug(&n, &c);
                    distance = fractal_complex::abs_sum_rug(&n);
                    iter += 1;
                }
                if distance <= converge_distance {
                    pixels.extend_from_slice(&[0, 0, 0, 255]);
                } else {
                    let color = fractal_complex::iter_gradient(iter, seq_iter);
                    pixels.extend_from_slice(&color);
                }
            }
        }

        WorkerResult {
            pixels,
            rrect: self.rrect,
            reload_dur: start.elapsed(),
        }
    }
}
