use std::io::{stdout, Write};

use crossterm::cursor::{self, MoveDown, MoveLeft, MoveTo};
use crossterm::style::{Color, Colors, Print, SetBackgroundColor, SetColors, SetForegroundColor};
use crossterm::{execute, queue, terminal, Result};

extern crate nalgebra as na;
use na::{DMatrix, Point2};

#[derive(Debug)]
pub struct Window {
    origin: Point2<u16>,
    pixels: DMatrix<Color>,
}

impl Window {
    pub fn new(height: u16, width: u16) -> Result<Self> {
        let (columns, rows) = terminal::size()?;
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            terminal::DisableLineWrap,
            cursor::Hide
        )?;
        terminal::enable_raw_mode()?;
        Ok(Window {
            origin: Point2::new(
                (columns as f32 / 2. - width as f32 / 2.) as u16,
                (rows as f32 / 2. - height as f32 / 4.) as u16,
            ),
            pixels: DMatrix::from_element(height.into(), width.into(), Color::Black),
        })
    }

    pub fn width(&self) -> u16 {
        self.pixels.ncols() as u16
    }

    pub fn height(&self) -> u16 {
        self.pixels.nrows() as u16
    }

    pub fn set_pixel(&mut self, y: u16, x: u16, color: Color) {
        debug_assert_ne!(color, Color::Reset, "Cannot set pixel to Color::Reset");
        self.pixels[(y as usize, x as usize)] = color;
    }

    pub fn draw(&self) -> Result<()> {
        queue!(stdout(), MoveTo(self.origin.x, self.origin.y))?;
        for (upper, lower) in std::iter::zip(
            self.pixels.row_iter().step_by(2),
            self.pixels.row_iter().skip(1).step_by(2),
        ) {
            for (foreground, background) in std::iter::zip(&upper, &lower) {
                queue!(
                    stdout(),
                    SetColors(Colors::new(*foreground, *background)),
                    Print("▀"),
                )?;
            }
            queue!(stdout(), MoveDown(1), MoveLeft(self.pixels.ncols() as u16))?;
        }
        if self.pixels.nrows() % 2 == 1 {
            queue!(stdout(), SetBackgroundColor(Color::Reset))?;
            for foreground in &self.pixels.row_iter().last().unwrap() {
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
        let _ = terminal::disable_raw_mode();
    }
}
