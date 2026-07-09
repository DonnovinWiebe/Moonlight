use iced::widget::canvas::{Fill, Frame, Path};
use iced::{Point};
use materialui::{components::ThemeProvider, material::{Depths, MaterialColors, Materials}};
use schrod::Schrod::{self, Pass};
use uuid::Uuid;

use crate::{state::app::App, workspace::{node::Node, tree::Tree}};

pub struct Map<'a> {
    nodules: Vec<Nodule<'a>>,
}
impl<'a> Map<'a> {
    // initializing
    /// Creates a new `Map`.
    #[must_use]
    pub fn new(tree: &'a Tree) -> Schrod<Map<'a>> {
        // builds individual branch maps
        let branch_maps_result = BranchMap::build_branch_maps(tree);
        if branch_maps_result.is_fail() {
            return branch_maps_result
                .convert("Map::new()")
                .fail("Failed to create new Map.", "Map::new()");
        }
        let branch_maps = branch_maps_result.wont_fail("This is past an is_fail() guard clause.", "Map::new()");

        // assembles the nodules from the branch maps
        let assembled_nodules_result = Map::assemble_nodules(tree, branch_maps);
        if assembled_nodules_result.is_fail() {
            return assembled_nodules_result
                .convert("Map::new()")
                .fail("Failed to create new Map.", "Map::new()");
        }
        let assembled_nodules = assembled_nodules_result.wont_fail("This is past an is_fail() guard clause.", "Map::new()");

        // returns a new Map
        Pass(Map { nodules: assembled_nodules })
    }
    
    /// Assembles a list of `BranchMap`s into a collection of positioned `Nodule`s.
    #[must_use]
    fn assemble_nodules(tree: &'a Tree, mut branch_maps: Vec<BranchMap<'a>>) -> Schrod<Vec<Nodule<'a>>> {
        // the nodules that have been positioned
        let mut mapped_nodules: Vec<Nodule<'a>> = Vec::new();
        // the current x position of brances so they do not overlap
        let mut current_branch_x = 0;
        // tracks how far down the tree the last nodule was in order to properly offset the
        // nodules in newly discovered branches
        let mut last_nodule_y = 0;
        // the root branch id
        let root_branch_id = tree.get_root().get_branch_id();
        // the queue of branches being explored
        let mut current_branch_ids = vec![root_branch_id];
        // the branches that have been seen/worked with
        // this is used to determine if the given branch needs to have its nodules shifted to the side (x offset)
        let mut branches_touched = vec![root_branch_id];

        // loops until all nodules have been examined
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
                for nodule in current_branch_map.get_nodules_mut() { nodule.add_position_offset(current_branch_x, last_nodule_y); }
            }

            // checks if the current branch has a new node to explore
            let current_nodule_result = current_branch_map.get_next_nodule();
            match current_nodule_result {
                // places the next nodule into the map
                Some(nodule) => {
                    // updates the last y
                    last_nodule_y = nodule.get_y();
                    
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



    // drawing
    /// How much space is around the `Nodule` tree in the `Map`.
    const PADDING: u32 = 16;
    
    /// Gets the overall size of the rendered `Map`.
    #[must_use]
    pub fn get_bounds(&self) -> (u32, u32) {
        let mut largest_x = 0;
        let mut largest_y = 0;
        
        for nodule in &self.nodules {
            if nodule.get_x() > largest_x { largest_x = nodule.get_x(); }
            if nodule.get_y() > largest_y { largest_y = nodule.get_x(); }
        }
        
        (
            largest_x + (Map::PADDING * 2),
            largest_y + (Map::PADDING * 2),
        )
    }
}



/// Holds the `Nodule`s that make up a branch in the `Tree`.
/// This is only used while constructing a `Map`.
#[derive(Debug, Clone)]
struct BranchMap<'a> {
    /// The list of `Nodule`s that make up the `BranchMap`.
    /// The first `Node`/`Nodule` is the end point in its corresponding branch and the
    /// last `Node`/`Nodule` is the base of the branch.
    nodules: Vec<Nodule<'a>>,
    /// Tracks the index of the next 'Nodule' being explored when assembling 'Nodule's
    /// into the final `Map`.
    next_nodule: usize,
}
impl<'a> BranchMap<'a> {
    // building
    /// Builds a collection `BranchMap`s from the given `Tree`.
    #[must_use]
    fn build_branch_maps(tree: &'a Tree) -> Schrod<Vec<BranchMap<'a>>> {
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
    
    /// Creates a new `BranchMap`.
    /// This is only used in building a collection of `BranchMap`s from a given `Tree`.
    #[must_use]
    fn new(node: &'a Node) -> BranchMap<'a> {
        let first_nodule = Nodule::new(node, 0);
        BranchMap { nodules: vec![first_nodule], next_nodule: 0 }
    }
    
    /// Adds a new `Node` to the `BranchMap` upstream of last `Node`.
    fn add_node_upstream(&mut self, node: &'a Node) {
        let last_y = self.nodules[self.nodules.len() - 1].get_y();
        let new_nodule = Nodule::new(node, last_y + 1);
        self.nodules.push(new_nodule);
    }

    /// Updates all the `Nodule` positions to be relative to the `Nodule` it branches off from.
    fn set_branch_attachment(&mut self, source_nodule: &Nodule) {
        let base_x = source_nodule.get_x() + 1;
        let base_y = source_nodule.get_y() + 1;
        for nodule in &mut self.nodules { nodule.add_position_offset(base_x, base_y); }
    }
    


    // basic getters
    /// Gets an immutable reference to the `Nodule`s.
    #[must_use]
    fn get_nodules(&self) -> &Vec<Nodule<'a>> { &self.nodules }
    
    /// Gets a mutable reference to the `Nodule`s.
    #[must_use]
    fn get_nodules_mut(&mut self) -> &mut Vec<Nodule<'a>> { &mut self.nodules }
    
    /// Gets the branch id of the `BranchMap`.
    #[must_use]
    fn get_branch_id(&self) -> Uuid { self.nodules[0].get_node().get_branch_id() }



    // parsing
    /// Gets the next `Nodule` being explored when assembling all `Nodule`s into a `Map`.
    #[must_use]
    fn get_next_nodule(&mut self) -> Option<Nodule<'a>> {
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
struct Nodule<'a> {
    /// The `Node` being referenced.
    node: &'a Node,
    ///How far out sideways this branch is visually.
    x: u32,
    /// How far down the tree the `Node` is.
    y: u32,
}
impl<'a> Nodule<'a> {
    // initializing
    /// Creates a new `Nodule`.
    /// The position is set up in stages. First the `BranchMap`s are created and each `Nodule`
    /// it contains is given a position relative only to the other `Nodule`s in that `BranchMap`.
    /// Then when all the `BranchMap`s are combined the positions are updated to be relative to
    /// the same overall grid/map.
    #[must_use]
    fn new(node: &'a Node, y: u32) -> Nodule<'a> {
        Nodule { node, x: 0, y }
    }


    
    // basic getters
    /// Gets the `Node`.
    fn get_node(&self) -> &Node { &self.node }
    
    /// Gets the `x` position.
    #[must_use]
    fn get_x(&self) -> u32 { self.x }

    /// Gets the `y` position.
    #[must_use]
    fn get_y(&self) -> u32 { self.y }



    // basic editing
    /// Adds a position offset.
    fn add_position_offset(&mut self, offset_x: u32, offset_y: u32) {
        self.x += offset_x;
        self.y += offset_y;
    }
    
    /// Sets the `x` position.
    fn set_x(&mut self, new_x: u32) { self.x = new_x }

    /// Sets the `y` position.
    fn set_y(&mut self, new_y: u32) { self.y = new_y }



    // drawing
    /// The size of the `Nodule` in the `Map`.
    const NODE_SIZE: f32 = Nodule::NODE_SYMBOL_SIZE + (2.0 * Nodule::NODE_INNER_PADDING);
    
    /// The size of the `Operation` symbol in the `Map`.
    const NODE_SYMBOL_SIZE: f32 = 16.0;
    
    /// How much space there is between the `Nodule`'s symbol and outer edges in the `Map`.
    const NODE_INNER_PADDING: f32 = 8.0;
    
    /// How round the `Nodule`'s corners are in the `Map`.
    const NODE_CORNER_RADIUS: f32 = 4.0;
    
    /// The length of the straight sections in the `Nodule`'s outer edges in the `Map`.
    const NODE_STRAIGH_WALL_LENGTH: f32 = Nodule::NODE_SIZE - (2.0 * Nodule::NODE_CORNER_RADIUS);
    
    /// The space between `Nodule`s in the `Map`.
    const NODE_SPACING: f32 = Nodule::NODE_SIZE * 2.0;
    
    /// Draws the given `Nodule` in the `Map`.
    fn draw_into(&self, app: &App, frame: &mut Frame, is_selected: bool) {
        // getting the nodule shape path
        let shape = Path::new(|pen| {
            // top left of the nodule
            let x = (self.x as f32 * Nodule::NODE_SPACING) - (Nodule::NODE_SIZE / 2.0);
            let y = (self.y as f32 * Nodule::NODE_SPACING) - (Nodule::NODE_SIZE / 2.0);
            // wall dimensions
            let r = Nodule::NODE_CORNER_RADIUS;
            let w = Nodule::NODE_STRAIGH_WALL_LENGTH;
            
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
        frame.fill_text(self.get_node().get_operation().canvas_icon(app));
    }
}