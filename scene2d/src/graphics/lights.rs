use core::nalgebra::Point2;

use graphics::layer::LayerId;

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
    pub fn from_srgb(cs: [f32; 3]) -> Self {
        use graphics::color::component_srgb_to_linear as conv;
        LightColor {
            r: conv(cs[0]),
            g: conv(cs[1]),
            b: conv(cs[2]),
        }
    }

    pub fn from_array(cs: [f32; 3]) -> Self {
        LightColor { r: cs[0], g: cs[1], b: cs[2] }
    }

    pub fn to_array(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}
