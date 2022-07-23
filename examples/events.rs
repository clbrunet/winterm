use std::{io::stdout, time::Duration, thread};

use crossterm::{Result, execute, terminal, cursor, event::{KeyCode, KeyModifiers}};
use winterm::Window;

fn main() -> Result<()> {
    let mut window = Window::new(4, 4)?;
    let _ = execute!(
        stdout(),
        terminal::LeaveAlternateScreen,
        terminal::EnableLineWrap,
        cursor::Show
    );
    for _ in 0..10 {
        window.poll_events()?;
        eprintln!("w {}\r", window.get_key(KeyCode::Char('w')));
        eprintln!("shift {}\r", window.get_modifiers(KeyModifiers::SHIFT));
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
