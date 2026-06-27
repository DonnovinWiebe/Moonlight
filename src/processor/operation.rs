use image::{ImageBuffer, Rgba};

#[derive(Debug, Copy, Clone)]
pub enum Operation {
    /// Only used for the root `Node` of a `Tree`.
    Root,
    /// Rotates an image in degrees.
    Rotate(f64),
    /// Crops an image from the top left corner `Point` at the given size (2nd `Point`).
    Crop(Point, Point),
    /// Adjusts the exposure of the image.
    Exposure(f64),
    /// Adjusts the overall brightness of the image.
    Brightness(f64),
    /// Adjusts the overall contrast of the image.
    Contrast(f64),
    /// Lightens the highlights of the image.
    Whites(f64),
    /// Darkens the shadows of the image.
    Blacks(f64),
    /// Pulls down the highlights of the image.
    Highlights(f64),
    /// Lifts the shadows of the image.
    Shadows(f64),
    /// Adjusts the brightness that is considered maximum white (clipping point).
    WhitePoint(f64),
    /// Adjusts the brightness that is considered maximum black (clipping point).
    BlackPoint(f64),
}
impl Operation {
    pub fn apply_to(&self, image: &mut ImageBuffer<Rgba<f32>, Vec<f32>>) {
        todo!()
    }
}

/// Defines a 2d point in an image in pixels.
/// This can also be used to define a size (as in Operation::Crop).
#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}
impl Point {
    /// Returns a new `Point`.
    #[must_use]
    pub fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}