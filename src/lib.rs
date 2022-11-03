//! # winterm
//! A Rust library to create a pixelated window inside a terminal.
//!
//! It uses [crossterm] as a backend.
//!
//! # Adding winterm as a dependency
//!
//! ```sh
//! cargo add winterm@0.6.0
//! cargo add crossterm@0.25.0
//! ```
//!
//! # Create a window
//!
//! ```
//! use winterm::Window;
//!
//! # let height = 9;
//! # let width = 16;
//! let mut window = Window::new(height, width)?;
//! # Ok::<(), crossterm::ErrorKind>(())
//! ```
//!
//! # Render the next frame
//!
//! ```
//! use crossterm::style::Color;
//! # use winterm::Window;
//!
//! # let mut window = Window::new(9, 16)?;
//! window.set_pixel(0, 0, Color::Red);
//! # let y = 1;
//! # let x = 1;
//! window.set_pixel(
//!     y,
//!     x,
//!     Color::Rgb {
//!         r: 0x3E,
//!         g: 0xB4,
//!         b: 0x89,
//!     },
//! );
//! window.redraw()?;
//! # Ok::<(), crossterm::ErrorKind>(())
//! ```
//!
//! # React to events
//!
//! ```
//! use crossterm::event::KeyCode;
//! # use winterm::Window;
//!
//! # let mut window = Window::new(9, 16)?;
//! window.poll_events()?;
//! if window.get_key(KeyCode::Esc) {
//!     // the Escape key has been pressed
//! }
//! if window.get_key(KeyCode::Char('w')) {
//!     // the W key has been pressed
//! }
//! # Ok::<(), crossterm::ErrorKind>(())
//! ```
//!
//! # Example
//!
//! ```no_run
//! use crossterm::{event::KeyCode, style::Color, Result};
//! use winterm::Window;
//!
//! fn main() -> Result<()> {
//!     let mut window = Window::new(9, 16)?;
//!     let mut color = Color::Black;
//!     loop {
//!         window.poll_events()?;
//!         if window.get_key(KeyCode::Esc) {
//!             break;
//!         }
//!         if window.get_key(KeyCode::Char('n')) {
//!             color = match color {
//!                 Color::Black => Color::Red,
//!                 Color::Red => Color::Rgb {
//!                     r: 0x3E,
//!                     g: 0xB4,
//!                     b: 0x89,
//!                 },
//!                 _ => Color::Black,
//!             }
//!         }
//!         for y in 0..window.height() {
//!             for x in 0..window.width() {
//!                 window.set_pixel(y, x, color);
//!             }
//!         }
//!         window.redraw()?;
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Debugging
//!
//! Since winterm uses the terminal "alternate screen", it can be complicated to debug using the print functions.
//!
//! One way to deal with this problem is to use [stderr] (e.g. [`dbg!`], [`eprintln!`], ...) and redirect it to a file :
//! ```sh
//! cargo run 2> logs
//! ```
//! You can now use `cat logs` after execution or `tail -f logs` in another terminal to get your output while the code is still running.
//!
//! [stderr]: https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)

use std::io::{stdout, Write};
use std::time::Duration;
use std::{cmp, iter};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::KeyModifiers;
use crossterm::event::{self, Event, Event::Key, Event::Resize, KeyCode};
use crossterm::style::{Color, Colors, Print, SetBackgroundColor, SetColors, SetForegroundColor};
use crossterm::terminal::{
    Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, queue, terminal, Result};

extern crate nalgebra as na;
use na::{DMatrix, Point2, Vector2};

const UPPER_HALF_BLOCK: &str = "▀";
const LOWER_HALF_BLOCK: &str = "▄";
const FULL_BLOCK: &str = "█";

/// Window representation.
/// Used for drawing and events handling.
#[derive(Debug)]
pub struct Window {
    terminal_size: Vector2<u16>,
    origin: Point2<i16>,
    pixels: DMatrix<Color>,
    last_events: Vec<Event>,
}

impl Window {
    fn calculate_origin(&mut self) {
        self.origin.x = (self.terminal_size.x as f32 / 2. - self.width() as f32 / 2.) as i16;
        self.origin.y = (self.terminal_size.y as f32 / 2. - self.height() as f32 / 4.) as i16;
    }

    /// Creates a window.
    pub fn new(height: u16, width: u16) -> Result<Self> {
        let (columns, rows) = terminal::size()?;
        execute!(stdout(), EnterAlternateScreen, DisableLineWrap, Hide)?;
        terminal::enable_raw_mode()?;
        let mut window = Window {
            terminal_size: Vector2::new(columns, rows),
            origin: Point2::origin(),
            pixels: DMatrix::from_element(height.into(), width.into(), Color::Black),
            last_events: Vec::new(),
        };
        window.calculate_origin();
        window.redraw_all()?;
        Ok(window)
    }

    /// Gets the window width.
    pub fn width(&self) -> u16 {
        self.pixels.ncols() as u16
    }

    /// Gets the window height.
    pub fn height(&self) -> u16 {
        self.pixels.nrows() as u16
    }

    fn end_x(&self) -> u16 {
        (self.origin.x + self.width() as i16) as u16
    }

    fn end_y(&self) -> u16 {
        (self.origin.y + ((self.height() + 1) / 2) as i16) as u16
    }

    /// Sets a pixel color.
    pub fn set_pixel(&mut self, y: u16, x: u16, color: Color) {
        self.pixels[(y.into(), x.into())] = color;
    }

    /// Redraws the window to the terminal.
    pub fn redraw(&self) -> Result<()> {
        let skipable_rows_count = cmp::max(-self.origin.y, 0) as usize;
        let skipable_columns_count = cmp::max(-self.origin.x, 0) as usize;
        let start_x = cmp::max(self.origin.x, 0) as u16;
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
            for (foreground, background) in iter::zip(
                upper
                    .into_iter()
                    .skip(skipable_columns_count)
                    .take(self.terminal_size.x as usize),
                lower
                    .into_iter()
                    .skip(skipable_columns_count)
                    .take(self.terminal_size.x as usize),
            ) {
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
            for background in self
                .pixels
                .row_iter()
                .last()
                .unwrap()
                .into_iter()
                .skip(skipable_columns_count)
                .take(self.terminal_size.x as usize)
            {
                queue!(
                    stdout(),
                    SetBackgroundColor(*background),
                    Print(LOWER_HALF_BLOCK)
                )?;
            }
        }
        queue!(stdout(), SetColors(Colors::new(Color::Reset, Color::Reset)))?;
        stdout().flush()?;
        Ok(())
    }

    fn redraw_border(&self, should_flush: bool) -> Result<()> {
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
        if should_flush {
            stdout().flush()?;
        }
        Ok(())
    }

    fn redraw_all(&self) -> Result<()> {
        queue!(stdout(), Clear(ClearType::All))?;
        self.redraw_border(false)?;
        self.redraw()?;
        Ok(())
    }

    /// Clears events and polls for newer events.
    pub fn poll_events(&mut self) -> Result<()> {
        self.last_events.clear();
        while event::poll(Duration::from_secs(0))? {
            self.last_events.push(event::read()?);
            if let Resize(columns, rows) = self.last_events.last().unwrap() {
                self.terminal_size.x = *columns;
                self.terminal_size.y = *rows;
                self.calculate_origin();
                self.redraw_all()?;
            }
        }
        Ok(())
    }

    /// Returns `true` if `key` was read during the last call to [`Window::poll_events`].
    pub fn get_key(&mut self, key: KeyCode) -> bool {
        self.last_events.iter().any(|event| {
            if let Key(key_event) = *event {
                if key_event.code == key {
                    return true;
                }
                if let (KeyCode::Char(char), KeyCode::Char(event_char)) = (key, key_event.code) {
                    if char.to_lowercase().to_string() == event_char.to_lowercase().to_string() {
                        return true;
                    }
                }
            }
            false
        })
    }

    /// Returns `true` if `modifiers` was read during the last call to [`Window::poll_events`].
    pub fn get_modifiers(&mut self, modifiers: KeyModifiers) -> bool {
        self.last_events.iter().any(|event| {
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
