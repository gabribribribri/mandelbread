use std::time::Instant;

use sfml::{
    cpp::FBox,
    graphics::{Color, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture},
    window::{ContextSettings, Event, Style},
};

use crate::{
    complex::Complex,
    fractal_engine::{FractalContext, FractalEngine},
    utils::{distance_gradient, map_range, tuple_to_sfml_color},
};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

pub struct SfmlEngine {
    win: FBox<RenderWindow>,
    ctx: FractalContext<f32>,
    texture: FBox<Texture>,
}

impl SfmlEngine {
    pub fn new() -> Result<SfmlEngine, Box<dyn std::error::Error>> {
        let mut win = RenderWindow::new(
            (WINDOW_WIDTH, WINDOW_HEIGHT),
            "Mandelbread",
            Style::DEFAULT,
            &ContextSettings::default(),
        )?;

        win.set_framerate_limit(60);

        let ctx = FractalContext {
            resolution: (WINDOW_WIDTH, WINDOW_HEIGHT),
            start: Complex::new(-2.0, 1.5),
            end: Complex::new(2.0, -1.5),
        };

        let image = Image::new_solid(WINDOW_WIDTH, WINDOW_HEIGHT, Color::rgb(32, 0, 0))?;
        let texture = Texture::from_image(&image, IntRect::default())?;

        Ok(Self { win, ctx, texture })
    }
}

impl FractalEngine for SfmlEngine {
    fn render(&mut self) {
        while let Some(event) = self.win.poll_event() {
            match event {
                Event::Closed => self.win.close(),
                _ => {}
            }
        }
        let sprite = Sprite::with_texture(&self.texture);
        self.win.clear(Color::rgb(0, 64, 0));
        self.win.draw(&sprite);
        self.win.display();
    }

    fn reload(&mut self) -> Result<std::time::Duration, Box<dyn std::error::Error>> {
        let now = Instant::now();

        let mut image = Image::new_solid(WINDOW_WIDTH, WINDOW_HEIGHT, Color::rgb(32, 0, 0))?;

        for x in 0..WINDOW_WIDTH as i32 {
            for y in 0..WINDOW_HEIGHT as i32 {
                let c_re = map_range(
                    0,
                    WINDOW_WIDTH as i32,
                    self.ctx.start.re,
                    self.ctx.end.re,
                    x,
                );

                let c_im = map_range(
                    0,
                    WINDOW_HEIGHT as i32,
                    self.ctx.start.im,
                    self.ctx.end.im,
                    y,
                );

                let mut n_re = c_re;
                let mut n_im = c_im;

                let mut distance = 0.0;
                for _ in 1..=99 {
                    let i_re = n_re * n_re - n_im * n_im + c_re;
                    let i_im = 2.0 * n_re * n_im + c_im;
                    n_re = i_re;
                    n_im = i_im;
                    distance = n_re.abs() + n_im.abs();
                    if distance >= 100.0 {
                        break;
                    }
                }
                if distance <= 100.0 {
                    image.set_pixel(x as u32, y as u32, Color::BLACK)?;
                } else {
                    image.set_pixel(
                        x as u32,
                        y as u32,
                        tuple_to_sfml_color(distance_gradient::<100, 1000>(distance)),
                    )?;
                }
            }
        }

        self.texture.load_from_image(&image, IntRect::default())?;
        return Ok(now.elapsed());
    }

    fn deinitialize(&mut self) {
        self.win.close();
    }
}
