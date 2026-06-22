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
        if self.source_image.is_none() {
            return Schrod::new_fail("No source image found!", "Pool::add_node()")
                .fail("Failed to add node.", "Pool::add_node()")
        }

        match self.position {
            Some(position) => {
                let parent_node_result = self.get(position);
                if parent_node_result.is_fail() {
                    return parent_node_result
                        .convert("Pool::add_node()")
                        .fail("Failed to add node.", "Pool::add_node()")
                }
                let parent_node = parent_node_result.wont_fail("This is past an is_fail() guard clause.", "Pool::add_node()");
                let new_node = Node::new(Some(parent_node.get_id()), parent_node.get_branch_id(), operation);
                self.position = Some(new_node.get_id());
                self.nodes.push(new_node);
            }

            None => {
                let new_node = Node::new(None, Uuid::now_v7(), operation);
                self.position = Some(new_node.get_id());
                self.nodes.push(new_node);
            }
        }



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

    /*
    pub fn get_parent_pool_index(&self, pool: &Pool) -> Schrod<Option<usize>> {
        match self.parent_id {
            Some(parent_id) => {
                for (index, node) in pool.nodes.iter().enumerate() {
                    if node.id == parent_id {
                        return Pass(Some(index));
                    }
                }
                Schrod::new_fail("Parent id does not exist in pool!", "Node::get_parent_pool_index()")
                    .fail("Failed to get parent pool index.", "Node::get_parent_pool_index()")
            }
            
            None => Pass(None)
        }
    }
    */

    /*
    pub fn get_parent<'a>(&'a self, pool: &'a Pool) -> Schrod<Option<&'a Node>> {
        match self.parent_id {
            Some(parent_id) => {
                let parent_result = pool.get(parent_id);
                match parent_result {
                    Pass(parent) => { Pass(Some(parent)) }
                    Fail(_) => {
                        return parent_result
                            .convert("Node::get_parent()")
                            .fail("Failed to get parent.", "Node::get_parent()")
                    }
                }
            }

            None => Pass(None)
        }
    }
    */
    
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