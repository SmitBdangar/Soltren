use std::time::{Duration, Instant};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;

use crate::map::{Map, Material};
use crate::player::Player;
use crate::raycaster;
use crate::renderer::buffer::FrameBuffer;
use crate::renderer::terminal::TerminalHandle;

/// The central engine managing game state and the Core Game Loop.
pub struct Engine {
    map: Map,
    player: Player,
    terminal: TerminalHandle,
    frame_buffer: FrameBuffer,
    last_time: Instant,
}

impl Engine {
    /// Initializes a new game engine session.
    pub fn init() -> Result<Self> {
        let terminal = TerminalHandle::init()?;
        let (cols, rows) = terminal::size()?;
        
        Ok(Self {
            map: Map::new(),
            player: Player::new(),
            terminal,
            frame_buffer: FrameBuffer::new(cols as usize, rows as usize),
            last_time: Instant::now(),
        })
    }

    /// Starts the main game loop. Blocks until the user exits.
    pub fn run(&mut self) -> Result<()> {
        loop {
            let current_time = Instant::now();
            let frame_time = current_time.duration_since(self.last_time).as_secs_f64();
            self.last_time = current_time;

            // Handle terminal resizing gracefully
            let (cols, rows) = terminal::size()?;
            self.frame_buffer.resize(cols as usize, rows as usize);

            // Input Handling (Non-blocking)
            // Process all pending events to prevent input lag
            while event::poll(Duration::from_millis(0))? {
                if let Event::Key(key_event) = event::read()? {
                    if !self.handle_input(key_event.code, frame_time) {
                        // User requested exit
                        self.terminal.cleanup()?;
                        return Ok(());
                    }
                }
            }

            // Render Preparation
            self.frame_buffer.clear();

            // Core Raycasting
            raycaster::render_frame(&self.player, &self.map, &mut self.frame_buffer);

            // FPS Overlay UI
            self.draw_fps_counter(frame_time);

            // Output Flush
            self.frame_buffer.render(&mut self.terminal.stdout)?;

            // Performance Management (Cap to ~60 FPS so we don't melt the CPU)
            let elapsed = current_time.elapsed();
            let target_frame_time = Duration::from_micros(16666);
            if elapsed < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed);
            }
        }
    }

    /// Processes keyboard input. Returns `false` if the engine should terminate.
    fn handle_input(&mut self, key: KeyCode, frame_time: f64) -> bool {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => return false,
            
            // Forward Movement
            KeyCode::Char('w') | KeyCode::Up => {
                let new_x = self.player.position.x + self.player.direction.x * self.player.move_speed * frame_time;
                if self.map.get(new_x as usize, self.player.position.y as usize) == Material::Empty {
                    self.player.position.x = new_x;
                }
                let new_y = self.player.position.y + self.player.direction.y * self.player.move_speed * frame_time;
                if self.map.get(self.player.position.x as usize, new_y as usize) == Material::Empty {
                    self.player.position.y = new_y;
                }
            },
            
            // Backward Movement
            KeyCode::Char('s') | KeyCode::Down => {
                let new_x = self.player.position.x - self.player.direction.x * self.player.move_speed * frame_time;
                if self.map.get(new_x as usize, self.player.position.y as usize) == Material::Empty {
                    self.player.position.x = new_x;
                }
                let new_y = self.player.position.y - self.player.direction.y * self.player.move_speed * frame_time;
                if self.map.get(self.player.position.x as usize, new_y as usize) == Material::Empty {
                    self.player.position.y = new_y;
                }
            },

            // Rotation
            KeyCode::Right | KeyCode::Char('d') => {
                self.player.rotate(-self.player.rot_speed * frame_time);
            },
            KeyCode::Left | KeyCode::Char('a') => {
                self.player.rotate(self.player.rot_speed * frame_time);
            }
            _ => {}
        }
        true
    }

    /// Overlays current Frames Per Second counter to the buffer
    fn draw_fps_counter(&mut self, frame_time: f64) {
        let fps = if frame_time > 0.0 { 1.0 / frame_time } else { 0.0 };
        let fps_str = format!(" FPS: {:.0} ", fps);
        let mut x_offset = 0;
        for ch in fps_str.chars() {
            self.frame_buffer.set(
                x_offset, 
                0, 
                ch, 
                crossterm::style::Color::White, 
                crossterm::style::Color::AnsiValue(236) // Dark Gray Background
            );
            x_offset += 1;
        }
    }
}
