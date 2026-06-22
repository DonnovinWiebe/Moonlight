use image::{ImageBuffer, Rgba};

#[derive(Debug, Copy, Clone)]
pub enum Operation {
    Rotate(f64), // in degrees
    Crop(Point, Point), // bounding box start, bounding box size
    Exposure(f64),
    Brightness(f64),
    Contrast(f64),
    Whites(f64), // lightens the whole image, but focuses on the extremes
    Blacks(f64), // darkens the whole image, but focuses on the extremes
    Highlights(f64), // lightens mid-light regions
    Shadows(f64), // darkens mid-dark regions
    WhitePoint(f64), // adjusts where colors are clipped to max white and regradients image
    BlackPoint(f64), // adjusts where colors are clipped to max black and regradients the image
}
impl Operation {
    pub fn apply_to(&self, image: &mut ImageBuffer<Rgba<f32>, Vec<f32>>) {
        todo!()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}
impl Point {
    pub fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}