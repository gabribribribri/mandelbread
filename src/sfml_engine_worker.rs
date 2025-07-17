use std::sync::{
    Arc, RwLock,
    mpsc::{Receiver, Sender},
};

use sfml::graphics::Rect;

use crate::{
    fractal_complex::Complex,
    fractal_engine::{FractalBackend, FractalContext},
    sfml_engine_internal::WorkerNotif,
};

pub struct SfmlEngineWorkerInternal {
    notif_rx: Receiver<WorkerNotif>,
    data_tx: Sender<(Vec<u8>, Rect<u32>)>,
    ctx_rwl: Arc<RwLock<FractalContext>>,
    render_rect: Rect<u32>,
}

impl SfmlEngineWorkerInternal {
    pub fn build_and_run(
        notif_rx: Receiver<WorkerNotif>,
        data_tx: Sender<(Vec<u8>, Rect<u32>)>,
        ctx_rwl: Arc<RwLock<FractalContext>>,
    ) {
        let mut worker = SfmlEngineWorkerInternal {
            ctx_rwl,
            notif_rx,
            data_tx,
            render_rect: Rect::<u32>::default(),
        };

        worker.run()
    }

    fn run(&mut self) {
        loop {
            match self.notif_rx.recv().unwrap() {
                WorkerNotif::Reload => {
                    let pixels = self.choose_reload_internal();
                    self.data_tx.send((pixels, self.render_rect)).unwrap();
                }
                WorkerNotif::Shutdown => break,
                WorkerNotif::SetRenderRect(render_rect) => self.render_rect = render_rect,
            }
        }
    }

    fn choose_reload_internal(&mut self) -> Vec<u8> {
        let backend = self.ctx_rwl.read().unwrap().backend;
        match backend {
            FractalBackend::F64 => self.reload_internal_f64(),
        }
    }

    fn reload_internal_f64(&mut self) -> Vec<u8> {
        let ctx = self.ctx_rwl.read().unwrap().clone();
        let center_c64 = Complex::new(ctx.center.real().to_f64(), ctx.center.imag().to_f64());
        let window_c64 = Complex::new(ctx.window.real().to_f64(), ctx.window.imag().to_f64());
        let res_lodiv_c64 = Complex::new(
            (ctx.res.x / ctx.lodiv) as f64,
            (ctx.res.y / ctx.lodiv) as f64,
        );
        let seq_iter = ctx.seq_iter;
        let mut pixels =
            vec![0; self.render_rect.width as usize * self.render_rect.height as usize * 4];
        drop(ctx);

        for x in 0..self.render_rect.width {
            for y in 0..self.render_rect.height {
                let c = Complex::map_pixel_value_f64(
                    res_lodiv_c64,
                    center_c64,
                    window_c64,
                    Complex::new(
                        (self.render_rect.left + x) as f64,
                        (self.render_rect.top + y) as f64,
                    ),
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
                    // new_image.set_pixel(x, y, Color::BLACK).unwrap()
                    let coo = 4 * (self.render_rect.width * y + x) as usize;
                    pixels[coo..coo + 4].copy_from_slice(&[0, 0, 0, 255]);
                } else {
                    let coo = 4 * (self.render_rect.width * y + x) as usize;
                    let color = Complex::distance_gradient_f64(distance);
                    pixels[coo..coo + 4].copy_from_slice(&color);
                }
            }
        }

        pixels
    }
}
