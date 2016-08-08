pub const QUAD_VERTICES: [Position; 4] = [
    Position { position: [ 1.0, -1.0] }, // bottom right
    Position { position: [ 1.0,  1.0] }, // top right
    Position { position: [-1.0,  1.0] }, // top left
    Position { position: [-1.0, -1.0] }, // bottom left
];

pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub const HALF_QUAD_VERTICES: [Position; 4] = [
    Position { position: [ 0.5, -0.5] }, // bottom right
    Position { position: [ 0.5,  0.5] }, // top right
    Position { position: [-0.5,  0.5] }, // top left
    Position { position: [-0.5, -0.5] }, // bottom left
];

pub const HALF_QUAD_INDICES: [u16; 6] = QUAD_INDICES;

gfx_defines! {
    vertex Position {
        position: [f32; 2] = "a_Position",
    }
}