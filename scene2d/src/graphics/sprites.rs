use core::nalgebra::{Point2, Vector2};
use {TextureBind, Color};

#[derive(Clone, Debug)]
pub struct Sprite {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub rotation: f32,
    pub texture: SpriteTexture,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct SpriteTexture {
    pub bind: TextureBind,
    pub coord_inf: [f32; 2],
    pub coord_sup: [f32; 2],
}
