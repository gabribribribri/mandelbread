use sfml::graphics::Color;


#[derive(Clone, Copy)]
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}

impl<T> Complex<T> {
    pub fn new(re: T, im: T) -> Self {
        Self {
            re,
            im 
        }
    }
}

impl Complex<f32> {
    pub fn map_pixel_value_f32(res: Complex<f32>, center: Complex<f32>, window: Complex<f32>, value: Complex<f32>) -> Complex<f32> {
        Self::new(
            center.re - (window.re/2.0) + (value.re / res.re) * window.re,
            center.im - (window.im/2.0) + (value.im / res.im) * window.im,
        )
    }

    pub fn fsq_add_f32(&mut self, c: Complex<f32>) {
        (self.re, self.im) = (
            self.re * self.re - self.im * self.im + c.re,
            2.0 * self.re * self.im + c.im
        );
    }

    pub fn distance_gradient_f32(distance: f32) -> Color {
        const START: f32 = 100.0;
        const END: f32 = 1500.0;
        const HALF: f32 = (END - START)/2.0;

        let clamped_value = distance.clamp(START, END);
        let (red, green, blue);

        if clamped_value <= HALF {
            let t = (clamped_value - START) / (HALF - START);
            red = (1.0 - t) * 255.0;
            green = t * 255.0;
            blue = 0.0;
        } else {
            let t = (clamped_value - HALF) / (HALF - START);
            red = 0.0;
            green = (1.0 - t) * 255.0;
            blue = t * 255.0;
        }

        Color::rgb(red as u8, green as u8, blue as u8)
        
    }
}

impl Complex<f64> {
    pub fn map_pixel_value_f64(res: Complex<f64>, center: Complex<f64>, window: Complex<f64>, value: Complex<f64>) -> Complex<f64> {
        Self::new(
            center.re - (window.re/2.0) + (value.re / res.re) * window.re,
            center.im - (window.im/2.0) + (value.im / res.im) * window.im,
        )
    }

    pub fn fsq_add_f64(&mut self, c: Complex<f64>) {
        (self.re, self.im) = (
            self.re * self.re - self.im * self.im + c.re,
            2.0 * self.re * self.im + c.im
        );
    }

    pub fn distance_gradient_f64(distance: f64) -> Color {
        const START: f64 = 100.0;
        const END: f64 = 1500.0;
        const HALF: f64 = (END - START)/2.0;

        let clamped_value = distance.clamp(START, END);
        let (red, green, blue);

        if clamped_value <= HALF {
            let t = (clamped_value - START) / (HALF - START);
            red = (1.0 - t) * 255.0;
            green = t * 255.0;
            blue = 0.0;
        } else {
            let t = (clamped_value - HALF) / (HALF - START);
            red = 0.0;
            green = (1.0 - t) * 255.0;
            blue = t * 255.0;
        }

        Color::rgb(red as u8, green as u8, blue as u8)
        
    }
}
 
