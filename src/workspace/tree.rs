use std::path::PathBuf;

use iced::widget::image::Handle;
use schrod::Schrod::{self, Fail, Pass};
use uuid::Uuid;

use crate::workspace::{node::{Buffer, Direction::{self, DownStream, UpStream}, Node, WorkingImage}, operation::Operation};

/// Holds all the project data.
pub struct Tree {
    /// The source path of the image being edited.
    /// It is optional since the `Tree` will exist before any image is loaded.
    source_path: Option<PathBuf>,
    /// The source image being edited.
    /// It is optional since the `Tree` will exist before any image is loaded.
    source_image: Option<Buffer>,
    /// An uncategorized and unsorted mass of all 'Node's.
    /// Each node keeps track of its own heredity.
    all_nodes: Vec<Node>,
    /// The base `Node` to which all other `Node`s are attached in the `Node` `Tree`.
    /// This cannot be edited or removed.
    root: Node,
    /// Which `Node` is being looked at.
    position: Uuid,
}
impl Tree {
    // initializing
    /// Creates a new `Tree`.
    #[must_use]
    pub fn new() -> Tree {
        let root = Node::new(None, Uuid::now_v7(), Operation::Root);
        let root_id = root.get_id();
        
        Tree {
            source_path: None,
            source_image: None,
            all_nodes: Vec::new(),
            root,
            position: root_id,
        }
    }

    /// Sets the image path.
    #[must_use]
    pub fn set_source_path(&mut self, path: PathBuf) -> Schrod<()> {
        let load_result = Schrod::from_result(image::open(path.clone()), &format!("Failed to open image at {:.?}", &path), "Tree::set_source_path()");
        if load_result.is_fail() {
            return load_result
                .convert("Tree::set_source_path()")
                .fail(&format!("Failed to open image at {:.?}", &path), "Tree::set_source_path()")
        }
        let rgba_image = load_result.wont_fail("This is past an is_fail() guard clause.", "Tree::set_source_path()").into_rgba32f();
        self.source_path = Some(path);
        self.source_image = Some(rgba_image);

        Pass(())
    }


    
    // basic getters
    /// Gets a `Handle` that `Iced` can display.
    #[must_use]
    pub fn get_current_handle(&self) -> Schrod<Handle> {
        let node_result = self.get(self.position);
        if node_result.is_fail() {
            return node_result
                .convert("Tree::get_handle_for()")
                .fail("Failed to get handle for node.", "Tree::get_handle_for()")
        }
        let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::get_handle_for()");

        match node.get_image() {
            Some(image) => {
                let srgb_image = image.get_srgb();
                let bytes: Vec<u8> = srgb_image.as_raw().iter().map(|&v| (v * 255.0).clamp(0.0, 255.0) as u8).collect();
                Pass(Handle::from_rgba(srgb_image.width(), srgb_image.height(), bytes))
            }
            
            None => {
                Schrod::new_fail("No image generated!", "Tree::get_handle_for()")
                    .fail("Failed to get handle for node.", "Tree::get_handle_for()")
            }
        }
    }
        
    /// Gets the source image of the `Tree`.
    #[must_use]
    fn get_image(&self) -> Option<&Buffer> {
        match &self.source_image {
            Some(image) => { Some(image) }
            None => { None }
        }
    }

    /// Gets an immutable reference to a `Node` in the `Tree`.
    #[must_use]
    pub fn get(&self, id: Uuid) -> Schrod<&Node> {
        for node in &self.all_nodes {
            if node.get_id() == id { return Pass(node) }
        }

        Schrod::new_fail(&format!("Id {id} does not exist!"), "Tree::get()")
            .fail(&format!("Failed to get node for id {id}."), "Tree::get()")
    }

    /// Gets a mutable reference to a `Node` in the `Tree`.
    #[must_use]
    pub fn get_mut(&mut self, id: Uuid) -> Schrod<&mut Node> {
        for node in &mut self.all_nodes {
            if node.get_id() == id { return Pass(node) }
        }

        Schrod::new_fail(&format!("Id {id} does not exist!"), "Tree::get_mut()")
            .fail(&format!("Failed to get node for id {id}."), "Tree::get_mut()")
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
                    .convert("Tree::get_all_upstream_nodes()")
                    .fail("Failed to get upstream end nodes.", "Tree::get_all_upstream_nodes()")
            }
            let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::get_all_upstream_nodes()");
            

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
                        .convert("Tree::get_downstream_end_nodes()")
                        .fail("Failed to get downstream end nodes.", "Tree::get_downstream_end_nodes()")
                }
                let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::get_downstream_end_nodes()");

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
                        .convert("Tree::get_downstream_end_nodes()")
                        .fail("Failed to get downstream end nodes.", "Tree::get_downstream_end_nodes()")
                }
                let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::get_downstream_end_nodes()");

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

    /// Gets every branch id.
    #[must_use]
    pub fn get_branch_ids(&self) -> Vec<Uuid> {
        let mut branches = Vec::new();
        for node in &self.all_nodes {
            if !branches.contains(&node.get_branch_id()) { branches.push(node.get_branch_id()) }
        }
        branches
    }



    // tree management
    /// Adds a new `Node` after the current `position` in the `Tree`.
    /// If there are `Node`s after the current `position`, the new `Node` is inserted.
    #[must_use]
    pub fn add_node(&mut self, operation: Operation) -> Schrod<()> {
        // immediately fails if there is no source image
        if self.source_image.is_none() {
            return Schrod::new_fail("No source image found!", "Tree::add_node()")
                .fail("Failed to add node.", "Tree::add_node()")
        }

        // gets the current position for repeated use
        let position = self.position;
        
        // gets the branch id from the current node and takes its children ids out to be moved to the new node
        let (branch_id, children_ids) = {
            let current_node_result = self.get_mut(position);
            if current_node_result.is_fail() {
                return current_node_result
                    .convert("Tree::add_node()")
                    .fail("Failed to add node.", "Tree::add_node()")
            }
            let current_node = current_node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::add_node()");
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
                    .convert("Tree::add_node()")
                    .fail("Failed to add node.", "Tree::add_node()")
            }
            let child = child_result.wont_fail("This is past an is_fail() guard clause.", "Tree::add_node()");
            child.set_parent(Some(new_node.get_id()));
        }

        // updates the current position and adds the new node to the tree
        self.position = new_node.get_id();
        self.all_nodes.push(new_node);
        
        // updates the image of the node
        let update_result = self.update_image_for(self.position);
        if update_result.is_fail() {
            return update_result
                .convert("Tree::add_node()")
        }

        // prunes to save resources and prevent erroneous image edit states downstream
        let prune_result = self.prune(self.position, Direction::UpStream);
        if prune_result.is_fail() { return prune_result.convert("Tree::add_node()") }
        let prune_result = self.prune(self.position, Direction::DownStream);
        if prune_result.is_fail() { return prune_result.convert("Tree::add_node()") }

        // returns a success
        Pass(())
    }
    
    /// Adds a new `Node` after the current `position` in the `Tree`.
    /// If there are `Node`s after the current `position`, the new `Node` is added to a new branch.
    #[must_use]
    pub fn add_branch(&mut self, operation: Operation) -> Schrod<()> {
        // immediately fails if there is no source image
        if self.source_image.is_none() {
            return Schrod::new_fail("No source image found!", "Tree::add_branch()")
                .fail("Failed to add branch.", "Tree::add_branch()")
        }

        // gets the current position for repeated use
        let position = self.position;

        // creates a new node with a new branch id
        let new_node = Node::new(Some(position), Uuid::now_v7(), operation);

        // updates the current position and adds the new node to the tree
        self.position = new_node.get_id();
        self.all_nodes.push(new_node);

        // updates the image of the node
        let update_result = self.update_image_for(self.position);
        if update_result.is_fail() {
            return update_result
                .convert("Tree::add_branch()")
        }
        
        // prunes to save resources
        let prune_result = self.prune(self.position, Direction::UpStream);
        if prune_result.is_fail() { return prune_result.convert("Tree::add_branch()") }

        // returns a success
        Pass(())
    }

    /// Edits the `Operation` of the given `Node`.
    #[must_use]
    pub fn edit_node(&mut self, node_id: Uuid, new_operation: Operation) -> Schrod<()> {
        // cannot edit the root node
        if node_id == self.root.get_id() {
            return Schrod::new_fail("Cannot edit root node!", "Tree::edit_node()")
                .fail("Failed to edit node.", "Tree::edit_node()")
        }
        
        // prunes to prevent erroneous image edit states downstream
        let prune_result = self.prune(self.position, Direction::DownStream);
        if prune_result.is_fail() { return prune_result.convert("Tree::edit_node()") }

        // edits the node
        let node_result = self.get_mut(node_id);
        if node_result.is_fail() {
            return node_result
                .convert("Tree::edit_node()")
                .fail("Failed to edit node.", "Tree::edit_node()")
        }
        let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::edit_node()");
        node.edit_operation(new_operation);

        // updates the image of the node
        let update_result = self.update_image_for(node_id);
        if update_result.is_fail() {
            return update_result
                .convert("Tree::edit_node()")
        }

        // returns a success
        Pass(())
    }

    /// Removes the given `Node` while preserving hereditiy.
    #[must_use]
    pub fn remove_node(&mut self, node_id: Uuid) -> Schrod<()> {
        // cannot edit the root node
        if node_id == self.root.get_id() {
            return Schrod::new_fail("Cannot edit root node!", "Tree::remove_node()")
                .fail("Failed to edit node.", "Tree::remove_node()")
        }
        
        // prunes to prevent erroneous image edit states downstream
        let prune_result = self.prune(self.position, Direction::DownStream);
        if prune_result.is_fail() { return prune_result.convert("Tree::remove_node()") }
        
        // gets the parent id and children ids for the node
        let (parent_id, children_ids) = {
            let node_result = self.get(node_id);
            if node_result.is_fail() {
                return node_result
                    .convert("Tree::remove_node()")
                    .fail("Failed to remove node.", "Tree::remove_node()")
            }
            let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::remove_node()");
            (node.get_parent_id(), node.get_children_ids())
        };

        // gives the parent the children ids and removes the given node as a child
        if let Some(parent_id) = parent_id {
            let parent_result = self.get_mut(parent_id);
            if parent_result.is_fail() {
                return parent_result
                    .convert("Tree::remove_node()")
                    .fail("Failed to remove node.", "Tree::remove_node()")
            }
            let parent = parent_result.wont_fail("This is past an is_fail() guard clause.", "Tree::remove_node()");
            parent.add_children(children_ids.clone());
            let remove_result = parent.remove_child(node_id);
            if remove_result.is_fail() {
                return remove_result
                    .convert("Tree::remove_node()")
                    .fail("Failed to remove node.", "Tree::remove_node()")
            }
        }

        // every node will have a parent except the root will have a parent
        else {
            return Schrod::new_fail("Tried to remove a node that has no parent!", "Tree::remove_node()")
                .fail("Failed to remove node.", "Tree::remove_node()")
        }

        // gives the parent id to the children
        for child_id in children_ids {
            let child_result = self.get_mut(child_id);
            if child_result.is_fail() {
                return child_result
                    .convert("Tree::remove_node()")
                    .fail("Failed to remove node.", "Tree::remove_node()")
            }
            let child = child_result.wont_fail("This is past an is_fail() guard clause.", "Tree::remove_node()");
            child.set_parent(parent_id);
        }

        // moves the tree's position if it is set to the node being removed
        if self.position == node_id {
            if let Some(parent_id) = parent_id { self.position = parent_id; }
            
            // every node will have a parent except the root will have a parent
            else {
                return Schrod::new_fail("Tried to remove a node that has no parent!", "Tree::remove_node()")
                    .fail("Failed to remove node.", "Tree::remove_node()")
            }
        }

        // removing the node
        self.all_nodes.retain(|existing_node| existing_node.get_id() !=  node_id);
        
        // updates the image for the current position
        let update_result = self.update_image_for(self.position);
        if update_result.is_fail() {
            return update_result
                .convert("Tree::remove_node()")
        }
        
        // returns a success
        Pass(())
    }



    // resource management
    /// Gets the cached image for the given `Node`.
    /// If the `Node` has no cached image, a new one is generated.
    #[must_use]
    pub fn get_image_for(&mut self, node_id: Uuid) -> Schrod<&WorkingImage> {
        // updates the image in the given node
        let update_result = self.update_image_for(node_id);
        match update_result {
            Pass(()) => {
                // gets the current node
                let node_result = self.get(node_id);
                if node_result.is_fail() {
                    return node_result
                        .convert("Tree::get_image_for()")
                        .fail(&format!("Failed to get the image for {node_id}."), "Tree::get_image_for()")
                }
                let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::get_image_for()");

                // tried to get the cached image
                if let Some(image) = node.get_image() { return Pass(image) }
                return Schrod::new_fail("Failed to get image after successful update!", "Tree::get_image_for()")
                    .fail(&format!("Failed to get the image for {node_id}."), "Tree::get_image_for()")
            }
            Fail(_) => {
                return update_result
                    .convert("Tree::get_image_for()")
                    .fail(&format!("Failed to get the image for {node_id}."), "Tree::get_image_for()")
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
                .convert("Tree::update_image_for()")
                .fail("Failed to generate image.", "Tree::update_image_for()")
        }
        let new_image = new_image_result.wont_fail("This is past an is_fail() guard clause.", "Tree::update_image_for()");
        
        // gets the node
        let node_result = self.get_mut(node_id);
        if node_result.is_fail() {
            return node_result
                .convert("Tree::update_image_for()")
                .fail("Failed to generate image.", "Tree::update_image_for()")
        }
        let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::update_image_for()");

        // updates the image
        node.set_image(Some(new_image));
        Pass(())
    }

    /// Generates a new image for the given `Node`
    #[must_use]
    fn generate_image_for(&self, node_id: Uuid) -> Schrod<WorkingImage> {
        // gets the node
        let node_result = self.get(node_id);
        if node_result.is_fail() {
            return node_result
                .convert("Tree::generate_image_for()")
                .fail("Failed to generate image.", "Tree::generate_image_for()")
        }
        let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::generate_image_for()");

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
                if previous_image_result.is_fail() {
                    return previous_image_result
                        .convert("Tree::generate_image_for()")
                        .fail("Failed to generate image.", "Tree::generate_image_for()")
                }
                let previous_image = previous_image_result.wont_fail("This is past an is_fail() guard clause.", "Tree::generate_image_for()");
    
                let new_image = node.get_operation().applied_to(&previous_image);
                return Pass(new_image)
            }
            
            // generates a new image based on the source image of the tree
            None => {
                if let Some(source_image) = self.get_image() {
                    let new_image = node.get_operation().applied_to(&WorkingImage::from_srgb(source_image.clone()));
                    return Pass(new_image)
                }
                else {
                    return Schrod::new_fail("No source image found!", "Tree::generate_image_for()")
                        .fail("Failed to generate image.", "Tree::generate_image_for()")
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
                .convert("Tree::prune()")
                .fail("Failed to prune.", "Tree::prune()")
        }
        let node_ids_to_snip = node_ids_to_snip_result.wont_fail("This is past an is_fail() guard clause.", "Tree::prune()");

        // snips each node
        for node_id in &node_ids_to_snip {
            let node_result = self.get_mut(*node_id);
            if node_result.is_fail() {
                return node_result
                    .convert("Tree::prune()")
                    .fail("Failed to prune.", "Tree::prune()")
            }
            let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Tree::prune()");
            node.set_image(None);
        }

        // returns a success
        Pass(())
    }
}