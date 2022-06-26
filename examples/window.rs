use std::{thread, time::Duration};

use crossterm::{style::Color, Result};
use winterm::Window;

fn main() -> Result<()> {
    let mut window = Window::new(4, 4)?;
    let vec = (1..=4)
        .rev()
        .map(|i| (i as f32 / 4. * u8::MAX as f32) as u8)
        .collect::<Vec<u8>>();
    for (x, color_brightness) in vec.iter().enumerate() {
        window.set_pixel(
            x.try_into().unwrap(),
            0,
            Color::Rgb {
                r: *color_brightness,
                g: 0,
                b: 0,
            },
        );
    }
    for (x, color_brightness) in vec.iter().enumerate() {
        window.set_pixel(
            x.try_into().unwrap(),
            1,
            Color::Rgb {
                r: 0,
                g: *color_brightness,
                b: 0,
            },
        );
    }
    for (x, color_brightness) in vec.iter().enumerate() {
        window.set_pixel(
            x.try_into().unwrap(),
            2,
            Color::Rgb {
                r: 0,
                g: 0,
                b: *color_brightness,
            },
        );
    }
    for (x, color_brightness) in vec.iter().enumerate() {
        window.set_pixel(
            x.try_into().unwrap(),
            3,
            Color::Rgb {
                r: *color_brightness,
                g: *color_brightness,
                b: *color_brightness,
            },
        );
    }
    window.draw()?;
    thread::sleep(Duration::from_secs(3));
    Ok(())
}
