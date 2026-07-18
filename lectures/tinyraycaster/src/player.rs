/// Player
pub struct Player {
    /// Player x coordinate
    pub x: f32,
    /// Player y coordinate
    pub y: f32,
    /// Player view direction in radians
    /// the angle between the view direction and the x axis
    pub ang: f32,
    /// Player fov in radians where midpoint is self.direction
    pub fov: f32,
}
