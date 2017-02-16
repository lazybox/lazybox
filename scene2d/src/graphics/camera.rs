use core::winit::Window;
use core::nalgebra::{Point2, Vector2};
use core::alga::linear::EuclideanSpace;

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub translate: Vector2<f32>,
    pub scale: Vector2<f32>,
}

impl Camera {
    pub fn new(position: Point2<f32>, size: Vector2<f32>, window: &Window) -> Self {
        let mut camera = Camera {
            position: position,
            size: size,
            translate: Vector2::new(0., 0.),
            scale: Vector2::new(1., 1.),
        };
        camera.update_transform(window);
        camera
    }

    pub fn update_transform(&mut self, window: &Window) {
        let (w, h) = window.get_inner_size_pixels().unwrap();
        let window_aspect_ratio = w as f32 / h as f32;
        let camera_aspect_ratio = self.size.x / self.size.y;
        let aspect_ratio = window_aspect_ratio / camera_aspect_ratio;

        let size = if aspect_ratio >= 1. {
            Vector2::new(self.size.x * aspect_ratio, self.size.y)
        } else {
            Vector2::new(self.size.x, self.size.y / aspect_ratio)
        };

        self.translate = -self.position.coordinates();
        self.scale = Vector2::new(2. / size.x, 2. / size.y);
    }

    pub fn window_point_to_gl(p: Point2<i32>, window: &Window) -> Point2<f32> {
        let (w, h) = window.get_inner_size_pixels().unwrap();
        Point2::new((p.x as f32 / w as f32) * 2.0 - 1.0,
                    1.0 - (p.y as f32 / h as f32) * 2.0)
    }

    pub fn window_vector_to_gl(v: Vector2<i32>, window: &Window) -> Vector2<f32> {
        let (w, h) = window.get_inner_size_pixels().unwrap();
        Vector2::new((v.x as f32 / w as f32),
                     (v.y as f32 / h as f32))
    }

    pub fn window_point_to_world(&self, p: Point2<i32>, window: &Window) -> Point2<f32> {
        let gl = Self::window_point_to_gl(p, window);
        Point2::new(gl.x / self.scale.x - self.translate.x,
                    gl.y / self.scale.y - self.translate.y)
    }

    pub fn window_vector_to_world(&self, v: Vector2<i32>, window: &Window) -> Vector2<f32> {
        let gl = Self::window_vector_to_gl(v, window);
        Vector2::new(gl.x / self.scale.x,
                     gl.y / self.scale.y)
    }
}

gfx_defines! {
    constant Locals {
        translate: [f32; 2] = "u_Translate",
        scale: [f32; 2] = "u_Scale",
    }
}
