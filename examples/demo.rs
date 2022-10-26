use crossterm::{
    event::{self, KeyCode},
    style::Color,
    terminal, Result,
};
use winterm::Window;

struct Player {
    x: u16,
    y: u16,
}

fn main() -> Result<()> {
    println!("Use arrows or WASD to move.");
    println!("[Press any key to continue]");
    terminal::enable_raw_mode()?;
    event::read()?;

    let mut window = Window::new(25, 25)?;
    let background_color = Color::Rgb {
        r: 100,
        g: 100,
        b: 100,
    };
    for y in 0..window.height() {
        for x in 0..window.width() {
            window.set_pixel(y, x, background_color);
        }
    }
    let mut player = Player { x: 12, y: 12 };
    loop {
        window.poll_events()?;
        if window.get_key(KeyCode::Esc) {
            break;
        }
        window.set_pixel(player.y, player.x, background_color);
        if player.y > 0 && (window.get_key(KeyCode::Up) || window.get_key(KeyCode::Char('w'))) {
            player.y -= 1;
        }
        if player.y < window.height() - 1
            && (window.get_key(KeyCode::Down) || window.get_key(KeyCode::Char('s')))
        {
            player.y += 1;
        }
        if player.x > 0 && (window.get_key(KeyCode::Left) || window.get_key(KeyCode::Char('a'))) {
            player.x -= 1;
        }
        if player.x < window.width() - 1
            && (window.get_key(KeyCode::Right) || window.get_key(KeyCode::Char('d')))
        {
            player.x += 1;
        }
        window.set_pixel(player.y, player.x, Color::Red);
        window.redraw()?;
    }
    Ok(())
}
