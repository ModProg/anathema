//! The display crates contains the essential types for drawing in the terminal.
//!
//! It uses two buffers and only draws the diffs from top left to bottom right, making it less
//! likely to flicker when moving the cursor etc.
//!
//! ```
//! # fn stdout() -> Vec<u8> {
//! #     vec![0u8; 80*50]
//! # }
//! use anathema::display::{Screen, Size, Style, ScreenPos, Color};
//! let mut output = stdout();
//! let mut screen = Screen::new(&mut output, (80u16, 50u16)).unwrap();
//!
//! // Clear the screen first
//! screen.clear_all(&mut output);
//!
//! // Set the foreground to red
//! let mut style = Style::new();
//! style.set_fg(Color::Red);
//!
//! screen.put('x', style, ScreenPos::new(2, 4));
//!
//! // Render to stdout
//! screen.render(&mut output);
//!
//! // ... and finally restore the mouse cursor etc.
//! screen.restore(&mut output);
//! ```
#![deny(missing_docs)]
use std::ops::{Add, Sub};

mod buffer;
mod screen;
mod style;

// -----------------------------------------------------------------------------
//     - Re-exports -
// -----------------------------------------------------------------------------
pub use buffer::Buffer;
pub use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
pub use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
pub use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
pub use crossterm::{cursor, ExecutableCommand, QueueableCommand};
pub use screen::Screen;
pub use style::{Attributes, Style};

pub mod events {
    //! Re-export crossterm events
    pub use crossterm::event::read;
    pub use crossterm::event::{
        Event as CrossEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
}

/// Size
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size {
    /// Width
    pub width: usize,
    /// Height
    pub height: usize,
}

impl Size {
    /// Zero size
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new Size
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl From<(usize, usize)> for Size {
    fn from(parts: (usize, usize)) -> Self {
        Size::new(parts.0, parts.1)
    }
}

impl From<(u16, u16)> for Size {
    fn from(parts: (u16, u16)) -> Self {
        Size::new(parts.0 as usize, parts.1 as usize)
    }
}

impl Add for Size {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self { width: self.width + other.width, height: self.height + other.height }
    }
}

impl Sub for Size {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self { width: self.width - other.width, height: self.height - other.height }
    }
}

/// Represents a position on the screen, meaning this should never
/// be a value outside of the screen size.
///
/// It will be ignored if the value is used in a drawing operation and it's outside the current
/// screen size.
///
/// `Screen::ZERO` is the top left of a buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenPos {
    /// The x coordinate on screen
    pub x: u16,
    /// The y coordinate on screen
    pub y: u16,
}

impl ScreenPos {
    /// A zero screen size
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new instance of a `ScreenPos`
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}
