use iced::Point;
use schrod::Schrod::{self, Pass};
use uuid::Uuid;

use crate::workspace::{node::Node, tree::Tree};

pub struct Map<'a> {
    nodules: Vec<Nodule<'a>>,
}
impl<'a> Map<'a> {
    pub fn assemble_nodules(tree: &'a Tree, mut branch_maps: Vec<BranchMap<'a>>) -> Schrod<Vec<Nodule<'a>>> {
        // the nodules that have been positioned
        let mut mapped_nodules: Vec<Nodule<'a>> = Vec::new();
        // the current x position of brances so they do not overlap
        let mut current_branch_x = 0;
        // the root branch id
        let root_branch_id = tree.get_root().get_branch_id();
        // the queue of branches being explored
        let mut current_branch_ids = vec![root_branch_id];
        // the branches that have been seen/worked with
        // this is used to determine if the given branch needs to have its nodules shifted to the side (x offset)
        let mut branches_touched = vec![root_branch_id];

        loop {
            // ends the loop if there are no more branched to explore
            if current_branch_ids.is_empty() { break; }

            // gets the current branch map
            let current_branch_map_result = 'block: {
                for map in &mut branch_maps {
                    if map.get_branch_id() == current_branch_ids[current_branch_ids.len() - 1] { break 'block Some(map) }
                }
                None
            };
            if current_branch_map_result.is_none() {
                return Schrod::from_option(current_branch_map_result, "Failed to find current Branch Map!", "Map::assemble_nodules()")
                    .convert("Map::assemble_nodules()")
                    .fail("Failed to assemble Nodules.", "Map::assemble_nodules()")
            }
            let current_branch_map = current_branch_map_result.expect("This is past an is_none() guard clause.");
            
            // updates the y position of all nodules in the current branch map if it has not been worked with up to this point
            if !branches_touched.contains(&current_branch_ids[current_branch_ids.len() - 1]) {
                current_branch_x += 1;
                branches_touched.push(current_branch_ids[current_branch_ids.len() - 1]);
                for nodule in current_branch_map.get_nodules_mut() { nodule.add_position_offset(current_branch_x, 0); }
            }

            // checks if the current branch has a new node to explore
            let current_nodule_result = current_branch_map.get_next_nodule();
            match current_nodule_result {
                // places the next nodule into the map
                Some(nodule) => {
                    // gets any new downstream branches
                    let downstream_branches_result = nodule.get_node().get_other_downstream_branches(tree);
                    if downstream_branches_result.is_fail() {
                        return downstream_branches_result
                            .convert("Map::assemble_nodules()")
                            .fail("Failed to assemble Nodules.", "Map::assemble_nodules()")
                    }
                    let downstream_branche_ids = downstream_branches_result.wont_fail("This is past an is_fail() guard clause.", "Map::assemble_nodules()");

                    // adds new branches to the queue
                    current_branch_ids.extend(downstream_branche_ids);

                    // adds the current nodule to the mapped nodules
                    mapped_nodules.push(nodule);
                }

                // removes the current branch from the queue if it has no more nodules to explore
                None => {
                    current_branch_ids.remove(current_branch_ids.len() - 1);
                }
            }
        }

        // returns the mapped nodules
        Pass(mapped_nodules)
    }
}



#[derive(Debug, Clone)]
pub struct BranchMap<'a> {
    /// The list of `Nodule`s that make up the `BranchMap`.
    /// The first `Node`/`Nodule` is the end point in its corresponding branch and the
    /// last `Node`/`Nodule` is the base of the branch.
    nodules: Vec<Nodule<'a>>,
    /// Tracks the index of the next 'Nodule' being explored when assembling 'Nodule's
    /// into the final `Map`.
    next_nodule: usize,
}
impl<'a> BranchMap<'a> {
    // initializing
    /// Creates a new `BranchMap`.
    #[must_use]
    pub fn new(node: &'a Node) -> BranchMap<'a> {
        let first_nodule = Nodule::new(node, 0);
        BranchMap { nodules: vec![first_nodule], next_nodule: 0 }
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

    /// Updates all the `Nodule` positions to be relative to the `Nodule` it branches off from.
    pub fn set_branch_attachment(&mut self, source_nodule: &Nodule) {
        let base_x = source_nodule.get_x() + 1;
        let base_y = source_nodule.get_y() + 1;
        for nodule in &mut self.nodules { nodule.add_position_offset(base_x, base_y); }
    }



    // basic getters
    /// Gets an immutable reference to the `Nodule`s.
    #[must_use]
    pub fn get_nodules(&self) -> &Vec<Nodule<'a>> { &self.nodules }
    
    /// Gets a mutable reference to the `Nodule`s.
    #[must_use]
    pub fn get_nodules_mut(&mut self) -> &mut Vec<Nodule<'a>> { &mut self.nodules }
    
    /// Gets the branch id of the `BranchMap`.
    #[must_use]
    pub fn get_branch_id(&self) -> Uuid { self.nodules[0].get_node().get_branch_id() }



    // parsing
    /// Gets the next `Nodule` being explored when assembling all `Nodule`s into a `Map`.
    #[must_use]
    pub fn get_next_nodule(&mut self) -> Option<Nodule<'a>> {
        if self.next_nodule < self.nodules.len() - 1 {
            let nodule = self.nodules[self.next_nodule].clone();
            self.next_nodule += 1;
            Some(nodule)
        }
        else { None }
    }
}



/// Holds a reference to a `Node` and it's position relative to other `Node`s/`Nodule`s.
#[derive(Debug, Clone)]
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
    /// Gets the `Node`.
    pub fn get_node(&self) -> &Node { &self.node }
    
    /// Gets the `x` position.
    #[must_use]
    pub fn get_x(&self) -> u8 { self.x }

    /// Gets the `y` position.
    #[must_use]
    pub fn get_y(&self) -> u8 { self.y }



    // basic editing
    /// Adds a position offset.
    pub fn add_position_offset(&mut self, offset_x: u8, offset_y: u8) {
        self.x += offset_x;
        self.y += offset_y;
    }
    
    /// Sets the `x` position.
    pub fn set_x(&mut self, new_x: u8) { self.x = new_x }

    /// Sets the `y` position.
    pub fn set_y(&mut self, new_y: u8) { self.y = new_y }
}