use std::time::Instant;

use eframe::WindowAttributes;
use egui::{Rect, Vec2};
use sfml::{
    cpp::FBox,
    graphics::{
        Color, FloatRect, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture, View,
    },
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
            res: (WINDOW_WIDTH, WINDOW_HEIGHT),
            start: Complex::new(-2.0, 1.5),
            end: Complex::new(1.0, -1.5),
        };

        let image = Image::new_solid(WINDOW_WIDTH, WINDOW_HEIGHT, Color::rgb(32, 0, 0))?;
        let texture = Texture::from_image(&image, IntRect::default())?;

        Ok(Self { win, ctx, texture })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ctx.res.0 = width;
        self.ctx.res.1 = height;

        self.win.set_view(
            &*View::from_rect(FloatRect::new(0.0, 0.0, width as f32, height as f32)).unwrap(),
        );
    }
}

impl FractalEngine for SfmlEngine {
    fn render(&mut self) {
        while let Some(event) = self.win.poll_event() {
            match event {
                Event::Closed => self.win.close(),
                Event::Resized { width, height } => self.resize(width, height),
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

        let mut image = Image::new_solid(self.ctx.res.0, self.ctx.res.1, Color::rgb(32, 0, 0))?;

        for x in 0..self.ctx.res.0 {
            for y in 0..self.ctx.res.1 {
                let c = Complex::map_between(self.ctx.res, self.ctx.start, self.ctx.end, (x, y));
                let mut n = c;
                let mut distance = 0.0;
                for _ in 1..=100 {
                    n.sq_add(c);
                    distance = n.re.abs() + n.im.abs();
                    if distance >= 100.0 {
                        break;
                    }
                }
                if distance <= 100.0 {
                    image.set_pixel(x as u32, y as u32, Color::BLACK)?;
                } else {
                    // TODO Make this a variable shader
                    image.set_pixel(x, y, distance_gradient::<100, 1500>(distance).into())?;
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
