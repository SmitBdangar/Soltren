use crate::math::Vector2D;

/// Represents the player entity navigating the map.
pub struct Player {
    /// The absolute position of the player in the map grid.
    pub position: Vector2D,
    /// The normalized direction vector the player is currently facing.
    pub direction: Vector2D,
    /// The camera plane vector, perpendicular to the direction, dictating FOV.
    pub camera_plane: Vector2D,
    /// Movement speed in units per second.
    pub move_speed: f64,
    /// Rotation speed in radians per second.
    pub rot_speed: f64,
}

impl Player {
    /// Creates a new player at the standard starting position with default speeds.
    pub fn new() -> Self {
        Self {
            position: Vector2D::new(22.0, 12.0),
            direction: Vector2D::new(-1.0, 0.0),
            camera_plane: Vector2D::new(0.0, 0.66), // ~66 degree FOV
            move_speed: 5.0,
            rot_speed: 3.0,
        }
    }

    /// Rotates the player's camera by the given rotation amount (in radians).
    /// Positive `rot_amt` rotates the camera right (clockwise in a top-down view).
    pub fn rotate(&mut self, rot_amt: f64) {
        self.direction = self.direction.rotate(rot_amt);
        self.camera_plane = self.camera_plane.rotate(rot_amt);
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
