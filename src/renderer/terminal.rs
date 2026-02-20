use anyhow::{Context, Result};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{stdout, Stdout};

/// Handles raw OS-level terminal interactions (Alternative Screen, Raw Mode).
pub struct TerminalHandle {
    pub stdout: Stdout,
}

impl TerminalHandle {
    /// Initializes the terminal handle, taking control of the screen state.
    pub fn init() -> Result<Self> {
        enable_raw_mode().context("Failed to enable terminal raw mode")?;
        let mut out = stdout();
        out.execute(EnterAlternateScreen).context("Failed to enter alternate terminal screen")?;
        
        Ok(Self { stdout: out })
    }

    /// Cleans up the terminal handle, restoring OS-default cooked mode.
    /// It is critical this is called on exit to un-bork the user's terminal.
    pub fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode().context("Failed to disable terminal raw mode")?;
        self.stdout.execute(LeaveAlternateScreen).context("Failed to leave alternate terminal screen")?;
        Ok(())
    }
}

impl Drop for TerminalHandle {
    fn drop(&mut self) {
        // Fallback cleanup if not explicitly called
        let _ = self.cleanup();
    }
}

/// A global panic hook to ensure terminal states are restored even 
/// on unexpected app crashes.
pub fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Force raw mode off so we can actually see the panic output
        let _ = disable_raw_mode();
        let mut stdout = stdout();
        let _ = stdout.execute(LeaveAlternateScreen);
        
        // Let the original panic hook print the message in cooked/normal mode
        original_hook(panic_info);
    }));
}
