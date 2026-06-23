use image::{ImageBuffer, Rgba};
use schrod::Schrod::{self, Fail, Pass};
use uuid::Uuid;

use crate::processor::operation::Operation;

pub struct Pool {
    source_image: Option<ImageBuffer<Rgba<f32>, Vec<f32>>>,
    nodes: Vec<Node>,
    position: Option<Uuid>,
}
impl Pool {
    // initializing
    pub fn new() -> Pool {
        Pool { source_image: None, nodes: Vec::new(), position: None }
    }

    pub fn set_image(&mut self, image: ImageBuffer<Rgba<f32>, Vec<f32>>) {
        self.source_image = Some(image);
    }


    
    // basic getters
    pub fn get_image(&self) -> Option<&ImageBuffer<Rgba<f32>, Vec<f32>>> {
        match &self.source_image {
            Some(image) => Some(&image),
            None => None
        }
    }

    pub fn get(&self, id: Uuid) -> Schrod<&Node> {
        for node in &self.nodes {
            if node.get_id() == id { return Pass(node) }
        }

        Schrod::new_fail(&format!("Id {id} does not exist!"), "Pool::get()").fail(&format!("Failed to get node for id {id}."), "Pool::get()")
    }

    pub fn get_mut(&mut self, id: Uuid) -> Schrod<&mut Node> {
        for node in &mut self.nodes {
            if node.get_id() == id { return Pass(node) }
        }

        Schrod::new_fail(&format!("Id {id} does not exist!"), "Pool::get_mut()").fail(&format!("Failed to get node for id {id}."), "Pool::get_mut()")
    }



    // resource management
    pub fn prune(&mut self, last_node_id: Uuid) -> Schrod<()> {
        let mut current_node_id = last_node_id;
        loop {
            let node_result = self.get_mut(current_node_id);
            if node_result.is_fail() {
                return node_result
                    .convert("Pool::prune()")
                    .fail("Failed to prune.", "Pool::prune()")
            }
            let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::prune()");
            
            node.snip();
            if let Some(parent_id) = node.get_parent_id() { current_node_id = parent_id; }
            else { break; }
        }
        
        Pass(())
    }



    // tree management
    pub fn add_node(&mut self, operation: Operation) -> Schrod<()> {
        // immediately fails if there is no source image
        if self.source_image.is_none() {
            return Schrod::new_fail("No source image found!", "Pool::add_node()")
                .fail("Failed to add node.", "Pool::add_node()")
        }

        // checks if the pool has a position set
        match self.position {
            // creates a node downstream of the current location
            Some(position) => {
                // gets the node at the position
                let current_node_result = self.get(position);
                if current_node_result.is_fail() {
                    return current_node_result
                        .convert("Pool::add_node()")
                        .fail("Failed to add node.", "Pool::add_node()")
                }
                let current_node = current_node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::add_node()");

                // if the current node already has children, a new branch is created
                let branch_id = if current_node.has_child() { Uuid::now_v7() } else { current_node.get_branch_id() };

                // creates a new node and sets the current position to the new node
                let new_node = Node::new(Some(current_node.get_id()), branch_id, operation);
                self.position = Some(new_node.get_id());
                self.nodes.push(new_node);
            }

            // creates a node downstream of the base image
            None => {
                // creates a new node and sets the current position to the new node
                let new_node = Node::new(None, Uuid::now_v7(), operation);
                self.position = Some(new_node.get_id());
                self.nodes.push(new_node);
            }
        }

        // returns a success
        Pass(())
    }

    pub fn edit_node(&mut self, node_id: Uuid, new_operation: Operation) -> Schrod<()> {
        let node_result = self.get_mut(node_id);
        if node_result.is_fail() {
            return node_result
                .convert("Pool::edit_node()")
                .fail("Failed to edit node.", "Pool::edit_node()")
        }
        let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::edit_node()");
        node.edit_operation(new_operation);

        Pass(())
    }

    pub fn remove_node(&mut self, node_id: Uuid) -> Schrod<()> {
        // the node being removed
        let node_result = self.get_mut(node_id);
        if node_result.is_fail() {
            return node_result
                .convert("Pool::remove_node()")
                .fail("Failed to remove node.", "Pool::remove_node()")
        }
        let node = node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::remove_node()");

        // adding the node's children to the node's parent and setting the children's parents as the nodes parent
        match node.get_parent_id() {
            // if the node has a parent
            Some(parent_id) => {
                // gets the parent
                let parent_node_result = self.get_mut(parent_id);
                if parent_node_result.is_fail() {
                    return parent_node_result
                        .convert("Pool::remove_node()")
                        .fail("Failed to remove node.", "Pool::remove_node()")
                }
                let parent_node = parent_node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::remove_node()");

                // updates the node's parent to have all the node's children
                let children_ids = node.get_children_ids();
                parent_node.add_children(children_ids.clone());
                
                // updates all the node's children to have the node's parent as their parent
                children_ids.into_iter().for_each(|id| {
                    let child_node_result = self.get_mut(id);
                    if child_node_result.is_fail() {
                        return child_node_result
                            .convert("Pool::remove_node()")
                            .fail("Failed to remove node.", "Pool::remove_node()")
                    }
                    let child_node = child_node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::remove_node()");
                    child_node.set_parent(Some(parent_id));
                });
            }

            // if the node has no parent
            None => {
                // updates all the node's children to have the node's parent (the pool) as their parent
                let children_ids = node.get_children_ids();
                children_ids.into_iter().for_each(|id| {
                    let child_node_result = self.get_mut(id);
                    if child_node_result.is_fail() {
                        return child_node_result
                            .convert("Pool::remove_node()")
                            .fail("Failed to remove node.", "Pool::remove_node()")
                    }
                    let child_node = child_node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::remove_node()");
                    child_node.set_parent(None);
                });
            }
        }

        // removing the node
        self.nodes.retain(|existing_node| existing_node !=  node);

        // succeeds
        Pass(())
    }
}



#[derive(Debug, Clone)]
pub struct Node {
    id: Uuid,
    branch_id: Uuid,
    parent_id: Option<Uuid>,
    children_ids: Vec<Uuid>,
    operation: Operation,
    image: Option<ImageBuffer<Rgba<f32>, Vec<f32>>>,
}
impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.id == other.id
    }
}
impl Node {
    // initializing
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
    pub fn get_id(&self) -> Uuid { self.id }

    pub fn get_branch_id(&self) -> Uuid { self.branch_id }
    
    pub fn get_parent_id(&self) -> Option<Uuid> { self.parent_id }

    pub fn has_child(&self) -> bool { self.children_ids.len() > 0 }

    pub fn get_children_ids(&self) -> Vec<Uuid> { self.children_ids.clone() }
    
    pub fn get_operation(&self) -> Operation { self.operation }

    pub fn get_image(&mut self, pool: &Pool) -> Schrod<&ImageBuffer<Rgba<f32>, Vec<f32>>> {
        let update_result = self.update_image(pool);
        match update_result {
            Pass(()) => {
                if let Some(image) = &self.image { return Pass(image) }
                return Schrod::new_fail("Failed to get image after successful update!", "Node::get_image()").fail("Failed to get image.", "Node::get_image()")
            }
            Fail(_) => {
                return update_result
                    .convert("Node::get_image()")
                    .fail("Failed to get image.", "Node::get_image()")
            }
        }
    }



    // basic editing
    fn set_parent(&mut self, parent_id: Option<Uuid>) {
        self.parent_id = parent_id;
    }
    
    fn add_child(&mut self, child_id: Uuid) {
        self.children_ids.push(child_id);
    }

    fn add_children(&mut self, children_ids: Vec<Uuid>) {
        self.children_ids.extend(children_ids);
    }

    fn remove_child(&mut self, child_id: Uuid) -> Schrod<()> {
        if !self.children_ids.contains(&child_id) {
            return Schrod::new_fail(&format!("{child_id} is not a child of {}!", self.id), "Node::remove_child()")
                .fail("Failed to remove child.", "Node::remove_child()")
        }
        self.children_ids.retain(|id| *id != child_id);

        Pass(())
    }

    fn edit_operation(&mut self, new_operation: Operation) {
        self.operation = new_operation;
    }

    

    // working with images
    fn update_image(&mut self, pool: &Pool) -> Schrod<()> {
        let new_image_result = self.generate_image(pool);
        if new_image_result.is_fail() {
            return new_image_result
                .convert("Node::update_image()")
                .fail("Failed to generate image.", "Node::update_image()")
        }
        let new_image = new_image_result.wont_fail("This is past an is_fail() guard clause.", "Node::update_image()");
        self.image = Some(new_image);
        Pass(())
    }

    fn generate_image(&self, pool: &Pool) -> Schrod<ImageBuffer<Rgba<f32>, Vec<f32>>> {
        // returns the node's own image if it has already been created
        if let Some(image) = &self.image { return Pass(image.clone()) } // expensive clone?

        // checks if this node has a parent
        match self.get_parent_id() {
            // generates a new image based on this node's parent
            Some(parent_id) => {
                let parent_result = pool.get(parent_id);
                if parent_result.is_fail() {
                    return parent_result
                        .convert("Node::generate_image()")
                        .fail("Failed to generate image.", "Node::generate_image()")
                }
                let parent = parent_result.wont_fail("This is past an is_fail() guard clause.", "Node::generate_image()");
                let previous_image_result =  parent.generate_image(&pool);
                if previous_image_result.is_fail() { return previous_image_result.fail("Failed to generate image.", "Node::generate_image()") }
                let mut previous_image = previous_image_result.wont_fail("This is past an is_fail() guard clause.", "Node::generate_image()");
    
                self.get_operation().apply_to(&mut previous_image);
                let new_image = previous_image;
                return Pass(new_image)
            }
            
            // generates a new image based on the source image of the pool
            None => {
                if let Some(source_image) = pool.get_image() {
                    let mut new_image = source_image.clone();
                    self.get_operation().apply_to(&mut new_image);
                    return Pass(new_image)
                }
                else {
                    return Schrod::new_fail("No source image found!", "Node::generate_image()")
                        .fail("Failed to generate image.", "Node::generate_image()")
                }
            }
        }
    }

    fn snip(&mut self) {
        self.image = None;
    }
}