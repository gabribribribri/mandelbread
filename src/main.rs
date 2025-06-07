mod utils;
use egui::load::Result;
use sfml::{
    graphics::{Color, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture},
    window::{ContextSettings, Event, Style},
};
use std::time::Instant;

use mandelbread::utils::{distance_gradient, map_range, tuple_to_sfml_color};

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;
const REAL_RANGE_START: f32 = -2.0;
const REAL_RANGE_END: f32 = 2.0;
const IMAG_RANGE_START: f32 = 1.5;
const IMAG_RANGE_END: f32 = -1.5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut window = RenderWindow::new(
        (800, 600),
        "Mandelbread",
        Style::DEFAULT,
        &ContextSettings::default(),
    )?;

    window.set_framerate_limit(60);

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                _ => {}
            }
        }

        window.clear(Color::rgb(0, 64, 0));

        let mut image = Image::new_solid(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, Color::BLACK)?;
        let now = Instant::now();

        for x in 0..WINDOW_WIDTH {
            for y in 0..WINDOW_HEIGHT {
                let c_re = map_range(0, WINDOW_WIDTH, REAL_RANGE_START, REAL_RANGE_END, x);

                let c_im = map_range(0, WINDOW_HEIGHT, IMAG_RANGE_START, IMAG_RANGE_END, y);

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

        let elapsed_time = now.elapsed();

        println!("Rendering took {:#?}", elapsed_time);

        let texture = Texture::from_image(&image, IntRect::new(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT))?;
        let mut sprite = Sprite::new();
        sprite.set_texture(&texture, true);
        window.draw(&sprite);
        window.display();
    }
    Ok(())
}
