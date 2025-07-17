use std::{
    sync::{
        Arc, RwLock,
        mpsc::{Receiver, Sender},
    },
    time::Instant,
};

use sfml::graphics::IntRect;

use crate::{
    fractal_complex::Complex,
    fractal_engine::{FractalBackend, FractalContext},
    sfml_engine_internal::WorkerNotif,
};

pub struct SfmlEngineWorkerInternal {
    notif_rx: Receiver<WorkerNotif>,
    data_tx: Sender<Vec<u8>>,
    ctx_rwl: Arc<RwLock<FractalContext>>,
    render_rect: IntRect,
}

impl SfmlEngineWorkerInternal {
    pub fn build_and_run(
        notif_rx: Receiver<WorkerNotif>,
        data_tx: Sender<Vec<u8>>,
        ctx_rwl: Arc<RwLock<FractalContext>>,
    ) {
        let mut worker = SfmlEngineWorkerInternal {
            ctx_rwl,
            notif_rx,
            data_tx,
            render_rect: IntRect::default(),
        };

        loop {
            match worker.notif_rx.recv().unwrap() {
                WorkerNotif::Reload => {
                    let data = worker.choose_reload_internal();
                    worker.data_tx.send(data).unwrap();
                }
                WorkerNotif::Shutdown => break,
                WorkerNotif::SetRenderRect(render_rect) => worker.render_rect = render_rect,
            }
        }
    }

    fn choose_reload_internal(&mut self) -> Vec<u8> {
        let backend;
        {
            let ctx = self.ctx_rwl.read().unwrap();
            backend = ctx.backend;
        }
        // TODO I forgot what we must do with the time... But do something.
        let start = Instant::now();

        let data = match backend {
            FractalBackend::F64 => self.reload_internal_f64(),
            _ => panic!("This backend is not implemented for SFML"),
        };

        self.ctx_rwl.write().unwrap().reload_dur = start.elapsed();

        data
    }

    fn reload_internal_f64(&mut self) -> Vec<u8> {
        let ctx = self.ctx_rwl.read().unwrap().clone();
        let center_c64 = Complex::new(ctx.center.real().to_f64(), ctx.center.imag().to_f64());
        let window_c64 = Complex::new(ctx.window.real().to_f64(), ctx.window.imag().to_f64());
        let res_lodiv_u32 = (ctx.res.x / ctx.lodiv, ctx.res.y / ctx.lodiv);
        let res_lodiv_c64 = Complex::new(res_lodiv_u32.0 as f64, res_lodiv_u32.1 as f64);
        let seq_iter = ctx.seq_iter;
        let mut pixels = vec![0; res_lodiv_u32.0 as usize * res_lodiv_u32.1 as usize * 4];
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
                    // new_image.set_pixel(x, y, Color::BLACK).unwrap()
                    let coo = 4 * (res_lodiv_u32.1 * y + x) as usize;
                    pixels[coo..coo + 4].copy_from_slice(&[0; 4]);
                } else {
                    let coo = 4 * (res_lodiv_u32.1 * y + x) as usize;
                    let color = Complex::distance_gradient_f64(distance);
                    pixels[coo..coo + 4].copy_from_slice(&color);
                }
            }
        }

        pixels
    }
}
