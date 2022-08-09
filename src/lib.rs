use std::io::{stdout, Write};
use std::time::Duration;
use std::{cmp, iter};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::KeyModifiers;
use crossterm::event::{poll, read, Event, Event::Key, KeyCode};
use crossterm::style::{Color, Colors, Print, SetBackgroundColor, SetColors, SetForegroundColor};
use crossterm::terminal::{
    DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, queue, terminal, Result};

extern crate nalgebra as na;
use na::{DMatrix, Point2, Vector2};

const UPPER_HALF_BLOCK: &str = "▀";
const LOWER_HALF_BLOCK: &str = "▄";
const FULL_BLOCK: &str = "█";

#[derive(Debug)]
pub struct Window {
    terminal_size: Vector2<u16>,
    origin: Point2<i16>,
    pixels: DMatrix<Color>,
    last_events: Vec<Event>,
}

impl Window {
    pub fn new(height: u16, width: u16) -> Result<Self> {
        let (columns, rows) = terminal::size()?;
        execute!(stdout(), EnterAlternateScreen, DisableLineWrap, Hide)?;
        terminal::enable_raw_mode()?;
        let window = Window {
            terminal_size: Vector2::new(columns, rows),
            origin: Point2::new(
                (columns as f32 / 2. - width as f32 / 2.) as i16,
                (rows as f32 / 2. - height as f32 / 4.) as i16,
            ),
            pixels: DMatrix::from_element(height.into(), width.into(), Color::Black),
            last_events: Vec::new(),
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

    fn end_x(&self) -> u16 {
        (self.origin.x + self.width() as i16) as u16
    }

    fn end_y(&self) -> u16 {
        (self.origin.y + ((self.height() + 1) / 2) as i16) as u16
    }

    pub fn set_pixel(&mut self, y: u16, x: u16, color: Color) {
        self.pixels[(y.into(), x.into())] = color;
    }

    pub fn draw(&self) -> Result<()> {
        let skipable_rows_count = cmp::max(-self.origin.y, 0) as usize;
        let start_x = cmp::max(self.origin.x, 0) as u16;
        let end_x = cmp::min(self.end_x(), self.terminal_size.x) as u16;
        for (y, (upper, lower)) in iter::zip(
            cmp::max(self.origin.y, 0) as u16..cmp::min(self.end_y(), self.terminal_size.y),
            iter::zip(
                self.pixels.row_iter().skip(skipable_rows_count).step_by(2),
                self.pixels
                    .row_iter()
                    .skip(skipable_rows_count + 1)
                    .step_by(2),
            ),
        ) {
                queue!(stdout(), MoveTo(start_x, y))?;
                for (_, (foreground, background)) in
                    iter::zip(start_x..end_x, iter::zip(&upper, &lower))
                    {
                        queue!(
                            stdout(),
                            SetColors(Colors::new(*foreground, *background)),
                            Print(UPPER_HALF_BLOCK),
                        )?;
                    }
            }
        if self.height() % 2 == 1 && self.end_y() <= self.terminal_size.y {
            queue!(
                stdout(),
                MoveTo(start_x, self.end_y() - 1),
                SetForegroundColor(Color::Reset)
            )?;
            for (_, background) in
                iter::zip(start_x..end_x, &self.pixels.row_iter().last().unwrap())
                {
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
        if self.origin.y > 0 {
            queue!(
                stdout(),
                MoveTo(
                    cmp::max(self.origin.x - 1, 0) as u16,
                    (self.origin.y - 1) as u16
                ),
                Print(
                    LOWER_HALF_BLOCK
                        .repeat(cmp::min(self.width() + 2, self.terminal_size.x).into())
                )
            )?;
        }
        let range = cmp::max(self.origin.y, 0) as u16..cmp::min(self.end_y(), self.terminal_size.y);
        if self.origin.x > 0 {
            for y in range.clone() {
                queue!(
                    stdout(),
                    MoveTo((self.origin.x - 1) as u16, y),
                    Print(FULL_BLOCK)
                )?;
            }
        }
        if self.end_x() < self.terminal_size.x {
            for y in range {
                queue!(stdout(), MoveTo(self.end_x(), y), Print(FULL_BLOCK))?;
            }
        }
        if self.height() % 2 == 0 && self.end_y() < self.terminal_size.y {
            queue!(
                stdout(),
                MoveTo(cmp::max(self.origin.x - 1, 0) as u16, self.end_y()),
                Print(
                    UPPER_HALF_BLOCK
                        .repeat(cmp::min(self.width() + 2, self.terminal_size.x).into())
                )
            )?;
        }
        Ok(())
    }

    fn draw_with_border(&self) -> Result<()> {
        self.draw_border()?;
        self.draw()?;
        Ok(())
    }

    pub fn poll_events(&mut self) -> Result<()> {
        self.last_events.clear();
        while poll(Duration::from_secs(0))? {
            self.last_events.push(read()?);
        }
        Ok(())
    }

    pub fn get_key(&mut self, key: KeyCode) -> bool {
        self.last_events
            .iter()
            .any(|event| {
                if let Key(key_event) = *event {
                    if key_event.code == key {
                        return true;
                    }
                    if let (KeyCode::Char(char), KeyCode::Char(event_char)) = (key, key_event.code)
                    {
                        if char.to_lowercase().to_string() == event_char.to_lowercase().to_string()
                        {
                            return true;
                        }
                    }
                }
                false
            })
    }

    pub fn get_modifiers(&mut self, modifiers: KeyModifiers) -> bool {
        self.last_events
            .iter()
            .any(|event| {
                if let Key(key_event) = *event {
                    if key_event.modifiers == modifiers {
                        return true;
                    }
                }
                false
            })
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = execute!(stdout(), LeaveAlternateScreen, EnableLineWrap, Show);
        let _ = terminal::disable_raw_mode();
    }
}
