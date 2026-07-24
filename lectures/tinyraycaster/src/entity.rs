pub trait Entity {
    /// Get x-position
    fn x(&self) -> f32;
    /// Get y-position
    fn y(&self) -> f32;
    /// Get angle of entity
    fn angle(&self) -> f32;
}
