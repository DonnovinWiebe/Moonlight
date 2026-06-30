use iced::Point;
use iced::widget::canvas::{Fill, Frame, Path};
use image::{ImageBuffer, Rgba};
use materialui::components::{TextSizes, ThemeProvider};
use materialui::material::{Depths, MaterialColors, Materials};
use palette::{LinSrgba, Srgba};
use palette::FromColor;
use schrod::Schrod::{self, Pass};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::app::App;
use crate::workspace::operation::Operation;

pub type Buffer = ImageBuffer<Rgba<f32>, Vec<f32>>;

/// Lists the two directions that can be traversed in the `Node` `Tree`.
pub enum Direction {
    /// Moving upwards toward the `root`.
    UpStream,
    /// Moving downwards towards the tips.
    DownStream,
}



/// Holds two copies of the image being edited.
/// One copy is for applying operations (`linear_image`) and the other is for displaying (`srgb_image`).
#[derive(Debug, Clone)]
pub struct WorkingImage {
    /// Used for applying operations.
    linear_image: Buffer,
    /// Used for displaying.
    srgb_image: Buffer,
}
impl WorkingImage {
    // general tools
    /// Converts an SRGB image to a linear image.
    #[must_use]
    pub fn srgb_to_linear(srgb_image: &Buffer) -> Buffer {
        let (w, h) = srgb_image.dimensions();
        let mut linear_image = ImageBuffer::new(w, h);
        for (x, y, px) in srgb_image.enumerate_pixels() {
            let srgba_pixel = Srgba::new(px[0], px[1], px[2], px[3]);
            let linear_pixel = LinSrgba::from_color(srgba_pixel);
            linear_image.put_pixel(x, y, Rgba([linear_pixel.red, linear_pixel.green, linear_pixel.blue, linear_pixel.alpha]));
        }
        linear_image
    }

    /// Converts a linear image to an SRGB image.
    #[must_use]
    pub fn linear_to_srgb(linear_image: &Buffer) -> Buffer {
        let (w, h) = linear_image.dimensions();
        let mut srgb_image = ImageBuffer::new(w, h);
        for (x, y, px) in linear_image.enumerate_pixels() {
            let linear_pixel = LinSrgba::new(px[0], px[1], px[2], px[3]);
            let srgba_pixel = Srgba::from_color(linear_pixel);
            srgb_image.put_pixel(x, y, Rgba([srgba_pixel.red, srgba_pixel.green, srgba_pixel.blue, srgba_pixel.alpha]));
        }
        srgb_image
    }



    // initializing
    /// Returns a new `WorkingImage` from a linear image.
    #[must_use]
    pub fn from_linear(linear_image: Buffer) -> WorkingImage {
        let srgb_image = WorkingImage::linear_to_srgb(&linear_image);
        WorkingImage { linear_image, srgb_image }
    }

    /// Returns a new `WorkingImage` from as srgb image.
    #[must_use]
    pub fn from_srgb(srgb_image: Buffer) -> WorkingImage {
        let linear_image = WorkingImage::srgb_to_linear(&srgb_image);
        WorkingImage { linear_image, srgb_image }
    }


    // basic getters
    /// Returns a reference to the `linear_image`.
    #[must_use]
    pub fn get_linear(&self) -> &Buffer {
        &self.linear_image
    }

    /// Returns a reference to the `srgb_image`.
    #[must_use]
    pub fn get_srgb(&self) -> &Buffer {
        &self.srgb_image
    }



    // basic editing
    /// Sets a new `linear_image` and updates the `srgb_image` based on it.
    pub fn set_linear(&mut self, new_linear_image: Buffer) {
        let new_srgb_image = WorkingImage::linear_to_srgb(&new_linear_image);
        self.linear_image = new_linear_image;
        self.srgb_image = new_srgb_image;
    }
}



/// Holds information for an individual `Operation` or step in the edit `Tree`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// The unique id for this `Node` so it can be identified.
    id: Uuid,
    /// Identifies which branch this `Node` lives in.
    branch_id: Uuid,
    /// holds the parent of this `Node`.
    /// The `root` in the `Tree` will have itself as a parent.
    parent_id: Uuid,
    /// Holds the children of this `Node`.
    children_ids: Vec<Uuid>,
    /// What edit operation is being performed by this `Node`.
    operation: Operation,
    /// A chached image of the project up to this point in the `Tree`.
    #[serde(skip)]
    image: Option<WorkingImage>,
}
impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.id == other.id
    }
}
impl Node {
    // initializing
    /// Creates a new `Node`.
    #[must_use]
    pub fn new(parent_id: Uuid, branch_id: Uuid, operation: Operation) -> Node {
        Node {
            id: Uuid::now_v7(),
            branch_id,
            parent_id,
            children_ids: Vec::new(),
            operation,
            image: None,
        }
    }
    
    /// Creates a new `root` `Node`.
    /// This is different function since it needs to reference itself as a parent.
    #[must_use]
    pub fn new_root() -> Node {
        let id = Uuid::now_v7();
        let branch_id = Uuid::now_v7();
        
        Node {
            id,
            branch_id,
            parent_id: id,
            children_ids: Vec::new(),
            operation: Operation::Root,
            image: None,
        }
    }


    
    // basic getters
    /// Checks if the given `Node` is the `root` `Node`.
    #[must_use]
    pub fn is_root(&self) -> bool { self.id == self.parent_id }
    
    /// Gets the `id`.
    #[must_use]
    pub fn get_id(&self) -> Uuid { self.id }

    /// Gets the `branch_id`.
    #[must_use]
    pub fn get_branch_id(&self) -> Uuid { self.branch_id }
    
    /// Gets the `parent_id`.
    #[must_use]
    pub fn get_parent_id(&self) -> Uuid { self.parent_id }

    /// Checks if this `Node` has at least one child.
    #[must_use]
    pub fn has_child(&self) -> bool { self.children_ids.len() > 0 }

    /// Gets the `children_ids`.
    #[must_use]
    pub fn get_children_ids(&self) -> Vec<Uuid> { self.children_ids.clone() }
    
    /// Gets the `operation`.
    #[must_use]
    pub fn get_operation(&self) -> Operation { self.operation }

    /// Gets the cached image.
    #[must_use]
    pub fn get_image(&self) -> Option<&WorkingImage> {
        match &self.image {
            Some(image) => { Some(image) }
            None => { None }
        }
    }



    // basic editing
    /// Sets the `parent_id`.
    pub fn set_parent(&mut self, parent_id: Uuid) {
        self.parent_id = parent_id;
    }

    /// Adds a child.
    pub fn add_child(&mut self, child_id: Uuid) {
        self.children_ids.push(child_id);
    }

    /// Adds a list of children.
    pub fn add_children(&mut self, children_ids: Vec<Uuid>) {
        self.children_ids.extend(children_ids);
    }

    /// Removes a given child.
    #[must_use]
    pub fn remove_child(&mut self, child_id: Uuid) -> Schrod<()> {
        if !self.children_ids.contains(&child_id) {
            return Schrod::new_fail(&format!("{child_id} is not a child of {}!", self.id), "Node::remove_child()")
                .fail("Failed to remove child.", "Node::remove_child()")
        }
        self.children_ids.retain(|id| *id != child_id);

        Pass(())
    }

    /// Removes all children.
    pub fn remove_children(&mut self) {
        self.children_ids = Vec::new();
    }

    /// Edits the `operation`.
    pub fn edit_operation(&mut self, new_operation: Operation) {
        self.operation = new_operation;
    }

    /// Sets the image
    pub fn set_image(&mut self, new_image: Option<WorkingImage>) {
        self.image = new_image;
    }



    // drawing
    /// The size of the `Node` in the visual tree.
    const NODE_SIZE: f32 = Node::NODE_SYMBOL_SIZE + (2.0 * Node::NODE_INNER_PADDING);
    
    /// The size of the `Operation` symbol in the visual tree.
    const NODE_SYMBOL_SIZE: f32 = 16.0;
    
    /// How much space there is between the `Node`'s symbol and outer edges in the visual tree.
    const NODE_INNER_PADDING: f32 = 8.0;
    
    /// How round the `Node`'s corners are in the visual tree.
    const NODE_CORNER_RADIUS: f32 = 4.0;
    
    /// The length of the straight sections in the `Node`'s outer edges in the visual tree.
    const NODE_STRAIGH_WALL_LENGTH: f32 = Node::NODE_SIZE - (2.0 * Node::NODE_CORNER_RADIUS);
    
    /// The space between `Node`s in the visual tree.
    const NODE_SPACING: f32 = Node::NODE_SIZE * 2.0;
    
    /// Draws the given `Node` in `Map`.
    pub fn draw_in(&self, app: &App, frame: &mut Frame, position: Point, is_selected: bool) {
        // getting the node shape path
        let shape = Path::new(|pen| {
            // top left of the node
            let x = position.x - (Node::NODE_SIZE / 2.0);
            let y = position.y - (Node::NODE_SIZE / 2.0);
            // wall dimensions
            let r = Node::NODE_CORNER_RADIUS;
            let w = Node::NODE_STRAIGH_WALL_LENGTH;
            
            // drawing
            pen.move_to(Point::new(x + r, y));
            pen.line_to(Point::new(x + w - r, y));
            pen.arc_to(Point::new(x + w, y), Point::new(x + w, y + r), r);
            pen.line_to(Point::new(x + w, y + w - r));
            pen.arc_to(Point::new(x + w, y + w), Point::new(x + w - r, y + w), r);
            pen.line_to(Point::new(x + r, y + w));
            pen.arc_to(Point::new(x, y + w), Point::new(x, y + w - r), r);
            pen.line_to(Point::new(x, y + r));
            pen.arc_to(Point::new(x, y), Point::new(x + r, y), r);
            pen.close();
        });

        // coloring
        let background =
            if is_selected { MaterialColors::accent(app.material_theme()).materialized(Materials::Plastic, Depths::Flat, false, app.material_theme()) }
            else { MaterialColors::Card.materialized(Materials::Plastic, Depths::Flat, false, app.material_theme()) };
        frame.fill(&shape, Fill::from(background));
    
        // icon
        frame.fill_text(self.operation.canvas_icon(app));
    }
}