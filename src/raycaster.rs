use crate::map::{Map, Material};
use crate::math::Vector2D;
use crate::player::Player;
use crate::renderer::buffer::FrameBuffer;
use crossterm::style::Color;

/// Executes the Digital Differential Analysis (DDA) raycasting algorithm
/// for every vertical column of the screen and draws it to the FrameBuffer.
pub fn render_frame(player: &Player, map: &Map, frame: &mut FrameBuffer) {
    let screen_width = frame.width;
    let screen_height = frame.height as f64;

    for x in 0..screen_width {
        // Calculate ray position and direction
        let camera_x = 2.0 * x as f64 / screen_width as f64 - 1.0; // x-coordinate in camera space
        let ray_dir = player.direction + player.camera_plane * camera_x;

        // Which box of the map we're in
        let mut map_x = player.position.x as i32;
        let mut map_y = player.position.y as i32;

        // Length of ray from current position to next x or y-side
        let mut side_dist_x: f64;
        let mut side_dist_y: f64;

        // Length of ray from one x or y-side to next x or y-side
        let delta_dist_x = if ray_dir.x == 0.0 { f64::MAX } else { (1.0 / ray_dir.x).abs() };
        let delta_dist_y = if ray_dir.y == 0.0 { f64::MAX } else { (1.0 / ray_dir.y).abs() };
        let perp_wall_dist: f64;

        // What direction to step in x or y-direction (either +1 or -1)
        let step_x: i32;
        let step_y: i32;

        let mut hit = false; // Was there a wall hit?
        let mut side = 0; // Was a NS or a EW wall hit?
        let mut hit_material = Material::Empty;

        // Calculate step and initial side_dist
        if ray_dir.x < 0.0 {
            step_x = -1;
            side_dist_x = (player.position.x - map_x as f64) * delta_dist_x;
        } else {
            step_x = 1;
            side_dist_x = (map_x as f64 + 1.0 - player.position.x) * delta_dist_x;
        }
        if ray_dir.y < 0.0 {
            step_y = -1;
            side_dist_y = (player.position.y - map_y as f64) * delta_dist_y;
        } else {
            step_y = 1;
            side_dist_y = (map_y as f64 + 1.0 - player.position.y) * delta_dist_y;
        }

        // Perform DDA
        while !hit {
            // Jump to next map square, either in x-direction, or in y-direction
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                side = 0;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                side = 1;
            }
            
            // Check if ray has hit a wall
            hit_material = map.get(map_x as usize, map_y as usize);
            if hit_material != Material::Empty {
                hit = true;
            }
        }

        // Calculate distance projected on camera direction
        if side == 0 {
            perp_wall_dist = (map_x as f64 - player.position.x + (1.0 - step_x as f64) / 2.0) / ray_dir.x;
        } else {
            perp_wall_dist = (map_y as f64 - player.position.y + (1.0 - step_y as f64) / 2.0) / ray_dir.y;
        }

        // Calculate height of line to draw on screen
        let line_height = (screen_height / perp_wall_dist) as i32;

        // Calculate lowest and highest pixel to fill in current stripe
        let mut draw_start = -line_height / 2 + frame.height as i32 / 2;
        if draw_start < 0 {
            draw_start = 0;
        }
        let mut draw_end = line_height / 2 + frame.height as i32 / 2;
        if draw_end >= frame.height as i32 {
            draw_end = frame.height as i32 - 1;
        }

        // Choose wall color based on material
        let (r, g, b) = match hit_material {
            Material::SolidWall => (180, 0, 0),     // Red
            Material::BrickWall => (0, 180, 0),     // Green
            Material::StoneWall => (0, 0, 180),     // Blue
            Material::WoodWall => (180, 180, 180),  // White
            Material::OutOfBounds => (50, 50, 50),  // Dark Gray border
            Material::Empty => (0, 0, 0),
        };

        // Make shadows by dimming colors (dist and side based)
        let dim_factor = if side == 1 { 0.7 } else { 1.0 }; // N/S vs E/W
        let dist_dim = (1.0 - (perp_wall_dist / 20.0)).max(0.1); 
        let final_dim = dim_factor * dist_dim;

        let color = Color::Rgb {
            r: (r as f64 * final_dim) as u8,
            g: (g as f64 * final_dim) as u8,
            b: (b as f64 * final_dim) as u8,
        };

        // Choose ASCII character based on distance for "texture"
        let ch = if perp_wall_dist <= 2.0 {
            '█'
        } else if perp_wall_dist <= 4.0 {
            '▓'
        } else if perp_wall_dist <= 8.0 {
            '▒'
        } else {
            '░'
        };

        // Draw the vertical stripe
        for y in draw_start..draw_end {
            frame.set(x, y as usize, ch, color, Color::Reset);
        }
    }
}
