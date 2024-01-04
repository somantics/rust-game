use std::cmp::{min, max};
use std::collections::HashSet;

use petgraph::visit::IntoNodeReferences;
use rand::prelude::*;
use rand_distr::StandardNormal;

use petgraph::graph::{Graph, NodeIndex};
use petgraph::stable_graph::StableGraph;

use crate::tile;
use crate::{
  map::{Coordinate, Euclidian, GameMap}, 
  tile::{GameTile, TileID}
};


/* Algorithm sketch
make tree
  split till correct level

populate leafs with rooms
  rooms are defined with box extends of coordinates

connect rooms with corridors
  make locally fully connected
    connect within leaf
    connect pairwise one level up (checking smallest distance)
    connect two pairs one level up (checking smallest distance)
    connect four pairs one level up
    connect four pairs
    connect eight pairs (doublecheck the math on this)
    continue till reached root
        OR
    use collisionboxes for the areas to test nearby rooms, regardless of minimal tree
      currently testing this

  trim down w/o breaking access to each room
    select random room w. atleast 3 edges
    trim edge
    check connectivity
    confirm trim

    repeat til ?
  no dead end rooms

tier rooms from start to finish
  select start and end
  base this on distance to start

populate rooms with encounters
  put connected encounters in same or one step earlier tier (will this be necessary on small map?)
  ensure room distance is small enough

*/

pub enum Axis {
  Horizontal,
  Vertical
}

pub struct MapBuilder {
  // this a bit is awkward
}

impl MapBuilder {

  pub fn generate_new(size_x: u32, size_y: u32) -> GameMap {

    let bsp_tree = MapBuilder::binary_space_partitioning(size_x, size_y);
    let graph = MapBuilder::make_rooms_from_bsp(&bsp_tree);
    let graph = MapBuilder::make_connected_graph(&graph, 2);

    MapBuilder::draw_rooms_to_map(&graph, size_x, size_y)
  }

  fn binary_space_partitioning(size_x: u32, size_y: u32) -> Graph<BoxExtends, (), petgraph::Undirected> {

    let mut graph = Graph::<BoxExtends,(), petgraph::Undirected>::new_undirected();
    let map_box = BoxExtends {
      top_left: Coordinate::default(),
      bottom_right: Coordinate { x: (size_x - 1) as i32, y: (size_y - 1) as i32}
    };

    let origin = graph.add_node( map_box );
    MapBuilder::split_branch(origin, &mut graph, 0, 4);

    graph
  }

  fn split_branch(parent: NodeIndex, graph: &mut Graph<BoxExtends, (), petgraph::Undirected>, current_depth: usize, max_depth: usize) {

    if current_depth >= max_depth {
      return;
    }

    let parent_box = graph.node_weight(parent).unwrap();
    let (a, b) = BoxExtends::split_box(parent_box);
    let branch_a = graph.add_node( a );
    let branch_b = graph.add_node( b );

    graph.extend_with_edges(&[
        (parent, branch_a), 
        (parent, branch_b)
    ]);

    MapBuilder::split_branch(branch_a, graph, current_depth + 1, max_depth);
    
    MapBuilder::split_branch(branch_b, graph, current_depth + 1, max_depth);
  }

  fn make_rooms_from_bsp(bsp_tree: &Graph<BoxExtends, (), petgraph::Undirected>) -> StableGraph<BoxExtends,(), petgraph::Undirected> {
    
    let bsp_leaves = MapBuilder::leaves_from_bsp(&bsp_tree);
    let mut graph = StableGraph::<BoxExtends,(), petgraph::Undirected>::default();

    for index in bsp_leaves {
      let room_box = match bsp_tree.node_weight(index) {
        Some(extends) => BoxExtends::random_subbox(extends, 3),
        None => continue,
      };

      graph.add_node(room_box);
    };

    graph
  }

  fn leaves_from_bsp<'a>(graph: &'a Graph<BoxExtends, (), petgraph::Undirected>) -> impl Iterator<Item = NodeIndex > + 'a {

    graph.node_indices()
      .filter(|index| {
        graph.neighbors_undirected(*index).count() == 1
      })
  }

  fn make_connected_graph(room_graph: &StableGraph::<BoxExtends,(), petgraph::Undirected>, max_scan_distance: i32) -> StableGraph<BoxExtends, (), petgraph::prelude::Undirected> {
    /* making edges for rooms where doors/corridors should be
      pick a node
        remove node from unprocessed list
        generate neighbor overlap boxes for that node's area
          orthogonal directions in distance that's exposed as variable should do it
        combare overlap boxes with un-processed nodes
          make an edge for each match
          put matched nodes into opened list
        
        if opened list is empty: we found an island, repeat process with random node
        else
          pick neighbor as next node (aka from the opened list)

      repeat till no unproccessed or opened node left 
    */

    let mut new_graph = StableGraph::<BoxExtends,(), petgraph::Undirected>::default();
    new_graph.clone_from(room_graph); // not explicit that we're only getting nodes w/o edges here

    let mut unprocessed = room_graph.node_references(); // this moves room_graph
    let mut opened: Vec<(NodeIndex, &BoxExtends)> = vec![];
    let mut closed: Vec<NodeIndex> = vec![];

    let mut current_node: NodeIndex;
    let mut current_area: &BoxExtends;


    loop {
      if opened.len() == 0 {
        (current_node, current_area) = match unprocessed.next() {
          Some(tuple) => tuple,
          None => break,
        };
      } else {
        (current_node, current_area) = match opened.pop() {
          Some(tuple) => tuple,
          None => break,
        };
      }

      closed.push(current_node);

      let collision_boxes: Vec<BoxExtends> = BoxExtends::make_edge_vicinity_boxes(&current_area, max_scan_distance);


      let neighbors = unprocessed
        .clone()
        .filter(|(_, area)| {
          collision_boxes.iter().any(|collision| area.overlaps(collision))
        })
        .filter(|(index, _)| !closed.contains(index));
      

      opened.extend(
        neighbors.clone()
      );

      new_graph.extend_with_edges(neighbors.map(|(index,_) | (current_node, index)));
    }

    new_graph
  }

  fn draw_rooms_to_map(graph: &StableGraph<BoxExtends, (), petgraph::Undirected>, size_x: u32, size_y: u32) -> GameMap {

    let mut map = GameMap::create_empty(size_x, size_y);
    let leaves = graph.node_indices();

    // floor and walls
    for index in leaves {
      let room_box: BoxExtends = match graph.node_weight(index) {
        Some(weight) => *weight,
        None => continue
      };

      let (left, top) = (room_box.top_left.x, room_box.top_left.y);
      let (right, bottom) = (room_box.bottom_right.x, room_box.bottom_right.y);

      for x in left..=right {
        // top row
        map.set_game_tile(
          Coordinate {x: x, y: top}, 
          GameTile { root_tile: TileID { index: 2 } } );
        
        // bottom row
        map.set_game_tile(
          Coordinate {x: x, y: bottom}, 
          GameTile { root_tile: TileID { index: 2 } } );

        for y in (top + 1)..bottom {
          let floor = GameTile { root_tile: TileID { index: 0 } };
          let wall = GameTile { root_tile: TileID { index: 2 } };

          let tile;

          if x == left || x == right {
            tile = wall;
          } else {
            tile = floor;
          }

          map.set_game_tile(
            Coordinate {x: x, y: y}, 
            tile,
          );
        }
      }
    }

    let neighbor_pairs = graph.edge_indices().map(|index| graph.edge_endpoints(index).unwrap());

    for (room_a, room_b) in neighbor_pairs {
      MapBuilder::draw_path_between_rooms(
        &mut map, 
        graph.node_weight(room_a).unwrap(),
        graph.node_weight(room_b).unwrap()
      )
    }

    map
  }

  fn draw_path_between_rooms(map: &mut GameMap, box_a: &BoxExtends, box_b: &BoxExtends) {

    // case overlap in x
    let a_x_range: HashSet<i32> = HashSet::from_iter(box_a.top_left.x + 1..box_a.bottom_right.x);
    let b_x_range: HashSet<i32> = HashSet::from_iter(box_b.top_left.x + 1..box_b.bottom_right.x);
    let x_range_overlap: HashSet<i32> = a_x_range.intersection(&b_x_range).map(|i| *i).collect();

    if x_range_overlap.len() > 0 {

      let corridor_x = *x_range_overlap.iter().next().unwrap();
      let corridor_start = Coordinate {x: corridor_x, y: box_a.position().y};
      let corridor_end = Coordinate {x: corridor_x, y: box_b.position().y};

      MapBuilder::draw_corridor(corridor_start, corridor_end, map);

      return;
    }

    // case overlap in y
    let a_y_range: HashSet<i32> = HashSet::from_iter(box_a.top_left.y + 1..box_a.bottom_right.y);
    let b_y_range: HashSet<i32> = HashSet::from_iter(box_b.top_left.y + 1..box_b.bottom_right.y);
    let y_range_overlap: HashSet<i32> = a_y_range.intersection(&b_y_range).map(|i| *i).collect();

    if y_range_overlap.len() > 0 {
      let corridor_y = *y_range_overlap.iter().next().unwrap();
      let corridor_start = Coordinate {x: box_a.position().x, y: corridor_y};
      let corridor_end = Coordinate {x: box_b.position().x, y: corridor_y};

      MapBuilder::draw_corridor(corridor_start, corridor_end, map);

      return;
    }
    
    return;
  }

  fn draw_corridor(start: Coordinate, end: Coordinate, map: &mut GameMap) {

    if start.x == end.x {
      let vertical = |y| Coordinate{x: start.x, y: y};
      let left_of = |coord: Coordinate| Coordinate { x: coord.x - 1, ..coord };
      let right_of = |coord: Coordinate| Coordinate { x: coord.x + 1, ..coord };

      let (low_y, high_y) = if start.y < end.y {(start.y, end.y)} else {(end.y, start.y)};

      for y in low_y..=high_y {
        match map.get_game_tile(vertical(y)) {
          Some(GameTile { root_tile: tile::WALL_TILE_ID }) => {
            map.set_game_tile(vertical(y), GameTile { root_tile: tile::FLOOR_TILE_ID });
          }
          Some(_) => {}
          None => {
            map.set_game_tile(vertical(y), GameTile { root_tile: tile::FLOOR_TILE_ID });
            map.set_game_tile(left_of(vertical(y)), GameTile { root_tile: tile::WALL_TILE_ID });
            map.set_game_tile(right_of(vertical(y)), GameTile { root_tile: tile::WALL_TILE_ID });
          } 
        }
      }


    } else if start.y == end.y {
      let horizontal = |x| Coordinate{x: x, y: start.y};
      let above = |coord: Coordinate| Coordinate { y: coord.y - 1, ..coord };
      let below = |coord: Coordinate| Coordinate { y: coord.y + 1, ..coord };

      let (low_x, high_x) = if start.x < end.x {(start.x, end.x)} else {(end.x, start.x)};

      for x in low_x..=high_x {
        match map.get_game_tile(horizontal(x)) {
          Some(GameTile { root_tile: tile::WALL_TILE_ID }) => {
            map.set_game_tile(horizontal(x), GameTile { root_tile: tile::FLOOR_TILE_ID });
          }
          Some(_) => {}
          None => {
            map.set_game_tile(horizontal(x), GameTile { root_tile: tile::FLOOR_TILE_ID });
            map.set_game_tile(above(horizontal(x)), GameTile { root_tile: tile::WALL_TILE_ID });
            map.set_game_tile(below(horizontal(x)), GameTile { root_tile: tile::WALL_TILE_ID });
          } 
        }
      }
    }
    
  }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct BoxExtends {
  top_left: Coordinate,
  bottom_right: Coordinate,
}

impl BoxExtends {

  pub fn contains(&self, other: &BoxExtends) -> bool {

    self.contains_point(other.top_left) && 
    self.contains_point(other.bottom_right)
  }

  pub fn overlaps(&self, other: &BoxExtends) -> bool {

    let self_top_right = Coordinate{ x: self.bottom_right.x, y: self.top_left.y };
    let self_bottom_left = Coordinate{ x: self.top_left.x, y: self.bottom_right.y };
    let other_top_right = Coordinate{ x: other.bottom_right.x, y: other.top_left.y };
    let other_bottom_left = Coordinate{ x: other.top_left.x, y: other.bottom_right.y };

    let self_overlaps_other = 
      self.contains_point(other.top_left) || 
      self.contains_point(other.bottom_right) ||
      self.contains_point(other_top_right) ||
      self.contains_point(other_bottom_left);

    let other_overlaps_self = 
      other.contains_point(self.top_left) || 
      other.contains_point(self.bottom_right) ||
      other.contains_point(self_top_right) ||
      other.contains_point(self_bottom_left);

    self_overlaps_other || other_overlaps_self
  }

  fn contains_point(&self, point: Coordinate) -> bool {

    self.top_left.x <= point.x && point.x <= self.bottom_right.x &&
    self.top_left.y <= point.y && point.y <= self.bottom_right.y
  }

  fn get_axis_size(&self, axis: Axis) -> i32 {

    match axis {
      Axis::Horizontal => self.bottom_right.x - self.top_left.x + 1,
      Axis::Vertical => self.bottom_right.y - self.top_left.y + 1,
    }
  }

  fn random_subbox(area: &BoxExtends, min_side_length: i32) -> BoxExtends {

    // set up distribution squashed into correct range
    let mut rng = thread_rng()
      .sample_iter::<f32, _>(StandardNormal)
      .map(|val| val + 0.1)
      .map(|val| val.clamp(0.0, 0.5));
    
    let x_range = area.bottom_right.x - area.position().x;
    let y_range = area.bottom_right.y - area.position().y;

    let top_left = Coordinate {
      x: area.top_left.x + (rng.next().unwrap() * x_range as f32) as i32,
      y: area.top_left.y + (rng.next().unwrap() * y_range as f32) as i32,
    };

    let mut bottom_right = Coordinate {
      x: area.bottom_right.x - (rng.next().unwrap() * x_range as f32) as i32,
      y: area.bottom_right.y + (rng.next().unwrap() * y_range as f32) as i32,
    };

    // request minimum size
    bottom_right.x = max(bottom_right.x, min_side_length + top_left.x - 1);
    bottom_right.y = max(bottom_right.y, min_side_length + top_left.y - 1);

    // enforce maximum size
    bottom_right.x = min(bottom_right.x, area.bottom_right.x );
    bottom_right.y = min(bottom_right.y, area.bottom_right.y );

    BoxExtends { top_left: top_left, bottom_right: bottom_right }

  }

  fn split_box(area: &BoxExtends) -> (BoxExtends, BoxExtends) {

    let min_margin = 0.35;

    let mut rng = thread_rng().sample_iter::<f32, _>(StandardNormal) 
      .map(|val| val + 0.5)
      .map(|val| val.clamp(min_margin, 1.0 - min_margin));

    let horizontal_size = area.get_axis_size(Axis::Horizontal);
    let vertical_size = area.get_axis_size(Axis::Vertical);
    let side_ratio = horizontal_size as f32 / vertical_size as f32;
    
    let (left, top) = (area.top_left.x, area.top_left.y);
    let (right, bottom) = (area.bottom_right.x, area.bottom_right.y);

    let split_axis = if rng.next().unwrap() * side_ratio > 0.5 { Axis::Horizontal } else { Axis::Vertical} ;

    match split_axis {
      Axis::Vertical => {
        let split_point = (rng.next().unwrap() * vertical_size as f32) as i32 + top;

        let upper = BoxExtends {
          top_left: area.top_left,
          bottom_right: Coordinate { x: right, y: split_point },
        };

        let lower = BoxExtends {
          top_left: Coordinate { x: left, y: split_point },
          bottom_right: area.bottom_right,
        };

        (upper, lower)
      }
      Axis::Horizontal => {
        let split_point = (rng.next().unwrap() * horizontal_size as f32) as i32 + left;
        
        let left = BoxExtends {
          top_left: area.top_left,
          bottom_right: Coordinate { x: split_point, y: bottom },
        };

        let right = BoxExtends {
          top_left: Coordinate { x: split_point, y: top },
          bottom_right: area.bottom_right,
        };

        (left, right)
      }
    }
  }

  pub fn make_edge_vicinity_boxes(area: &BoxExtends, scan_distance: i32) -> Vec<BoxExtends> {

    let above = BoxExtends {
      top_left: Coordinate {
        x: area.top_left.x + 1, 
        y: if scan_distance <= area.top_left.y {area.top_left.y - scan_distance} else {0}
      },
      bottom_right: Coordinate { x: area.bottom_right.x - 1, y: area.top_left.y}
    };

    let below = BoxExtends {
      top_left: Coordinate {x: area.top_left.x + 1, y: area.bottom_right.y},
      bottom_right: Coordinate { x: area.bottom_right.x - 1, y: area.bottom_right.y + scan_distance}
    };

    let left = BoxExtends {
      top_left: Coordinate {
        x: if scan_distance <= area.top_left.x {area.top_left.x - scan_distance} else {0},
        y: area.top_left.y + 1},
      bottom_right: Coordinate { x: area.top_left.x, y: area.bottom_right.y - 1}
    };
    
    let right = BoxExtends {
      top_left: Coordinate {x: area.bottom_right.x , y: area.top_left.y + 1},
      bottom_right: Coordinate { x: area.bottom_right.x + scan_distance, y: area.bottom_right.y - 1}
    };

    vec![above, below, left, right]
  }
}

impl Euclidian for BoxExtends {

  fn distance_to<T>(&self, other: T) -> f32
    where 
      T: Euclidian {

      self.position().distance(other.position())
  }

  fn position(&self) -> Coordinate {

    let delta_x = self.bottom_right.x - self.top_left.x;
    let delta_y = self.bottom_right.y - self.top_left.y;

    Coordinate {
      x: delta_x/2 + self.top_left.x,
      y: delta_y/2 + self.top_left.y,
    }
  }
}