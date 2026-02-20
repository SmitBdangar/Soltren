pub mod engine;
pub mod map;
pub mod math;
pub mod player;
pub mod raycaster;
pub mod renderer;

use anyhow::Result;

fn main() -> Result<()> {
    // Ensure terminal restores cooked mode even on panic
    renderer::terminal::setup_panic_hook();

    let mut game = engine::Engine::init()?;
    game.run()?;

    Ok(())
}
