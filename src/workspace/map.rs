use iced::{Point};
use schrod::Schrod::{self, Pass};
use uuid::Uuid;

use crate::workspace::{node::Node, tree::Tree};

pub struct Map<'a> {
    branch_maps: Vec<BranchMap<'a>>,
}
impl<'a> Map<'a> {
    pub fn new(tree: &'a Tree) -> Schrod<()> {
        // gets the branch maps
        let branch_maps_result = BranchMap::build_branch_maps(tree);
        if branch_maps_result.is_fail() {
            return branch_maps_result
                .convert("Map::new()")
                .fail("Failed to create Map.", "Map::new()")
        }
        let branch_maps = branch_maps_result.wont_fail("This is past an is_fail() guard clause.", "Map::new()");

        // gets the root branch map
        let mut root_branch_map_result = None;
        for branch_map in &branch_maps {
            if branch_map.get_branch_id() == tree.get_root().get_branch_id() { root_branch_map_result = Some(branch_map); }
        }
        let root_branch_map_result = Schrod::from_option(root_branch_map_result, "Failed to find the root's branch to start assembling!", "Map::new()");
        if root_branch_map_result.is_fail() {
            return root_branch_map_result
                .convert("Map::new()")
                .fail("Failed to create Map.", "Map::new()")
        }
        let root_branch_map = root_branch_map_result.wont_fail("This is past an is_fail() guard clause.", "Map::new()");

        // starts assembling the branch maps into an overall node grid
        for node in &root_branch_map.nodules {
            
        }


        
        Pass(())
    }
}



pub struct BranchMap<'a> {
    /// The list of `Nodule`s that make up the `BranchMap`.
    /// The first `Node`/`Nodule` is the end point in its corresponding branch and the
    /// last `Node`/`Nodule` is the base of the branch.
    nodules: Vec<Nodule<'a>>
}
impl<'a> BranchMap<'a> {
    // initializing
    /// Creates a new `BranchMap`.
    #[must_use]
    pub fn new(node: &'a Node) -> BranchMap<'a> {
        let first_nodule = Nodule::new(node, 0);
        BranchMap { nodules: vec![first_nodule] }
    }



    // building
    /// Adds a new `Node` to the `BranchMap` upstream of last `Node`.
    pub fn add_node_upstream(&mut self, node: &'a Node) {
        let last_y = self.nodules[self.nodules.len() - 1].get_y();
        let new_nodule = Nodule::new(node, last_y + 1);
        self.nodules.push(new_nodule);
    }

    /// Builds a collection `BranchMap`s from the given `Tree`.
    #[must_use]
    pub fn build_branch_maps(tree: &'a Tree) -> Schrod<Vec<BranchMap<'a>>> {
        // gets the end point nodes
        let end_point_ids_result = tree.get_all_downstream_nodes(tree.get_root().get_id());
        if end_point_ids_result.is_fail() {
            return end_point_ids_result
                .convert("BranchMap::build_branch_maps()")
                .fail("Failed to create BranchMap.", "BranchMap::build_branch_maps()")
        }
        let end_point_ids = end_point_ids_result.wont_fail("This is past an is_fail() guard clause.", "BranchMap::build_branch_maps()()");


        
        // creates a list of branch maps for each branch
        // the root node is excluded and will be added later
        let mut branch_maps: Vec<BranchMap<'a>> = Vec::new();
        for end_point_id in end_point_ids {
            // gets the end point node
            let end_point_result = tree.get(end_point_id);
            if end_point_result.is_fail() {
                return end_point_result
                    .convert("BranchMap::build_branch_maps()")
                    .fail("Failed to create BranchMap.", "BranchMap::build_branch_maps()")
            }
            let end_point = end_point_result.wont_fail("This is past an is_fail() guard clause.", "BranchMap::build_branch_maps()()");

            // creates a new branch map with the end point node
            let mut branch_map = BranchMap::new(end_point);

            // gets the first parent to start moving up each branch
            let first_parent_result = tree.get(end_point.get_parent_id());
            if first_parent_result.is_fail() {
                return first_parent_result
                    .convert("BranchMap::build_branch_maps()")
                    .fail("Failed to create BranchMap.", "BranchMap::build_branch_maps()")
            }
            let first_parent = first_parent_result.wont_fail("This is past an is_fail() guard clause.", "BranchMap::build_branch_maps()()");

            // the current node being explored
            let mut current_node = first_parent;

            // explores each node in the same branch as the end point
            // this excludes the root node
            loop {
                // ends the loop if the branch has been fully explored
                if current_node.is_root() || current_node.get_branch_id() != end_point.get_branch_id() { break; }

                // adds the current node to the branch map
                branch_map.add_node_upstream(current_node);

                // gets the next node to explore
                let next_node_result = tree.get(current_node.get_parent_id());
                if next_node_result.is_fail() {
                    return next_node_result
                        .convert("BranchMap::build_branch_maps()")
                        .fail("Failed to create BranchMap.", "BranchMap::build_branch_maps()")
                }
                current_node = next_node_result.wont_fail("This is past an is_fail() guard clause.", "BranchMap::build_branch_maps()()");
            }

            // adds the branch map to the list of branch map
            branch_maps.push(branch_map);
        }


        
        // adds the root to the right branch map
        let mut added_root = false;
        for branch_map in &mut branch_maps {
            if branch_map.get_branch_id() == tree.get_root().get_branch_id() {
                branch_map.add_node_upstream(tree.get_root());
                added_root = true;
            }
        }

        // fails if the right branch for the root could not be found
        if !added_root {
            return Schrod::new_fail("Could not find the root's branch to add the root Node!", "BranchMap::build_branch_maps()()")
                .fail("Failed to create BranchMap.", "BranchMap::build_branch_maps()")
        }

        // returns the branch maps
        Pass(branch_maps)
    }



    // basic getters
    /// Gets the branch id of the `BranchMap`.
    #[must_use]
    pub fn get_branch_id(&self) -> Uuid { self.nodules[0].node.get_branch_id() }
}



/// Holds a reference to a `Node` and it's position relative to other `Node`s/`Nodule`s.
pub struct Nodule<'a> {
    /// The `Node` being referenced.
    node: &'a Node,
    ///How far out sideways this branch is visually.
    x: u8,
    /// How far down the tree the `Node` is.
    y: u8,
}
impl<'a> Nodule<'a> {
    // initializing
    /// Creates a new `Nodule`.
    /// The position is set up in stages. First the `BranchMap`s are created and each `Nodule`
    /// it contains is given a position relative only to the other `Nodule`s in that `BranchMap`.
    /// Then when all the `BranchMap`s are combined the positions are updated to be relative to
    /// the same overall grid/map.
    #[must_use]
    pub fn new(node: &'a Node, y: u8) -> Nodule {
        Nodule { node, x: 0, y }
    }


    
    // basic getters
    /// Gets the `x` position.
    #[must_use]
    pub fn get_x(&self) -> u8 { self.x }

    /// Gets the `y` position.
    #[must_use]
    pub fn get_y(&self) -> u8 { self.y }



    // basic editing
    /// Sets the `x` position.
    pub fn set_x(&mut self, new_x: u8) { self.x = new_x }

    /// Sets the `y` position.
    pub fn set_y(&mut self, new_y: u8) { self.y = new_y }
}