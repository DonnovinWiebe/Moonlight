use iced::{Element, widget::{column, image}};
use materialui::{components::{Orientations, Spacing, TextSizes, spacer, ui_string}, material::MaterialColors};

use crate::state::{app::App, signal::Signal};

/// A basic image view for displaying the current image for the current `Node` in the current `Tree`.
#[must_use]
pub fn image_viewer<'a>(app: &'a App) -> Element<'a, Signal> {
    let current_tree_result = app.get_current_tree();
    if current_tree_result.is_none() { return ui_string(app, "Start a project!", TextSizes::LargeHeading, MaterialColors::StrongText) }
    let tree = current_tree_result.expect("This was just checked for someness.");
    let handle_result = tree.get_current_handle();
    if handle_result.is_fail() { return ui_string(app, "Could not display current image!", TextSizes::LargeHeading, MaterialColors::StrongText) }
    let handle = handle_result.wont_fail("This is past an is_fail() guard clause.", "viewport - image_viewer()");
    
    column![
        image(&handle),
        spacer(Orientations::Vertical, Spacing::Small),
        ui_string(app, "Current Node", TextSizes::Body, MaterialColors::WeakText),
    ]
    .spacing(0)
    .into()
}