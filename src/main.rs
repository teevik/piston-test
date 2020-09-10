// extern crate image;

use image::{ImageBuffer, Rgba};
use vecmath::{vec2_scale, Vector2, vec2_cast};
use piston_window::{MouseButton, Button, clear, TextureSettings, Texture, TextureContext, WindowSettings, OpenGL, PistonWindow, RenderEvent, PressEvent, MouseCursorEvent, ReleaseEvent, Transformed, Filter, math};
use image::math::utils::clamp;
use piston_window::math::abs_transform;
use gfx::format::ChannelSource::X;
use legion::World;

fn main() {
    const CANVAS_SCALE: f32 = 1.0 / 8.0;
    
    let opengl = OpenGL::V3_2;
    let (width, height) = (800, 600);
    
    // let canvas_width = (width as f32 * CANVAS_SCALE / 8.0) as u32;
    // let canvas_height = (height as f32 * CANVAS_SCALE / 8.0) as u32;

    let chunk_size = 16;
    let chunk_amounts = [7, 4];
    
    let world = World::default();

    let mut window: PistonWindow =
        WindowSettings::new("piston: paint", (width, height))
            .resizable(false)
            .exit_on_esc(true)
            .graphics_api(opengl)
            .build()
            .unwrap();

    let mut canvas = ImageBuffer::new(chunk_size, chunk_size);
    let mut draw = false;
    let mut cursor_position: Option<Vector2<f64>> = None;
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    let mut texture = Texture::from_image(
        &mut texture_context,
        &canvas,
        &TextureSettings::new().min(Filter::Nearest).mag(Filter::Nearest)
    ).unwrap();
    
    let mut camera_transform = math::translate([0.0, 0.0]);
    
    while let Some(event) = window.next() {
        
        if let Some(_) = event.render_args() {
            // Example of moving the camera
            // camera_transform = camera_transform.trans(0.001, 0.0);
            // camera_transform = camera_transform.scale(0.999, 0.999);
            
            texture.update(&mut texture_context, &canvas).unwrap();
            
            window.draw_2d(&event, |context, graphics, device| {
                // Update texture before rendering.
                texture_context.encoder.flush(device);
                
                let transform = context.transform
                    .prepend_transform(camera_transform)
                    .scale(1.0 / (CANVAS_SCALE as f64), 1.0 / (CANVAS_SCALE as f64));
                
                clear([1.0; 4], graphics);
                
                for chunk_x in 0 .. chunk_amounts[0] {
                    for chunk_y in 0 .. chunk_amounts[1] {
                        piston_window::image(&texture, transform.trans((chunk_x * chunk_size) as f64, (chunk_y * chunk_size) as f64), graphics);
                    }
                }
            });
        }
        
        if let Some(button) = event.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                draw = true;
            }
        };
        if let Some(button) = event.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                draw = false;
            }
        };
        
        if let Some(new_cursor_position) = event.mouse_cursor_args() {
            cursor_position = Option::Some(new_cursor_position);
        };

        if draw {
            if let Some(cursor_position) = cursor_position {
                let canvas_cursor_position = vec2_scale(cursor_position, CANVAS_SCALE as f64);
                let [x, y] = canvas_cursor_position;
        
                let clamped_x = clamp(x as u32, 0, chunk_size - 1);
                let clamped_y = clamp(y as u32, 0, chunk_size - 1);
        
                canvas.put_pixel(clamped_x, clamped_y, Rgba([0, 0, 0, 255]));
            }
        }
    }
}
