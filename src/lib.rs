use std::io::{stdout, Write};

use crossterm::cursor::{self, MoveDown, MoveLeft, MoveTo};
use crossterm::style::{Color, Colors, Print, SetBackgroundColor, SetColors, SetForegroundColor};
use crossterm::{execute, queue, terminal, Result};

#[derive(Debug)]
struct Dimension {
    width: u16,
    height: u16,
}

#[derive(Debug)]
pub struct Window {
    dimension: Dimension,
    origin: (u16, u16),
    pixels: Box<[Box<[Color]>]>,
}

impl Window {
    pub fn new(width: u16, height: u16) -> Result<Window> {
        let (columns, rows) = terminal::size()?;
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            terminal::DisableLineWrap,
            cursor::Hide
        )?;
        Ok(Window {
            dimension: Dimension { width, height },
            origin: (
                (columns as f32 / 2. - width as f32 / 2.) as u16,
                (rows as f32 / 2. - height as f32 / 4.) as u16,
            ),
            pixels: vec![vec![Color::Black; width.into()].into_boxed_slice(); height.into()]
                .into_boxed_slice(),
        })
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, color: Color) {
        debug_assert_ne!(color, Color::Reset, "Cannot set pixel to Color::Reset");
        self.pixels[y as usize][x as usize] = color;
    }

    pub fn render(&self) -> Result<()> {
        queue!(stdout(), MoveTo(self.origin.0, self.origin.1))?;
        for row_chunk in self.pixels.chunks_exact(2) {
            if let [upper, lower] = row_chunk {
                for (foreground, background) in upper.iter().zip(lower.iter()) {
                    queue!(
                        stdout(),
                        SetColors(Colors::new(*foreground, *background)),
                        Print("▀"),
                    )?;
                }
                queue!(stdout(), MoveDown(1), MoveLeft(self.dimension.width))?;
            }
        }
        if self.dimension.height % 2 == 1 {
            let upper = self.pixels.last().unwrap();
            queue!(stdout(), SetBackgroundColor(Color::Reset))?;
            for foreground in upper.iter() {
                queue!(stdout(), SetForegroundColor(*foreground), Print("▀"))?;
            }
        }
        stdout().flush()?;
        Ok(())
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = execute!(
            stdout(),
            terminal::LeaveAlternateScreen,
            terminal::EnableLineWrap,
            cursor::Show
        );
    }
}
