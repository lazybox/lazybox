use ecs::Context;
use ecs::state::CommitArgs;
use ecs::entity::{Accessor, Entities};
use ecs::module::{Module, HasComponent};
use ecs::module::{Component, Template, ComponentType};
use ecs::module::{StorageLock, StorageReadGuard, StorageWriteGuard};
use modules::storages::packed::Packed;
use modules::data::storages::Storage;

use cgmath::{Point2, Vector2};
use glutin::{Window, WindowBuilder};
use graphics::{Graphics, Camera, NormalizedColor};
use graphics::sprites::Sprite;
use graphics::lights::{AmbientLight, LightColor};
use graphics::specialized::sprites::Renderer;

pub type SpriteStorage = Packed<Sprite>;
impl Template for Sprite {}
derive_component!(Sprite, Sprite, GraphicsModule);
impl_has_component!(Sprite, SpriteStorage, GraphicsModule => sprites);

pub struct GraphicsContext {
    window: Window,
    graphics: Graphics,
    renderer: Renderer,
    camera: Camera,
    clear_color: NormalizedColor,
    ambient_light: AmbientLight,
}

impl GraphicsContext {
    pub fn new(builder: WindowBuilder) -> Self {
        let (window, mut graphics) = Graphics::new(builder);
        GraphicsContext {
            renderer: Renderer::new(&mut graphics),
            window: window,
            graphics: graphics,
            camera: Camera::new(Point2::new(1., 1.), Vector2::new(2., 2.)),
            clear_color: NormalizedColor::from_srgb([0.1, 0.1, 0.2, 1.0]),
            ambient_light: AmbientLight {
                color: LightColor::from_srgb([0.2, 0.2, 0.2]),
                intensity: 0.5,
            }
        }
    }

    pub fn resize(&mut self) {
        self.graphics.resize(&self.window);
        self.renderer.resize(&mut self.graphics);
        self.camera.update_transform(&self.window);
    }
}

pub trait HasGraphicsContext: Context {
    fn graphics(&mut self) -> &mut GraphicsContext;
}

pub struct GraphicsModule {
    sprites: StorageLock<SpriteStorage>,
}

impl GraphicsModule {
    pub fn new() -> Self {
        GraphicsModule {
            sprites: StorageLock::new(Packed::new())
        }
    }
}

impl<Cx: HasGraphicsContext> Module<Cx> for GraphicsModule {
    fn commit(&mut self, args: &CommitArgs, context: &mut Cx) {
        let graphics = context.graphics();
        let mut sprites = self.sprites.write();
        let mut r = args.update_reader_for::<Sprite>();

        while let Some((entity, template)) = r.next_attach_query() {
            sprites.insert(unsafe { Accessor::new_unchecked(entity) }, template);
        }

        while let Some(entity) = r.next_detach_query() {
            sprites.remove(unsafe { Accessor::new_unchecked(entity) });
        }

        for entity in args.world_removes() {
            sprites.remove(unsafe { Accessor::new_unchecked(entity.id()) });
        }
    }
}

