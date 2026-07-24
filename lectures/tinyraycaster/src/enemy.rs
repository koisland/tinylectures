use crate::entity::Entity;

#[derive(Debug, Clone)]
pub struct Enemy {
    /// x position
    pub x: f32,
    /// y position
    pub y: f32,
    /// Angle of enemy
    pub angle: f32,
    /// texture id on sprite sheet
    // TODO: Add field for front texture and back texture or hashmap of texture types (front, back, side,)
    pub _texture_id: usize,
}

impl Entity for Enemy {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }

    fn angle(&self) -> f32 {
        self.angle
    }
}
