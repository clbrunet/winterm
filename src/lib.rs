use std::io::{stdout, Write};

use crossterm::cursor::{self, MoveDown, MoveLeft, MoveTo};
use crossterm::style::{Color, Colors, Print, SetBackgroundColor, SetColors, SetForegroundColor};
use crossterm::{execute, queue, terminal, Result};

extern crate nalgebra as na;
use na::{DMatrix, Point2};

const UPPER_HALF_BLOCK: &str = "▀";
const LOWER_HALF_BLOCK: &str = "▄";
const FULL_BLOCK: &str = "█";

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
        let window = Window {
            origin: Point2::new(
                (columns as f32 / 2. - width as f32 / 2.) as u16,
                (rows as f32 / 2. - height as f32 / 4.) as u16,
            ),
            pixels: DMatrix::from_element(height.into(), width.into(), Color::Black),
        };
        window.draw_with_border()?;
        Ok(window)
    }

    pub fn width(&self) -> u16 {
        self.pixels.ncols() as u16
    }

    pub fn height(&self) -> u16 {
        self.pixels.nrows() as u16
    }

    pub fn set_pixel(&mut self, y: u16, x: u16, color: Color) {
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
                        Print(UPPER_HALF_BLOCK),
                    )?;
                }
                queue!(stdout(), MoveDown(1), MoveLeft(self.width() as u16))?;
            }
        if self.height() % 2 == 1 {
            queue!(stdout(), SetForegroundColor(Color::Reset))?;
            for background in &self.pixels.row_iter().last().unwrap() {
                queue!(
                    stdout(),
                    SetBackgroundColor(*background),
                    Print(LOWER_HALF_BLOCK)
                )?;
            }
        }
        stdout().flush()?;
        Ok(())
    }

    fn draw_border(&self) -> Result<()> {
        queue!(
            stdout(),
            MoveTo(self.origin.x - 1, self.origin.y - 1),
            Print(LOWER_HALF_BLOCK.repeat((self.width() + 2).into()))
        )?;
        for y in 0..((self.height() + 1) / 2) {
            queue!(
                stdout(),
                MoveTo(self.origin.x - 1, self.origin.y + y),
                Print(FULL_BLOCK),
                MoveTo(self.origin.x + self.width(), self.origin.y + y),
                Print(FULL_BLOCK),
            )?;
        }
        if self.height() % 2 == 0 {
            queue!(
                stdout(),
                MoveTo(self.origin.x - 1, self.origin.y + (self.height() / 2)),
                Print(UPPER_HALF_BLOCK.repeat((self.width() + 2).into()))
            )?;
        }
        Ok(())
    }

    fn draw_with_border(&self) -> Result<()> {
        self.draw_border()?;
        self.draw()?;
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
