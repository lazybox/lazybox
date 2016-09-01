extern crate lazybox_graphics as graphics;
extern crate lazybox_frameclock as frameclock;
#[macro_use] extern crate conrod;
extern crate glutin;
extern crate cgmath;
extern crate rayon;

use graphics::Graphics;
use graphics::combined::conrod::Renderer;
use graphics::types::ColorFormat;
use conrod::widget;
use glutin::{WindowBuilder, Event};
use frameclock::*;

fn main() {
    let builder = WindowBuilder::new()
        .with_title("Conrod".to_string())
        .with_dimensions(512, 512)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)));

    let (window, mut graphics) = Graphics::new(builder);
    let mut renderer = Renderer::new(window.hidpi_factor(), &mut graphics);

    let mut ui = conrod::UiBuilder::new().build();
    ui.fonts.insert_from_file("resources/fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    widget_ids!{
        CANVAS,
        BUTTON,
        DEMO_TEXT,
        FPS_TEXT,
        LINE,
        IMAGE_RED,
        IMAGE_GREEN,
        IMAGE_BLUE,
    }

    let texture = graphics.load_texture_from_image::<ColorFormat>("resources/cloud.png");
    let image_map = image_map! {
        (IMAGE_RED, texture.clone()),
        (IMAGE_GREEN, texture.clone()),
        (IMAGE_BLUE, texture),
    };

    let mut frameclock = FrameClock::start(1.);
    let mut fps_counter = FpsCounter::new(1.);
    let mut mspf = 0.0;
    let mut fps = 0;

    'main: loop {
        let (w, h) = window.get_inner_size_pixels().unwrap();
        let dpi_factor = window.hidpi_factor() as conrod::Scalar;
        let delta_time = frameclock.reset();
        if let Some(fps_sample) = fps_counter.update(delta_time) {
            mspf = 1000. / fps_sample;
            fps = fps_sample as u32;
        }

        for event in window.poll_events() {
            match event {
                Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                Event::Closed => break 'main,
                _ => {},
            }

            let (w, h) = (w as conrod::Scalar, h as conrod::Scalar);
            if let Some(event) = conrod::backend::glutin::convert(event.clone(), w, h, dpi_factor) {
                ui.handle_event(event);
            }
        }

        ui.handle_event(conrod::event::render(delta_time, w, h, dpi_factor));
        {
            let mut ui = &mut ui.set_widgets();
            use conrod::{Colorable, Positionable, Sizeable, Widget};

            widget::Canvas::new().color(conrod::color::DARK_CHARCOAL).set(CANVAS, ui);

            let demo_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. \
                Nulla finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. \
                Interdum et malesuada fames ac ante ipsum primis in faucibus. \
                Cras rhoncus nisi nec dolor bibendum pellentesque. \
                Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
                Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. \
                Nam magna est, efficitur suscipit dolor eu, consectetur consectetur urna.";

            widget::Text::new(demo_text)
                .middle_of(CANVAS)
                .w_of(CANVAS)
                .font_size(20)
                .color(conrod::color::BLACK)
                .align_text_middle()
                .set(DEMO_TEXT, ui);

            let fps_text = &format!("{:.2} ms/frame - {} fps", mspf, fps);

            widget::Text::new(fps_text)
                .top_left_with_margin_on(CANVAS, 6.)
                .wh_of(CANVAS)
                .font_size(14)
                .color(conrod::color::PURPLE)
                .align_text_left()
                .set(FPS_TEXT, ui);

            let style = widget::line::Style {
                maybe_pattern: None,
                maybe_color: None,
                maybe_thickness: Some(20.),
                maybe_cap: Some(widget::line::Cap::Round),
            };

            widget::Line::abs_styled([0., 0.], [120., 0.], style).set(LINE, ui);

            let image = widget::Image::new().w_h(64., 64.);
            image.clone()
                .color(Some(conrod::color::RED))
                .bottom_left_with_margin_on(CANVAS, 12.)
                .set(IMAGE_RED, ui);
            image.clone()
                .color(Some(conrod::color::GREEN))
                .right_from(IMAGE_RED, 12.)
                .set(IMAGE_GREEN, ui);
            image.clone()
                .color(Some(conrod::color::BLUE))
                .right_from(IMAGE_GREEN, 12.)
                .set(IMAGE_BLUE, ui);
        }

        let mut frame = graphics.draw();
        ui.draw_if_changed().map(|ps| renderer.changed(ps, &image_map, &mut frame));
        renderer.render(&mut frame);
        frame.present(&window);
    }
}
