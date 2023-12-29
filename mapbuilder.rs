use std::thread::current;

use rand::prelude::*;
use rand_distr::StandardNormal;

use petgraph::graph::{Graph, NodeIndex};

use crate::{
  map::{Coordinate, Euclidian, GameMap}, 
  tile::{GameTile, TileID}
};


/* Algorithm sketch
make tree
  split till correct level

populate leafs with rooms
  rooms are defined with box extend of coordinates

connect rooms with corridors
  make locally fully connected
    connect within leaf
    connect pairwise one level up (checking smallest distance)
    connect two pairs one level up (checking smallest distance)
    connect four pairs one level up
    connect four pairs
    connect eight pairs (doublecheck the math on this)

    continue till reached root
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
  pub fn binary_space_partitioning(size_x: u32, size_y: u32) -> Graph<BoxExtends, (), petgraph::Undirected> {
    let mut graph: Graph<BoxExtends, (), petgraph::Undirected> = Graph::<BoxExtends,(), petgraph::Undirected>::new_undirected();
    let map_box = BoxExtends {
      top_left: Coordinate::default(),
      bottom_right: Coordinate { x: size_x - 1, y: size_y - 1}
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

  pub fn make_rooms_from_bsp(graph: &Graph<BoxExtends, (), petgraph::Undirected>, size_x: u32, size_y: u32) -> GameMap {
    let leaves = graph
      .node_indices()
      .filter(|index| {
        graph.neighbors(*index).count() == 1
      });

    let mut map = GameMap::create_empty(size_x, size_y);

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

    map
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BoxExtends {
  top_left: Coordinate,
  bottom_right: Coordinate,
}

impl BoxExtends {
  pub fn contains(&self, other: BoxExtends) -> bool {
    self.contains_point(other.top_left) && 
    self.contains_point(other.bottom_right)
  }

  pub fn overlaps(&self, other: BoxExtends) -> bool {
    self.contains_point(other.top_left) || 
    self.contains_point(other.bottom_right)
  }

  fn contains_point(&self, point: Coordinate) -> bool {
    self.top_left.x <= point.x && point.x <= self.bottom_right.x &&
    self.top_left.y <= point.y && point.y <= self.bottom_right.y
  }

  fn get_axis_size(&self, axis: Axis) -> u32 {
    match axis {
      Axis::Horizontal => self.bottom_right.x - self.top_left.x + 1,
      Axis::Vertical => self.bottom_right.y - self.top_left.y + 1,
    }
  }

  fn split_box(area: &BoxExtends) -> (BoxExtends, BoxExtends) {
    let min_margin: f32 = 0.35;

    let rand_factor: f32 = (thread_rng().sample::<f32, StandardNormal>(StandardNormal) + 0.5) // constant pushes mean to middle of 0-1
      .clamp(min_margin, 1.0-min_margin);

    let horizontal_size = area.get_axis_size(Axis::Horizontal);
    let vertical_size = area.get_axis_size(Axis::Vertical);
    
    let (left, top) = (area.top_left.x, area.top_left.y);
    let (right, bottom) = (area.bottom_right.x, area.bottom_right.y);

    if vertical_size >= horizontal_size {
      let split_point = (rand_factor * vertical_size as f32) as u32 + top;

      let upper = BoxExtends {
        top_left: area.top_left,
        bottom_right: Coordinate { x: right, y: split_point },
      };

      let lower = BoxExtends {
        top_left: Coordinate { x: left, y: split_point +1 },
        bottom_right: area.bottom_right,
      };

      (upper, lower)
    } else {
      let split_point = (rand_factor * horizontal_size as f32) as u32 + left;
      
      let left = BoxExtends {
        top_left: area.top_left,
        bottom_right: Coordinate { x: split_point, y: bottom },
      };

      let right = BoxExtends {
        top_left: Coordinate { x: split_point + 1, y: top },
        bottom_right: area.bottom_right,
      };

      (left, right)
    }
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