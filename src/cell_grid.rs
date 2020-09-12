use image::{Rgba, ImageBuffer};
use crate::constants::CHUNK_SIZE;
use piston_window::{G2d, Filter, TextureSettings, Texture, TextureContext, PistonWindow, Transformed};
use std::collections::HashMap;
use vecmath::{Vector2, vec2_add, vec2_cast};
use gfx_device_gl::{Factory, Resources, CommandBuffer, Device};
use piston_window::math::Matrix2d;
use crate::live_cell_states::sand_cell_state::SandCellState;
use itertools::{Itertools, Chunk};
use std::f32::INFINITY;

#[derive(Copy, Clone)]
pub struct LiveCellApi {
    pub tile_position: Vector2<u32>,
    pub chunks: [[Option<[[Cell; CHUNK_SIZE]; CHUNK_SIZE]>; 3]; 3]
}

impl LiveCellApi {
    pub fn get_cell(self, tile_offset: Vector2<i32>) -> Cell {
        let [target_tile_x, target_tile_y] = vec2_add([self.tile_position[0] as i32, self.tile_position[1] as i32], tile_offset);
        
        let chunk_x = ((target_tile_x as f32) / (CHUNK_SIZE as f32)).floor() as i32;
        let chunk_y = ((target_tile_y as f32) / (CHUNK_SIZE as f32)).floor() as i32;

        let relative_target_tile_x = (target_tile_x + (chunk_x * (CHUNK_SIZE as i32))) as u32;
        let relative_target_tile_y = (target_tile_y + (chunk_y * (CHUNK_SIZE as i32))) as u32;

        let chunk = self.chunks[(chunk_x + 1) as usize][(chunk_y + 1) as usize];
        
        if let Some(chunk) = chunk {
            return chunk[relative_target_tile_x as usize][relative_target_tile_y as usize]
        }
        
        return Cell::Static(StaticCell::new(Rgba([0, 0, 0, 0])))
    }
    
    pub fn is_empty(self, tile_offset: Vector2<i32>) -> bool {
        self.get_cell(tile_offset) == Cell::Empty
    }
}

#[derive(Copy, Clone)]
pub struct LiveCellInstructions {
    pub move_instruction: Option<LiveCellMoveInstruction>,
    pub new_color_instruction: Option<Rgba<u8>>
}

impl LiveCellInstructions {
    pub fn new() -> Self {
        Self {
            move_instruction: None,
            new_color_instruction: None
        }
    }
    
    pub fn with_move_instruction(&mut self, move_instruction: Option<LiveCellMoveInstruction>) -> &mut Self {
        self.move_instruction = move_instruction;
        self
    }

    pub fn with_new_color_instruction(&mut self, new_color_instruction: Option<Rgba<u8>>) -> &mut Self {
        self.new_color_instruction = new_color_instruction;
        self
    }
}

#[derive(Copy, Clone)]
pub enum LiveCellMoveInstruction {
    Replace(Vector2<i32>),
    Switch(Vector2<i32>),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticCell {
    color: Rgba<u8>
}

impl StaticCell {
    pub fn new(color: Rgba<u8>) -> Self {
        Self {
            color
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LiveCellState {
    SandState(SandCellState)
}

impl LiveCellState {
    pub fn update(&mut self, api: LiveCellApi) -> LiveCellInstructions {
        match self {
            LiveCellState::SandState(sand_cell_state) => sand_cell_state.update(api)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LiveCell {
    pub state: LiveCellState,
    pub last_frame_updated: u64
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Static(StaticCell),
    Live(LiveCell)
}

impl Cell {
    fn get_color(&self) -> Rgba<u8> {
        match self {
            Cell::Empty => Rgba([0, 0, 0, 0]),
            Cell::Static(static_cell) => static_cell.color,
            Cell::Live(live_cell) => Rgba([0, 0, 0, 255])
        }
    }
}

pub struct CellChunk {
    cells: [[Cell; CHUNK_SIZE]; CHUNK_SIZE],
    canvas: ImageBuffer<Rgba<u8>, Vec<u8>>,
    texture_context: TextureContext<Factory, Resources, CommandBuffer>,
    texture: Texture<Resources>,
    needs_redraw: bool
}

impl CellChunk {
    pub fn new(window: &mut PistonWindow) -> Self {
        let mut cells = [[Cell::Empty; CHUNK_SIZE]; CHUNK_SIZE];

        let mut canvas = ImageBuffer::new(CHUNK_SIZE as u32, CHUNK_SIZE as u32);
        
        let mut texture_context = TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into()
        };

        let mut texture = Texture::from_image(
            &mut texture_context,
            &canvas,
            &TextureSettings::new().min(Filter::Nearest).mag(Filter::Nearest)
        ).unwrap();
        
        Self {
            cells,
            canvas,
            texture_context,
            texture,
            needs_redraw: false
        }
    }
    
    pub fn get_cell(&self, cell_position: Vector2<u32>) -> Cell {
        self.cells[cell_position[0] as usize][cell_position[1] as usize]
    }
    
    pub fn set_cell(&mut self, cell_position: Vector2<u32>, cell: Cell) {
        self.cells[cell_position[0] as usize][cell_position[1] as usize] = cell;
        self.canvas.put_pixel(cell_position[0], cell_position[1], cell.get_color());
        
        self.needs_redraw = true;
    }
    
    pub fn render(&mut self, graphics: &mut G2d, device: &mut Device, transform: Matrix2d) {
        if self.needs_redraw {
            self.texture.update(&mut self.texture_context, &self.canvas).unwrap();
            self.texture_context.encoder.flush(device);
            
            self.needs_redraw = false;
        }
        
        piston_window::image(&self.texture, transform, graphics);
    }
    
    pub fn update(&mut self, current_frame: u64, neighbor_chunks: [Option<&mut CellChunk>; 8]) {
        let is_even = current_frame % 2 == 0;

        // let chunks: [[Option<[[Cell; CHUNK_SIZE]; CHUNK_SIZE]>; 3]; 3] = [
        //     [neighbor_chunks[0].map(|a| a.cells), neighbor_chunks[1].map(|a| a.cells), neighbor_chunks[2].map(|a| a.cells)],
        //     [neighbor_chunks[3].map(|a| a.cells), Some(self.cells), neighbor_chunks[4].map(|a| a.cells)],
        //     [neighbor_chunks[5].map(|a| a.cells), neighbor_chunks[6].map(|a| a.cells), neighbor_chunks[7].map(|a| a.cells)]
        // ];
        // 
        // for x in if is_even { (CHUNK_SIZE - 1)..=1 } else { 0..=(CHUNK_SIZE - 1) } {
        //     for y in (CHUNK_SIZE - 1)..=0 {
        //         let cell = &mut self.cells[x][y];
        //         
        //         let tile_position = [x as i32, y as i32];
        //         
        //         if let Cell::Live(live_cell) = cell {
        //             if live_cell.last_frame_updated == current_frame {
        //                 continue;
        //             }
        //             
        //             let live_cell_instructions = live_cell.state.update(LiveCellApi {
        //                 tile_position: [x as u32, y as u32],
        //                 chunks
        //             });
        //             
        //             let move_instruction = live_cell_instructions.move_instruction;
        //             let new_color_instruction = live_cell_instructions.new_color_instruction;
        //             
        //             if let Some(move_instruction) = move_instruction {
        //                 match move_instruction {
        //                     LiveCellMoveInstruction::Replace(tile_offset) => {
        //                         let [target_tile_x, target_tile_y] = vec2_add(tile_position, tile_offset);
        // 
        //                         let chunk_x = ((target_tile_x as f32) / (CHUNK_SIZE as f32)).floor() as i32;
        //                         let chunk_y = ((target_tile_y as f32) / (CHUNK_SIZE as f32)).floor() as i32;
        // 
        //                         let relative_target_tile_x = (target_tile_x + (chunk_x * (CHUNK_SIZE as i32))) as u32;
        //                         let relative_target_tile_y = (target_tile_y + (chunk_y * (CHUNK_SIZE as i32))) as u32;
        // 
        //                         
        //                         
        //                         // let chunk = self.chunks[(chunk_x + 1) as usize][(chunk_y + 1) as usize];
        //                         // 
        //                         // if let Some(chunk) = chunk {
        //                         //     return chunk[relative_target_tile_x as usize][relative_target_tile_y as usize]
        //                         // }
        // 
        //                         // return Cell::Static(StaticCell::new(Rgba([0, 0, 0, 0])))
        //                         // if let Some(a) = neighbor_chunks[0] {
        //                         //     a.cells[0][0] = Cell::Empty;
        //                         // }
        //                     },
        //                     
        //                     LiveCellMoveInstruction::Switch(tile_offset) => {
        //                         
        //                     }
        //                 }
        //             }
        //         }
        //     }  
        // }
    }
}

pub struct CellGrid {
    chunks: HashMap<Vector2<i32>, CellChunk>
}

impl CellGrid {
    pub fn new(window: &mut PistonWindow) -> Self {
        let mut chunks = HashMap::new();

        for column in (0..5) {
            for row in (0..3) {
                chunks.insert([column, row], CellChunk::new(window));
            }
        }

        Self {
            chunks
        }
    }
    
    pub fn set_cell(&mut self, grid_position: Vector2<i32>, cell: Cell) {
        let chunk_position = [(grid_position[0] as f32) / (CHUNK_SIZE as f32), (grid_position[1] as f32) / (CHUNK_SIZE as f32)];
        let chunk_position = [chunk_position[0].floor() as i32, chunk_position[1].floor() as i32];
        
        if let Some(chunk) = self.chunks.get_mut(&chunk_position) {
            let cell_position = [
                (grid_position[0] - chunk_position[0] * (CHUNK_SIZE as i32)) as u32,
                (grid_position[1] - chunk_position[1] * (CHUNK_SIZE as i32)) as u32,
            ];
            
            chunk.set_cell(cell_position, cell);
        }
    }

    pub fn render(&mut self, graphics: &mut G2d, device: &mut Device, transform: Matrix2d) {
        for (chunk_position, chunk) in &mut self.chunks {
            let [chunk_x, chunk_y] = chunk_position;

            let chunk_transform = transform.trans((chunk_x * (CHUNK_SIZE as i32)) as f64, (chunk_y * (CHUNK_SIZE as i32)) as f64);

            chunk.render(graphics, device, chunk_transform);
        }
    }
    
    pub fn update(&mut self, current_frame: u64) {
        let even_frame = current_frame % 2 == 0;

        fn get_chunk_at_offset(chunks: &mut HashMap<Vector2<i32>, CellChunk>, chunk_position: Vector2<i32>, offset: Vector2<i32>) -> Option<&mut CellChunk> {
            let a = vec2_add(chunk_position, offset);
            
            chunks.get_mut(&a)
        }

        let mut min_x_chunk_position = i32::MAX;
        let mut min_y_chunk_position = i32::MAX;
        let mut max_x_chunk_position = i32::MIN;
        let mut max_y_chunk_position = i32::MIN;

        for [chunk_position_x, chunk_position_y] in self.chunks.keys() {
            if *chunk_position_x < min_x_chunk_position { min_x_chunk_position = *chunk_position_x }
            if *chunk_position_x > max_x_chunk_position { max_x_chunk_position = *chunk_position_x }
            if *chunk_position_y < min_y_chunk_position { min_y_chunk_position = *chunk_position_y }
            if *chunk_position_y > max_y_chunk_position { max_y_chunk_position = *chunk_position_y }
        }

        let x_range = if even_frame { min_x_chunk_position .. max_x_chunk_position } else { max_x_chunk_position .. min_x_chunk_position };
        
        for chunk_position_x in x_range {
            for chunk_position_y in max_y_chunk_position .. min_y_chunk_position {
                let chunk_position = [chunk_position_x, chunk_position_y];
                
                let chunk = self.chunks.get_mut(&chunk_position);
                
                if let Some(chunk) = chunk {
                    chunk.update(current_frame, [
                        get_chunk_at_offset(&mut self.chunks, chunk_position, [-1, -1]),
                        get_chunk_at_offset(&mut self.chunks, chunk_position, [-1, -1]),
                        get_chunk_at_offset(&mut self.chunks, chunk_position, [-1, -1]),
                        get_chunk_at_offset(&mut self.chunks, chunk_position, [-1, -1]),
                        get_chunk_at_offset(&mut self.chunks, chunk_position, [-1, -1]),
                        get_chunk_at_offset(&mut self.chunks, chunk_position, [-1, -1]),
                        get_chunk_at_offset(&mut self.chunks, chunk_position, [-1, -1]),
                        get_chunk_at_offset(&mut self.chunks, chunk_position, [-1, -1])
                    ])
                }
            }
        }
    }
}