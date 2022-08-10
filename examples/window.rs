use crossterm::{event::KeyCode, style::Color, Result};
use nalgebra::Vector3;
use winterm::Window;

fn set_line_gradation(window: &mut Window, y: u16, color: &Vector3<u8>) {
    for x in 0..window.width() {
        let multiplier = (x + 1) as f64 / window.width() as f64;
        window.set_pixel(
            y,
            x,
            Color::Rgb {
                r: (color.x as f64 * multiplier) as u8,
                g: (color.y as f64 * multiplier) as u8,
                b: (color.z as f64 * multiplier) as u8,
            },
        );
    }
}

fn main() -> Result<()> {
    let mut window = Window::new(9, 80)?;
    let colors = [
        Vector3::new(255, 255, 255),
        Vector3::new(255, 0, 0),
        Vector3::new(0, 255, 0),
        Vector3::new(0, 0, 255),
        Vector3::new(255, 255, 0),
        Vector3::new(0, 255, 255),
        Vector3::new(255, 0, 255),
        Vector3::new(255, 255, 255),
        Vector3::new(255, 255, 255),
    ];
    for y in 0..window.height() {
        set_line_gradation(&mut window, y, &colors[y as usize]);
    }
    loop {
        window.poll_events()?;
        if window.get_key(KeyCode::Esc) {
            break;
        }
        window.redraw()?;
    }
    Ok(())
}
