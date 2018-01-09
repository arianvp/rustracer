#[derive(Debug, Clone)]
pub struct Vec2 {
    pub position: [f32; 2],
}
impl_vertex!(Vec2, position);
