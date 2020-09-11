use image::{Rgba, ImageBuffer};
use crate::constants::CHUNK_SIZE;
use piston_window::{G2d, Filter, TextureSettings, Texture, TextureContext, PistonWindow};
use std::collections::HashMap;
use vecmath::Vector2;
use gfx_device_gl::{Factory, Resources, CommandBuffer, Device};
use piston_window::math::Matrix2d;

#[derive(Clone, Copy)]
pub struct StaticCell {
    color: Rgba<u8>
}

#[derive(Clone, Copy)]
pub enum LiveCellState {
}

#[derive(Clone, Copy)]
pub struct LiveCell {
    pub state: LiveCellState,
    pub last_frame_updated: u64
}

#[derive(Clone, Copy)]
pub enum Cell {
    Empty,
    Static(StaticCell),
    Live(LiveCell)
}


pub struct CellChunk {
    cells: [[Cell; CHUNK_SIZE]; CHUNK_SIZE],
    canvas: ImageBuffer<Rgba<u8>, Vec<u8>>,
    texture_context: TextureContext<Factory, Resources, CommandBuffer>,
    texture: Texture<Resources>
}

impl CellChunk {
    pub fn new(window: &mut PistonWindow) -> Self {
        let mut cells = [[Cell::Empty; CHUNK_SIZE]; CHUNK_SIZE];

        let mut canvas = ImageBuffer::new(CHUNK_SIZE as u32, CHUNK_SIZE as u32);

        canvas.put_pixel(0, 0, Rgba([0, 0, 0, 255]));

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
            texture
        }
    }

    pub fn render(&mut self, graphics: &mut G2d, device: &mut Device, transform: Matrix2d) {
        self.texture.update(&mut self.texture_context, &self.canvas).unwrap();
        // Update texture before rendering.
        self.texture_context.encoder.flush(device);

        piston_window::image(&self.texture, transform, graphics);
    }
}

pub struct CellGrid {
    chunks: HashMap<Vector2<i32>, CellChunk>
}

impl CellGrid {
    pub fn new(window: &mut PistonWindow) -> Self {
        let mut chunks = HashMap::new();

        for column in (0..3) {
            for row in (0..3) {
                chunks.insert([column, row], CellChunk::new(window));
            }
        }

        Self {
            chunks
        }
    }

    pub fn render(&mut self, graphics: &mut G2d, device: &mut Device, transform: Matrix2d) {
        for (chunk_position, chunk) in &mut self.chunks {
            let [chunk_x, chunk_y] = chunk_position;

            let chunk_transform = transform.trans((chunk_x * (CHUNK_SIZE as i32)) as f64, (chunk_y * (CHUNK_SIZE as i32)) as f64);

            chunk.render(graphics, device, chunk_transform);
        }
    }
}