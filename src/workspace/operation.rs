use std::str::EscapeUnicode;

use iced::{Pixels, Point, alignment::{self, Vertical}, widget::{canvas::{self, Text}, text::Alignment}};
use image::{ImageBuffer, Rgba};
use materialui::{components::{TextSizes, ThemeProvider}, material::{Depths, MaterialColors, MaterialThemes, Materials}};
use serde::{Deserialize, Serialize};

use crate::{FA_SOLID, state::app::App, workspace::node::WorkingImage};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Only used for the root `Node` of a `Tree`.
    Root,
    /// Rotates an image in degrees.
    Rotate(f64),
    /// Crops an image from the top left corner `Point` at the given size (2nd `Point`).
    Crop(Position, Position),
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
    /// Returns a new `WorkingImage` with the given `Operation` applied to the given `WorkingImage`.
    #[must_use]
    pub fn applied_to(&self, image: &WorkingImage) -> WorkingImage {
        image.clone()
    }

    /// Returns an icon for the given `Operation` that can be used with Iced's `canvas`.
    #[must_use]
    pub fn canvas_icon(&self, theme: MaterialThemes) -> Text {
        match self {
            Operation::Root =>          Operation::canvas_icon_base(theme, "\u{f192}".to_string()), // circle-dot
            Operation::Rotate(_) =>     Operation::canvas_icon_base(theme, "\u{f2f1}".to_string()), // rotate
            Operation::Crop(_, _) =>    Operation::canvas_icon_base(theme, "\u{f565}".to_string()), // crop-simple
            Operation::Exposure(_) =>   Operation::canvas_icon_base(theme, "\u{f185}".to_string()), // sun
            Operation::Brightness(_) => Operation::canvas_icon_base(theme, "\u{f0eb}".to_string()), // lightbulb
            Operation::Contrast(_) =>   Operation::canvas_icon_base(theme, "\u{f042}".to_string()), // circle/half/stroke
            Operation::Whites(_) =>     Operation::canvas_icon_base(theme, "\u{57}"  .to_string()), // w
            Operation::Blacks(_) =>     Operation::canvas_icon_base(theme, "\u{42}"  .to_string()), // b
            Operation::Highlights(_) => Operation::canvas_icon_base(theme, "\u{e52f}".to_string()), // mountain-sun
            Operation::Shadows(_) =>    Operation::canvas_icon_base(theme, "\u{f186}".to_string()), // moon
            Operation::WhitePoint(_) => Operation::canvas_icon_base(theme, "\u{f111}".to_string()), // circle
            Operation::BlackPoint(_) => Operation::canvas_icon_base(theme, "\u{f192}".to_string()), // circle-dot
        }
    }

    /// A helper function for getting `canvas` compatible icons.
    #[must_use]
    fn canvas_icon_base(theme: MaterialThemes, code: String) -> Text {
        Text {
            content: code,
            font: FA_SOLID,
            size: Pixels(TextSizes::Interactable.size()),
            position: Point::new(0.0, 0.0),
            color: MaterialColors::StrongText.materialized(Materials::Plastic, Depths::Flat, false, theme),
            align_x: Alignment::Center,
            align_y: Vertical::Center,
            ..canvas::Text::default()
        }
    }
}



/// Defines a 2d position in an image in pixels.
/// This can also be used to define a size (as in Operation::Crop).
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}
impl Position {
    /// Returns a new `Position`.
    #[must_use]
    pub fn new(x: u32, y: u32) -> Position {
        Position { x, y }
    }
}