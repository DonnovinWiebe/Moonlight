use image::{ImageBuffer, Rgba};
use schrod::Schrod::{self, Pass};
use uuid::Uuid;

use crate::workspace::operation::Operation;

/// Lists the two directions that can be traversed in the `Node` `Tree`.
pub enum Direction {
    /// Moving upwards toward the `root`.
    UpStream,
    /// Moving downwards towards the tips.
    DownStream,
}



/// Holds information for an individual `Operation` or step in the edit `Tree`.
#[derive(Debug, Clone)]
pub struct Node {
    /// The unique id for this `Node` so it can be identified.
    id: Uuid,
    /// Identifies which branch this `Node` lives in.
    branch_id: Uuid,
    /// holds the parent of this `Node`.
    /// This will only be `None` for the `root` in the `Tree`.
    parent_id: Option<Uuid>,
    /// Holds the children of this `Node`.
    children_ids: Vec<Uuid>,
    /// What edit operation is being performed by this `Node`.
    operation: Operation,
    /// A chached image of the project up to this point in the `Tree`.
    image: Option<ImageBuffer<Rgba<f32>, Vec<f32>>>,
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
    pub fn new(parent_id: Option<Uuid>, branch_id: Uuid, operation: Operation) -> Node {
        Node {
            id: Uuid::now_v7(),
            branch_id,
            parent_id,
            children_ids: Vec::new(),
            operation,
            image: None,
        }
    }


    
    // basic getters
    /// Gets the `id`.
    #[must_use]
    pub fn get_id(&self) -> Uuid { self.id }

    /// Gets the `branch_id`.
    #[must_use]
    pub fn get_branch_id(&self) -> Uuid { self.branch_id }
    
    /// Gets the optional `parent_id`.
    #[must_use]
    pub fn get_parent_id(&self) -> Option<Uuid> { self.parent_id }

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
    pub fn get_image(&self) -> Option<&ImageBuffer<Rgba<f32>, Vec<f32>>> {
        match &self.image {
            Some(image) => { Some(image) }
            None => { None }
        }
    }



    // basic setters
    /// Sets the `parent_id`.
    pub fn set_parent(&mut self, parent_id: Option<Uuid>) {
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
    pub fn set_image(&mut self, new_image: Option<ImageBuffer<Rgba<f32>, Vec<f32>>>) {
        self.image = new_image;
    }
}