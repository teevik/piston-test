use image::Rgba;
use crate::cell_grid::{LiveCellInstructions, LiveCellApi, Cell, LiveCellMoveInstruction};
use rand::random;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SandCellState {
    pub color: Rgba<u8>
}

impl SandCellState {
    pub fn new() -> Self {
        // let color: Lch = SAND_GRADIENT.get(thread_rng().gen_range(0.0, 1.0));
        // let color: Rgb = color.into();

        Self {
            color: Rgba([0, 0, 0, 255])
        }
    }

    pub fn update(&mut self, api: LiveCellApi) -> LiveCellInstructions {
        let cell_below = api.get_cell([0, 1]);

        let mut a: Option<LiveCellMoveInstruction> = None;
        
        if cell_below == Cell::Empty {
            a = Some(LiveCellMoveInstruction::Replace([0, 1]));
        } else {
            let random_direction = if random() { -1 } else { 1 };
            
            if api.get_cell([random_direction, 1]) == Cell::Empty {
                a = Some(LiveCellMoveInstruction::Replace([random_direction, 1]));
            }
        }

        *LiveCellInstructions::new()
            .with_move_instruction(a)
    }
}
