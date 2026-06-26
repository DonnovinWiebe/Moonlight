use image::{ImageBuffer, Rgba};
use schrod::Schrod::{self, Fail, Pass};
use uuid::Uuid;

use crate::processor::{node::Direction::{DownStream, UpStream}, operation::Operation};

/// Lists the two directions that can be traversed in the `Node` tree.
pub enum Direction {
    /// Moving upwards toward the `root`.
    UpStream,
    /// Moving downwards towards the tips.
    DownStream,
}

/// Holds the tree edit information for an image project.
pub struct Pool {
    /// The source image being edited.
    /// It is optional since the `Pool` will exist in its `Workspace` before any image is loaded.
    source_image: Option<ImageBuffer<Rgba<f32>, Vec<f32>>>,
    /// An uncategorized and unsorted mass of all 'Node's.
    /// Each node keeps track of its own heredity.
    all_nodes: Vec<Node>,
    /// The base `Node` to which all other `Node`s are attached.
    /// This cannot be edited or removed.
    root: Node,
    /// Which `Node` is being looked at.
    position: Uuid,
}
impl Pool {
    // initializing
    /// Creates a new `Pool`.
    #[must_use]
    pub fn new() -> Pool {
        let root = Node::new(None, Uuid::now_v7(), Operation::Root);
        let root_id = root.get_id();
        
        Pool {
            source_image: None,
            all_nodes: Vec::new(),
            root,
            position: root_id,
        }
    }

    /// Sets the image.
    pub fn set_image(&mut self, image: ImageBuffer<Rgba<f32>, Vec<f32>>) {
        self.source_image = Some(image);
    }


    
    // basic getters
    /// Gets the source image of the `Pool`.
    #[must_use]
    pub fn get_image(&self) -> Option<&ImageBuffer<Rgba<f32>, Vec<f32>>> {
        match &self.source_image {
            Some(image) => { Some(image) }
            None => { None }
        }
    }

    /// Gets an immutable reference to a `Node` in the `Pool`.
    #[must_use]
    pub fn get(&self, id: Uuid) -> Schrod<&Node> {
        for node in &self.all_nodes {
            if node.get_id() == id { return Pass(node) }
        }

        Schrod::new_fail(&format!("Id {id} does not exist!"), "Pool::get()").fail(&format!("Failed to get node for id {id}."), "Pool::get()")
    }

    /// Gets a mutable reference to a `Node` in the `Pool`.
    #[must_use]
    pub fn get_mut(&mut self, id: Uuid) -> Schrod<&mut Node> {
        for node in &mut self.all_nodes {
            if node.get_id() == id { return Pass(node) }
        }

        Schrod::new_fail(&format!("Id {id} does not exist!"), "Pool::get_mut()").fail(&format!("Failed to get node for id {id}."), "Pool::get_mut()")
    }

    /// Gets all the upstream `Node`s from the given `Node`.
    /// This does not include the given `Node`.
    #[must_use]
    fn get_all_upstream_nodes(&self, node_id: Uuid) -> Schrod<Vec<Uuid>> {
        // the lists in use
        let mut current_node_id = node_id;
        let mut all_nodes: Vec<Uuid> = Vec::new();

        // loops until every node upstream has been expolored
        loop {
            // gets the current node
            let node_result = self.get(current_node_id);
            if node_result.is_fail() {
                return node_result
                    .convert("Pool::get_all_upstream_nodes()")
                    .fail("Failed to get upstream end nodes.", "Pool::get_all_upstream_nodes()")
            }
            let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::get_all_upstream_nodes()");
            

            // adds the node to the list of all nodes
            if node.get_id() != node_id { all_nodes.push(node.get_id()) }
            // sets the node's parent as the curren node or ends the loop if the top has been found
            match node.get_parent_id() {
                Some(parent_id) => { current_node_id = parent_id }
                None => { break }
            }
        }
        
        // returns the verified end points
        Pass(all_nodes)
    }

    /// Gets all the downstream `Node`s from the given `Node`.
    /// This does not include the given `Node`.
    #[must_use]
    fn get_all_downstream_nodes(&self, node_id: Uuid) -> Schrod<Vec<Uuid>> {
        // the lists in use
        let mut current_nodes: Vec<Uuid> = vec![node_id];
        let mut new_nodes: Vec<Uuid> = Vec::new();
        let mut all_nodes: Vec<Uuid> = Vec::new();

        // continues until all downstream nodes have been explored
        loop {
            // gets the current node
            for current_node in &current_nodes {
                let node_result = self.get(*current_node);
                if node_result.is_fail() {
                    return node_result
                        .convert("Pool::get_downstream_end_nodes()")
                        .fail("Failed to get downstream end nodes.", "Pool::get_downstream_end_nodes()")
                }
                let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::get_downstream_end_nodes()");

                // adds the node to the list of all nodes
                if node.get_id() != node_id { all_nodes.push(node.get_id()) }
                // gets the nodes children if it has any
                if node.has_child() { new_nodes.extend(node.get_children_ids()); }
            }

            // breaks out of the loop if no new nodes were found to explore
            if new_nodes.is_empty() { break }
            // moves the new nodes to the current list being explored
            else {
                current_nodes = new_nodes;
                new_nodes = Vec::new();
            }
        }

        // returns the verified end points
        Pass(all_nodes)
    }

    /// Gets all the farthest downstream `Node`s from the given `Node`.
    /// This will include the given `Node` if it is the downstream end point.
    #[must_use]
    fn get_downstream_end_nodes(&self, node_id: Uuid) -> Schrod<Vec<Uuid>> {
        // the lists in use
        let mut current_end_points: Vec<Uuid> = vec![node_id];
        let mut new_end_points: Vec<Uuid> = Vec::new();
        let mut verified_end_points: Vec<Uuid> = Vec::new();

        // continues until all downstream nodes have been explored
        loop {
            // gets the current node
            for current_end_point in &current_end_points {
                let node_result = self.get(*current_end_point);
                if node_result.is_fail() {
                    return node_result
                        .convert("Pool::get_downstream_end_nodes()")
                        .fail("Failed to get downstream end nodes.", "Pool::get_downstream_end_nodes()")
                }
                let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::get_downstream_end_nodes()");

                // gets the nodes children if it has any
                if node.has_child() { new_end_points.extend(node.get_children_ids()); }
                // adds the node to the verified end points if it has no children
                else { verified_end_points.push(node.get_id()) }
            }

            // breaks out of the loop if no new nodes were found to explore
            if new_end_points.is_empty() { break }
            // moves the new nodes to the current list being explored
            else {
                current_end_points = new_end_points;
                new_end_points = Vec::new();
            }
        }

        // returns the verified end points
        Pass(verified_end_points)
    }



    // tree management
    /// Adds a new `Node` after the current `position` in the `Pool`.
    /// If there are `Node`s after the current `position`, the new `Node` is inserted.
    #[must_use]
    pub fn add_node(&mut self, operation: Operation) -> Schrod<()> {
        // immediately fails if there is no source image
        if self.source_image.is_none() {
            return Schrod::new_fail("No source image found!", "Pool::add_node()")
                .fail("Failed to add node.", "Pool::add_node()")
        }

        // gets the current position for repeated use
        let position = self.position;
        
        // gets the branch id from the current node and takes its children ids out to be moved to the new node
        let (branch_id, children_ids) = {
            let current_node_result = self.get_mut(position);
            if current_node_result.is_fail() {
                return current_node_result
                    .convert("Pool::add_node()")
                    .fail("Failed to add node.", "Pool::add_node()")
            }
            let current_node = current_node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::add_node()");
            let children_ids = current_node.get_children_ids();
            current_node.remove_children();
            (current_node.get_branch_id(), children_ids)
        };

        // creates a new node and gives it all the children
        let mut new_node = Node::new(Some(position), branch_id, operation);
        new_node.add_children(children_ids.clone());

        // gives all the children the new node for their parent
        for child_id in children_ids {
            let child_result = self.get_mut(child_id);
            if child_result.is_fail() {
                return child_result
                    .convert("Pool::add_node()")
                    .fail("Failed to add node.", "Pool::add_node()")
            }
            let child = child_result.wont_fail("This is past an is_fail() guard clause.", "Pool::add_node()");
            child.set_parent(Some(new_node.get_id()));
        }

        // updates the current position and adds the new node to the pool
        self.position = new_node.get_id();
        self.all_nodes.push(new_node);
        
        // updates the image of the node
        let update_result = self.update_image_for(self.position);
        if update_result.is_fail() {
            return update_result
                .convert("Pool::add_node()")
        }

        // prunes to save resources and prevent erroneous image edit states downstream
        let prune_result = self.prune(self.position, Direction::UpStream);
        if prune_result.is_fail() { return prune_result.convert("Pool::add_node()") }
        let prune_result = self.prune(self.position, Direction::DownStream);
        if prune_result.is_fail() { return prune_result.convert("Pool::add_node()") }

        // returns a success
        Pass(())
    }
    
    /// Adds a new `Node` after the current `position` in the `Pool`.
    /// If there are `Node`s after the current `position`, the new `Node` is added to a new branch.
    #[must_use]
    pub fn add_branch(&mut self, operation: Operation) -> Schrod<()> {
        // immediately fails if there is no source image
        if self.source_image.is_none() {
            return Schrod::new_fail("No source image found!", "Pool::add_branch()")
                .fail("Failed to add branch.", "Pool::add_branch()")
        }

        // gets the current position for repeated use
        let position = self.position;

        // creates a new node with a new branch id
        let new_node = Node::new(Some(position), Uuid::now_v7(), operation);

        // updates the current position and adds the new node to the pool
        self.position = new_node.get_id();
        self.all_nodes.push(new_node);

        // updates the image of the node
        let update_result = self.update_image_for(self.position);
        if update_result.is_fail() {
            return update_result
                .convert("Pool::add_branch()")
        }
        
        // prunes to save resources
        let prune_result = self.prune(self.position, Direction::UpStream);
        if prune_result.is_fail() { return prune_result.convert("Pool::add_branch()") }

        // returns a success
        Pass(())
    }

    /// Edits the `Operation` of the given `Node`.
    #[must_use]
    pub fn edit_node(&mut self, node_id: Uuid, new_operation: Operation) -> Schrod<()> {
        // cannot edit the root node
        if node_id == self.root.get_id() {
            return Schrod::new_fail("Cannot edit root node!", "Pool::edit_node()")
                .fail("Failed to edit node.", "Pool::edit_node()")
        }
        
        // prunes to prevent erroneous image edit states downstream
        let prune_result = self.prune(self.position, Direction::DownStream);
        if prune_result.is_fail() { return prune_result.convert("Pool::edit_node()") }

        // edits the node
        let node_result = self.get_mut(node_id);
        if node_result.is_fail() {
            return node_result
                .convert("Pool::edit_node()")
                .fail("Failed to edit node.", "Pool::edit_node()")
        }
        let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::edit_node()");
        node.edit_operation(new_operation);

        // updates the image of the node
        let update_result = self.update_image_for(node_id);
        if update_result.is_fail() {
            return update_result
                .convert("Pool::edit_node()")
        }

        // returns a success
        Pass(())
    }

    /// Removes the given `Node` while preserving hereditiy.
    #[must_use]
    pub fn remove_node(&mut self, node_id: Uuid) -> Schrod<()> {
        // cannot edit the root node
        if node_id == self.root.get_id() {
            return Schrod::new_fail("Cannot edit root node!", "Pool::remove_node()")
                .fail("Failed to edit node.", "Pool::remove_node()")
        }
        
        // prunes to prevent erroneous image edit states downstream
        let prune_result = self.prune(self.position, Direction::DownStream);
        if prune_result.is_fail() { return prune_result.convert("Pool::remove_node()") }
        
        // gets the parent id and children ids for the node
        let (parent_id, children_ids) = {
            let node_result = self.get(node_id);
            if node_result.is_fail() {
                return node_result
                    .convert("Pool::remove_node()")
                    .fail("Failed to remove node.", "Pool::remove_node()")
            }
            let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::remove_node()");
            (node.get_parent_id(), node.get_children_ids())
        };

        // gives the parent the children ids and removes the given node as a child
        if let Some(parent_id) = parent_id {
            let parent_result = self.get_mut(parent_id);
            if parent_result.is_fail() {
                return parent_result
                    .convert("Pool::remove_node()")
                    .fail("Failed to remove node.", "Pool::remove_node()")
            }
            let parent = parent_result.wont_fail("This is past an is_fail() guard clause.", "Pool::remove_node()");
            parent.add_children(children_ids.clone());
            let remove_result = parent.remove_child(node_id);
            if remove_result.is_fail() {
                return remove_result
                    .convert("Pool::remove_node()")
                    .fail("Failed to remove node.", "Pool::remove_node()")
            }
        }

        // every node will have a parent except the root will have a parent
        else {
            return Schrod::new_fail("Tried to remove a node that has no parent!", "Pool::remove_node()")
                .fail("Failed to remove node.", "Pool::remove_node()")
        }

        // gives the parent id to the children
        for child_id in children_ids {
            let child_result = self.get_mut(child_id);
            if child_result.is_fail() {
                return child_result
                    .convert("Pool::remove_node()")
                    .fail("Failed to remove node.", "Pool::remove_node()")
            }
            let child = child_result.wont_fail("This is past an is_fail() guard clause.", "Pool::remove_node()");
            child.set_parent(parent_id);
        }

        // moves the pool's position if it is set to the node being removed
        if self.position == node_id {
            if let Some(parent_id) = parent_id { self.position = parent_id; }
            
            // every node will have a parent except the root will have a parent
            else {
                return Schrod::new_fail("Tried to remove a node that has no parent!", "Pool::remove_node()")
                    .fail("Failed to remove node.", "Pool::remove_node()")
            }
        }

        // removing the node
        self.all_nodes.retain(|existing_node| existing_node.get_id() !=  node_id);
        
        // updates the image for the current position
        let update_result = self.update_image_for(self.position);
        if update_result.is_fail() {
            return update_result
                .convert("Pool::remove_node()")
        }
        
        // returns a success
        Pass(())
    }



    // resource management
    /// Gets the cached image for the given `Node`.
    /// If the `Node` has no cached image, a new one is generated.
    #[must_use]
    pub fn get_image_for(&mut self, node_id: Uuid) -> Schrod<&ImageBuffer<Rgba<f32>, Vec<f32>>> {
        // updates the image in the given node
        let update_result = self.update_image_for(node_id);
        match update_result {
            Pass(()) => {
                // gets the current node
                let node_result = self.get(node_id);
                if node_result.is_fail() {
                    return node_result
                        .convert("Pool::get_image_for()")
                        .fail(&format!("Failed to get the image for {node_id}."), "Pool::get_image_for()")
                }
                let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::get_image_for()");

                // tried to get the cached image
                if let Some(image) = node.get_image() { return Pass(image) }
                return Schrod::new_fail("Failed to get image after successful update!", "Pool::get_image_for()")
                    .fail(&format!("Failed to get the image for {node_id}."), "Pool::get_image_for()")
            }
            Fail(_) => {
                return update_result
                    .convert("Pool::get_image_for()")
                    .fail(&format!("Failed to get the image for {node_id}."), "Pool::get_image_for()")
            }
        }
    }
    
    /// Updates the cached image for the given `Node`.
    #[must_use]
    fn update_image_for(&mut self, node_id: Uuid) -> Schrod<()> {
        // gets the updated the image
        let new_image_result = self.generate_image_for(node_id);
        if new_image_result.is_fail() {
            return new_image_result
                .convert("Pool::update_image_for()")
                .fail("Failed to generate image.", "Pool::update_image_for()")
        }
        let new_image = new_image_result.wont_fail("This is past an is_fail() guard clause.", "Pool::update_image_for()");
        
        // gets the node
        let node_result = self.get_mut(node_id);
        if node_result.is_fail() {
            return node_result
                .convert("Pool::update_image_for()")
                .fail("Failed to generate image.", "Pool::update_image_for()")
        }
        let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::update_image_for()");

        // updates the image
        node.image = Some(new_image);
        Pass(())
    }

    /// Generates an image for the given `Node`
    #[must_use]
    fn generate_image_for(&self, node_id: Uuid) -> Schrod<ImageBuffer<Rgba<f32>, Vec<f32>>> {
        // gets the node
        let node_result = self.get(node_id);
        if node_result.is_fail() {
            return node_result
                .convert("Pool::generate_image_for()")
                .fail("Failed to generate image.", "Pool::generate_image_for()")
        }
        let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::generate_image_for()");

        // this would cut back on operation time, but may cause some nodes not to update
        /*
        // returns the node's own image if it has already been created
        if let Some(image) = node.get_image() { return Pass(image.clone()) } // expensive clone?
        */

        // checks if this node has a parent
        match node.get_parent_id() {
            // generates a new image based on this node's parent
            Some(parent_id) => {
                let previous_image_result =  self.generate_image_for(parent_id);
                if previous_image_result.is_fail() { return previous_image_result.fail("Failed to generate image.", "Pool::generate_image_for()") }
                let mut previous_image = previous_image_result.wont_fail("This is past an is_fail() guard clause.", "Pool::generate_image_for()");
    
                node.get_operation().apply_to(&mut previous_image);
                let new_image = previous_image;
                return Pass(new_image)
            }
            
            // generates a new image based on the source image of the pool
            None => {
                if let Some(source_image) = self.get_image() {
                    let mut new_image = source_image.clone();
                    node.get_operation().apply_to(&mut new_image);
                    return Pass(new_image)
                }
                else {
                    return Schrod::new_fail("No source image found!", "Pool::generate_image_for()")
                        .fail("Failed to generate image.", "Pool::generate_image_for()")
                }
            }
        }
    }
    
    /// Clears all cached images in all the `Node`s `UpStream` or `DownStream` of the `starting_node_id`.
    /// This does not affect the given `Node`.
    #[must_use]
    pub fn prune(&mut self, starting_node_id: Uuid, direction: Direction) -> Schrod<()> {
        // gets the nodes to snip
        let node_ids_to_snip_result = match direction {
            UpStream => self.get_all_upstream_nodes(starting_node_id),
            DownStream => self.get_all_downstream_nodes(starting_node_id),
        };
        if node_ids_to_snip_result.is_fail() {
            return node_ids_to_snip_result
                .convert("Pool::prune()")
                .fail("Failed to prune.", "Pool::prune()")
        }
        let node_ids_to_snip = node_ids_to_snip_result.wont_fail("This is past an is_fail() guard clause.", "Pool::prune()");

        // snips each node
        for node_id in &node_ids_to_snip {
            let node_result = self.get_mut(*node_id);
            if node_result.is_fail() {
                return node_result
                    .convert("Pool::prune()")
                    .fail("Failed to prune.", "Pool::prune()")
            }
            let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::prune()");
            node.snip_image();
        }

        // returns a success
        Pass(())
    }
}



/// Holds information for an individual `Operation` or step in the edit tree.
#[derive(Debug, Clone)]
pub struct Node {
    /// The unique id for this `Node` so it can be identified.
    id: Uuid,
    /// Identifies which branch this `Node` lives in.
    branch_id: Uuid,
    /// holds the parent of this `Node`.
    /// This will only be `None` for the `root` in the `Pool`.
    parent_id: Option<Uuid>,
    /// Holds the children of this `Node`.
    children_ids: Vec<Uuid>,
    /// What edit operation is being performed by this `Node`.
    operation: Operation,
    /// A chached image of the project up to this point in the tree.
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



    // basic editing
    /// Sets the `parent_id`.
    fn set_parent(&mut self, parent_id: Option<Uuid>) {
        self.parent_id = parent_id;
    }

    /// Adds a child.
    fn add_child(&mut self, child_id: Uuid) {
        self.children_ids.push(child_id);
    }

    /// Adds a list of children.
    fn add_children(&mut self, children_ids: Vec<Uuid>) {
        self.children_ids.extend(children_ids);
    }

    /// Removes a given child.
    #[must_use]
    fn remove_child(&mut self, child_id: Uuid) -> Schrod<()> {
        if !self.children_ids.contains(&child_id) {
            return Schrod::new_fail(&format!("{child_id} is not a child of {}!", self.id), "Node::remove_child()")
                .fail("Failed to remove child.", "Node::remove_child()")
        }
        self.children_ids.retain(|id| *id != child_id);

        Pass(())
    }

    /// Removes all children.
    fn remove_children(&mut self) {
        self.children_ids = Vec::new();
    }

    /// Edits the `operation`.
    fn edit_operation(&mut self, new_operation: Operation) {
        self.operation = new_operation;
    }

    /// Sets the image
    fn set_image(&mut self, new_image: ImageBuffer<Rgba<f32>, Vec<f32>>) {
        self.image = Some(new_image);
    }

    /// Clears the cached image.
    fn snip_image(&mut self) {
        self.image = None;
    }
}