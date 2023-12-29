

use crate::map::{Coordinate};

struct BoxExtends {
  top_left: Coordinate,
  bottom_right: Coordinate,
}

impl Euclidian for BoxExtends {
  fn distance_to<T>(&self, other: T) -> f32
    where 
      T: Euclidian {
      self.position.distance(other.position())
  }

  fn position(&self) -> Coordinate {
    let delta_x = bottom_right.x - top_left.x;
    let delta_y = bottom_right.y - top_left.y;

    Coordinate {
      x: delta_x/2 + top_left.x,
      y: delta_y/2 + top_left.y,
    }
  }
}
/* 
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