mod constants;
mod cell_grid;
mod helpers;
mod live_cell_states;

use vecmath::{Vector2, vec2_mul, vec2_scale, vec2_sub};
use piston_window::{clear, WindowSettings, OpenGL, PistonWindow, RenderEvent, MouseCursorEvent, Transformed, Filter, math, G2d, PressEvent, ReleaseEvent, Button, MouseButton, UpdateEvent};
use crate::cell_grid::{CellGrid, Cell, StaticCell};
use crate::constants::CHUNK_SIZE;
use crate::helpers::vec2::vec2_floor;
use vecmath::traits::Cast;
use image::Rgba;

fn main() {
    const CANVAS_SCALE: f32 = 1.0 / 8.0;
    
    let opengl = OpenGL::V3_2;
    let (width, height) = (800, 600);
    
    let mut window: PistonWindow =
        WindowSettings::new("piston: paint", (width, height))
            .resizable(false)
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
    
    let camera_transform = math::translate([0.0, 0.0])
        .scale(1.0 / (CANVAS_SCALE as f64), 1.0 / (CANVAS_SCALE as f64));

    let mut cell_grid = CellGrid::new(&mut window);
    let mut holding_left_click: bool = false;
    let mut current_frame: u64 = 0;

    while let Some(event) = window.next() {
        if let Some(button) = event.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                holding_left_click = true;
            }
        };
        
        if let Some(button) = event.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                holding_left_click = false;
            }
        };
        
        if holding_left_click {
            if let Some(cursor_position) = event.mouse_cursor_args() {
                let world_cursor_position = vec2_floor(vec2_scale(cursor_position, CANVAS_SCALE as f64));
                
                cell_grid.set_cell(world_cursor_position, Cell::Static(StaticCell::new(Rgba([0, 255, 0, 255]))));
            }
        }
        
        if let Some(update_args) = event.update_args() {
            current_frame += 1;
            
            println!("{:?}", 1.0 / update_args.dt);
            
            cell_grid.update(current_frame);
        }
        
        if let Some(_) = event.render_args() {
            window.draw_2d(&event, |context, graphics, device| {
                let transform = context.transform
                    .append_transform(camera_transform);
                
                clear([0.91, 0.90, 0.85, 1.0], graphics);

                cell_grid.render(graphics, device, transform);
            });
        }
    }
}
