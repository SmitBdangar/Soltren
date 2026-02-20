use anyhow::{Context, Result};
use crossterm::{
    cursor,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use std::io::Write;

/// A double-buffered 2D grid containing ASCII characters and 
/// foreground/background terminal colors.
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    buffer: Vec<char>,
    fg_colors: Vec<Color>,
    bg_colors: Vec<Color>,
}

impl FrameBuffer {
    /// Creates a new framebuffer of a specific dimension.
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            buffer: vec![' '; size],
            fg_colors: vec![Color::Reset; size],
            bg_colors: vec![Color::Reset; size],
        }
    }

    /// Dynamically resizes the framebuffer if the terminal changes dimensions.
    pub fn resize(&mut self, width: usize, height: usize) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        let size = width * height;
        self.buffer.resize(size, ' ');
        self.fg_colors.resize(size, Color::Reset);
        self.bg_colors.resize(size, Color::Reset);
    }

    /// Clears the screen, drawing a split ceiling (top) and floor (bottom) gradient.
    pub fn clear(&mut self) {
        let half_height = self.height / 2;
        
        // Ceiling (top half)
        for y in 0..half_height {
            for x in 0..self.width {
                self.set(x, y, ' ', Color::Reset, Color::AnsiValue(234));
            }
        }
        
        // Floor (bottom half)
        for y in half_height..self.height {
            for x in 0..self.width {
                self.set(x, y, ' ', Color::Reset, Color::AnsiValue(238));
            }
        }
    }

    /// Writes a character with specified colors to a specific coordinate.
    pub fn set(&mut self, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.buffer[idx] = ch;
            self.fg_colors[idx] = fg;
            self.bg_colors[idx] = bg;
        }
    }

    /// Flushes the entire buffer sequentially to stdout to prevent flickering.
    pub fn render<W: Write>(&self, stdout: &mut W) -> Result<()> {
        stdout.queue(cursor::Hide).context("Failed to hide cursor")?;
        stdout.queue(cursor::MoveTo(0, 0)).context("Failed to move cursor")?;

        let mut current_fg = Color::Reset;
        let mut current_bg = Color::Reset;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let fg = self.fg_colors[idx];
                let bg = self.bg_colors[idx];
                let ch = self.buffer[idx];

                if fg != current_fg {
                    stdout.queue(SetForegroundColor(fg)).context("Failed to set fg color")?;
                    current_fg = fg;
                }
                if bg != current_bg {
                    stdout.queue(SetBackgroundColor(bg)).context("Failed to set bg color")?;
                    current_bg = bg;
                }
                stdout.queue(Print(ch)).context("Failed to print char")?;
            }
            if y < self.height - 1 {
                // Clear color states purely for terminal robustness before printing line endings
                stdout.queue(SetForegroundColor(Color::Reset)).context("Failed to reset fg color")?;
                stdout.queue(SetBackgroundColor(Color::Reset)).context("Failed to reset bg color")?;
                current_fg = Color::Reset;
                current_bg = Color::Reset;
                stdout.queue(Print("\r\n")).context("Failed to new line")?;
            }
        }
        stdout.flush().context("Failed to flush frame buffer to stdout")?;
        Ok(())
    }
}
