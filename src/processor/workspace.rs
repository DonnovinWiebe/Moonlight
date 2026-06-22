use std::path::PathBuf;

use image::{ImageBuffer, Rgba};

use crate::processor::node::Pool;

pub struct Workspace {
    source_path: Option<PathBuf>,
    current_image: Option<ImageBuffer<Rgba<f32>, Vec<f32>>>,
    pool: Pool,
}