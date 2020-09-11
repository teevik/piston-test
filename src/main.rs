mod constants;
mod cell_grid;

use vecmath::{Vector2};
use piston_window::{clear, WindowSettings, OpenGL, PistonWindow, RenderEvent, MouseCursorEvent, Transformed, Filter, math, G2d};
use crate::cell_grid::CellGrid;

fn main() {
    const CANVAS_SCALE: f32 = 1.0 / 8.0;
    
    let opengl = OpenGL::V3_2;
    let (width, height) = (800, 600);
    
    let mut window: PistonWindow =
        WindowSettings::new("piston: paint", (width, height))
            .resizable(false)
            .exit_on_esc(true)
            .graphics_api(opengl)
            .build()
            .unwrap();
    
    let camera_transform = math::translate([0.0, 0.0]);
    
    let mut cell_grid = CellGrid::new(&mut window);
    let mut cursor_position: Option<Vector2<f64>> = None;
    
    while let Some(event) = window.next() {
        
        if let Some(_) = event.render_args() {
            
            window.draw_2d(&event, |context, graphics, device| {
                let transform = context.transform
                    .prepend_transform(camera_transform)
                    .scale(1.0 / (CANVAS_SCALE as f64), 1.0 / (CANVAS_SCALE as f64));
                
                if let Some(cursor_position) = cursor_position {
                    let transformed_cursor_position = math::transform_pos(transform, cursor_position);
                    println!("{:?}", transformed_cursor_position);
                }
                
                clear([0.91, 0.90, 0.85, 1.0], graphics);

                cell_grid.render(graphics, device, transform);
            });
        }
        
        if let Some(new_cursor_position) = event.mouse_cursor_args() {
            cursor_position = Some(new_cursor_position);
        };
    }
}
