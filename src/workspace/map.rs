use iced::{Point};
use schrod::Schrod::{self, Pass};

use crate::workspace::{node::Node, tree::Tree};

pub struct Map<'a> {
    nodules: Vec<Nodule<'a>>,
}
impl<'a> Map<'a> {
    pub fn new(tree: &'a Tree) -> Schrod<()> {
        // gets the end point nodes
        let end_point_ids_result = tree.get_all_downstream_nodes(tree.get_root().get_id());
        if end_point_ids_result.is_fail() {
            return end_point_ids_result
                .convert("Map::new()")
                .fail("Failed to create Map.", "Map::new()")
        }
        let end_point_ids = end_point_ids_result.wont_fail("This is past an is_fail() guard clause.", "Map::new()");

        // creates a list of nodules for each branch
        // the root node is excluded and will be added later
        let mut nodules: Vec<Nodule<'a>> = Vec::new();
        for end_point_id in end_point_ids {
            // gets the end point node
            let end_point_result = tree.get(end_point_id);
            if end_point_result.is_fail() {
                return end_point_result
                    .convert("Map::new()")
                    .fail("Failed to create Map.", "Map::new()")
            }
            let end_point = end_point_result.wont_fail("This is past an is_fail() guard clause.", "Map::new()");

            // creates a new nodule with the end point node
            let mut nodule = Nodule::new(end_point);

            // gets the first parent to start moving up each branch
            let first_parent_result = tree.get(end_point.get_parent_id());
            if first_parent_result.is_fail() {
                return first_parent_result
                    .convert("Map::new()")
                    .fail("Failed to create Map.", "Map::new()")
            }
            let first_parent = first_parent_result.wont_fail("This is past an is_fail() guard clause.", "Map::new()");

            // the current node being explored
            let mut current_node = first_parent;

            // explores each node in the same branch as the end point
            // this excludes the root node
            loop {
                // ends the loop if the branch has been fully explored
                if current_node.is_root() || current_node.get_branch_id() != end_point.get_branch_id() { break; }

                // adds the current node to the nodule
                nodule.add(current_node);

                // gets the next node to explore
                let next_node_result = tree.get(current_node.get_parent_id());
                if next_node_result.is_fail() {
                    return next_node_result
                        .convert("Map::new()")
                        .fail("Failed to create Map.", "Map::new()")
                }
                current_node = next_node_result.wont_fail("This is past an is_fail() guard clause.", "Map::new()");
            }

            // adds the nodule to the list of nodules
            nodules.push(nodule);
        }

        // adds the root to the right nodule
        let mut added_root = false;
        for nodule in &mut nodules {
            if nodule.get_base().get_branch_id() == tree.get_root().get_branch_id() {
                nodule.add(tree.get_root());
                added_root = true;
            }
        }

        // fails if the right branch for the root could not be found
        if !added_root {
            return Schrod::new_fail("Could not find the root's branch!", "Map::new()")
                .fail("Failed to create Map.", "Map::new()")
        }

        Pass(())
    }
}



pub struct Nodule<'a> {
    nodes: Vec<&'a Node>
}
impl<'a> Nodule<'a> {
    fn new(node: &'a Node) -> Nodule<'a> {
        Nodule { nodes: vec![node] }
    }

    fn add(&mut self, node: &'a Node) {
        self.nodes.push(node);
    }

    fn get_base(&self) -> &'a Node {
        self.nodes[self.nodes.len() - 1]
    }
}