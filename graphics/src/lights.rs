use cgmath::Point2;

use layer::LayerId;

#[derive(Clone, Debug)]
pub struct AmbientLight {
    pub color: LightColor,
    pub intensity: f32,
}

#[derive(Clone, Debug)]
pub struct Light {
    pub center: Point2<f32>,
    pub radius: f32,
    pub source_radius: f32,
    pub source_layer: LayerId,
    pub color: LightColor,
    pub intensity: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct LightColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl LightColor {
    pub fn from_srgb([r, g, b]: [f32; 3]) -> Self {
        use color::component_srgb_to_linear as conv;
        LightColor {
            r: conv(r),
            g: conv(g),
            b: conv(b),
        }
    }

    pub fn from_array([r, g, b]: [f32; 3]) -> Self {
        LightColor { r: r, g: g, b: b }
    }

    pub fn to_array(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}