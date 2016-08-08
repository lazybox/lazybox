//! All colors are linear RGBA

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct PackedColor(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct NormalizedColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_srgb([r, g, b, a]: [u8; 4]) -> Self {
        let conv = |v|
            component_from_float(
                component_srgb_to_linear(
                    component_to_float(v)));
                    
        Color {
            r: conv(r),
            g: conv(g),
            b: conv(b),
            a: a,
        }
    }

    pub fn from_array([r, g, b, a]: [u8; 4]) -> Self {
        Color { r: r, g: g, b: b, a: a }
    }

    pub fn to_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn white() -> Self {
        Self::from_array([255, 255, 255, 255])
    }
}

impl NormalizedColor {
    pub fn from_srgb([r, g, b, a]: [f32; 4]) -> Self {
        use self::component_srgb_to_linear as conv;
        NormalizedColor {
            r: conv(r),
            g: conv(g),
            b: conv(b),
            a: a,
        }
    }

    pub fn from_array([r, g, b, a]: [f32; 4]) -> Self {
        NormalizedColor { r: r, g: g, b: b, a: a }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<Color> for PackedColor {
    fn from(c: Color) -> Self {
        PackedColor((c.r as u32) << 24 | (c.g as u32) << 16 | (c.b as u32) << 8 | c.a as u32)
    }
}

impl From<PackedColor> for Color {
    fn from(PackedColor(c): PackedColor) -> Self {
        const U8_MASK: u32 = 0x000000FF;
        Color {
            r: ( c >> 24) as u8,
            g: ((c >> 16) & U8_MASK) as u8,
            b: ((c >>  8) & U8_MASK) as u8,
            a: ( c        & U8_MASK) as u8,
        }
    }
}

impl From<Color> for NormalizedColor {
    fn from(c: Color) -> Self {
        use self::component_to_float as conv;
        NormalizedColor {
            r: conv(c.r),
            g: conv(c.g),
            b: conv(c.b),
            a: conv(c.a),
        }
    }
}

impl From<NormalizedColor> for Color {
    fn from(c: NormalizedColor) -> Self {
        use self::component_from_float as conv;
        Color {
            r: conv(c.r),
            g: conv(c.g),
            b: conv(c.b),
            a: conv(c.a)
        }
    }
}

impl From<NormalizedColor> for PackedColor {
    fn from(c: NormalizedColor) -> Self {
        Color::from(c).into()
    }
}

impl From<PackedColor> for NormalizedColor {
    fn from(c: PackedColor) -> Self {
        Color::from(c).into()
    }
}

#[inline(always)]
pub fn component_from_float(v: f32) -> u8 {
    (v * 255.0) as u8
}

#[inline(always)]
pub fn component_to_float(v: u8) -> f32 {
    v as f32 / 255.0
}

#[inline(always)]
pub fn component_srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}