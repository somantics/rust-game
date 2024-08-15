use rand::prelude::*;
use rand_distr::StandardNormal;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};

use super::mapbuilder::Axis;
use crate::ecs::spawning::OBJECT_SPAWN_NAMES;
use crate::ecs::{spawning, ECS};
use crate::map::{Coordinate, Euclidian};

// Tracks areas on the grid and supports overlapping and orthogonal adjacency checks.
// is also responsible for dividing space when an area is split into two.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BoxExtends {
    pub top_left: Coordinate,
    pub bottom_right: Coordinate,
}

impl BoxExtends {
    pub fn contains(&self, other: &BoxExtends) -> bool {
        self.contains_point(other.top_left) && self.contains_point(other.bottom_right)
    }

    pub fn overlaps(&self, other: &BoxExtends) -> bool {
        let self_top_right = Coordinate {
            x: self.bottom_right.x,
            y: self.top_left.y,
        };
        let self_bottom_left = Coordinate {
            x: self.top_left.x,
            y: self.bottom_right.y,
        };
        let other_top_right = Coordinate {
            x: other.bottom_right.x,
            y: other.top_left.y,
        };
        let other_bottom_left = Coordinate {
            x: other.top_left.x,
            y: other.bottom_right.y,
        };

        let self_overlaps_other = self.contains_point(other.top_left)
            || self.contains_point(other.bottom_right)
            || self.contains_point(other_top_right)
            || self.contains_point(other_bottom_left);

        let other_overlaps_self = other.contains_point(self.top_left)
            || other.contains_point(self.bottom_right)
            || other.contains_point(self_top_right)
            || other.contains_point(self_bottom_left);

        self_overlaps_other || other_overlaps_self
    }

    pub fn contains_point(&self, point: Coordinate) -> bool {
        self.top_left.x <= point.x
            && point.x <= self.bottom_right.x
            && self.top_left.y <= point.y
            && point.y <= self.bottom_right.y
    }

    pub fn get_axis_size(&self, axis: Axis) -> i32 {
        // Axis size counts whole length of the side, i.e. x:3 to x:5 is len 3.
        match axis {
            Axis::Horizontal => self.bottom_right.x - self.top_left.x + 1,
            Axis::Vertical => self.bottom_right.y - self.top_left.y + 1,
        }
    }

    pub fn get_area(&self) -> i32 {
        let delta_x = self.bottom_right.x - self.top_left.x + 1;
        let delta_y = self.bottom_right.y - self.top_left.y + 1;

        delta_x * delta_y
    }

    pub fn get_inner_area(&self) -> i32 {
        // inner area ignores walls in area calculation
        let inner_delta_x = self.bottom_right.x - self.top_left.x - 1;
        let inner_delta_y = self.bottom_right.y - self.top_left.y - 1;

        if inner_delta_x <= 0 || inner_delta_y <= 0 {
            return 0;
        };

        inner_delta_x * inner_delta_y
    }

    pub fn random_subbox(area: &BoxExtends, shrink_term: f32, min_side_length: i32) -> BoxExtends {
        // Create a new box randomly shrinked from the provided one.
        // Used in randomizing size of rooms in a partition from the bsp.

        // Normal distribution yields weight towards small shrinking.
        // Clamping prevents extreme values but creates probability artefacts.
        let mut rng = thread_rng()
            .sample_iter::<f32, _>(StandardNormal)
            .map(|val| val + shrink_term)
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
        bottom_right.x = min(bottom_right.x, area.bottom_right.x);
        bottom_right.y = min(bottom_right.y, area.bottom_right.y);

        BoxExtends {
            top_left: top_left,
            bottom_right: bottom_right,
        }
    }

    pub fn split_box(area: &BoxExtends) -> (BoxExtends, BoxExtends) {
        let min_margin = 0.35;

        // Squashes values in the middle, but clamping yields probability artefacts.
        let mut rng = thread_rng()
            .sample_iter::<f32, _>(StandardNormal)
            .map(|val| val + 0.5)
            .map(|val| val.clamp(min_margin, 1.0 - min_margin));

        // Scale side selection probability with side len ratio.
        let horizontal_size = area.get_axis_size(Axis::Horizontal);
        let vertical_size = area.get_axis_size(Axis::Vertical);
        let side_ratio = horizontal_size as f32 / vertical_size as f32;

        let (left, top) = (area.top_left.x, area.top_left.y);
        let (right, bottom) = (area.bottom_right.x, area.bottom_right.y);

        let split_axis = if rng.next().unwrap() * side_ratio > 0.5 {
            Axis::Horizontal
        } else {
            Axis::Vertical
        };

        match split_axis {
            Axis::Vertical => {
                let split_point = (rng.next().unwrap() * vertical_size as f32) as i32 + top;

                let upper = BoxExtends {
                    top_left: area.top_left,
                    bottom_right: Coordinate {
                        x: right,
                        y: split_point,
                    },
                };

                let lower = BoxExtends {
                    top_left: Coordinate {
                        x: left,
                        y: split_point,
                    },
                    bottom_right: area.bottom_right,
                };

                (upper, lower)
            }
            Axis::Horizontal => {
                let split_point = (rng.next().unwrap() * horizontal_size as f32) as i32 + left;

                let left = BoxExtends {
                    top_left: area.top_left,
                    bottom_right: Coordinate {
                        x: split_point,
                        y: bottom,
                    },
                };

                let right = BoxExtends {
                    top_left: Coordinate {
                        x: split_point,
                        y: top,
                    },
                    bottom_right: area.bottom_right,
                };

                (left, right)
            }
        }
    }

    pub fn make_edge_vicinity_boxes(area: &BoxExtends, scan_distance: i32, overlap: i32) -> Vec<BoxExtends> {
        // Creates hitboxes that check the sides of 'area' up to 'scan_distance'
        let above = BoxExtends {
            top_left: Coordinate {
                x: area.top_left.x + overlap,
                y: if scan_distance <= area.top_left.y {
                    area.top_left.y - scan_distance
                } else {
                    0
                },
            },
            bottom_right: Coordinate {
                x: area.bottom_right.x - overlap,
                y: area.top_left.y,
            },
        };

        let below = BoxExtends {
            top_left: Coordinate {
                x: area.top_left.x + overlap,
                y: area.bottom_right.y,
            },
            bottom_right: Coordinate {
                x: area.bottom_right.x - overlap,
                y: area.bottom_right.y + scan_distance,
            },
        };

        let left = BoxExtends {
            top_left: Coordinate {
                x: if scan_distance <= area.top_left.x {
                    area.top_left.x - scan_distance
                } else {
                    0
                },
                y: area.top_left.y + overlap,
            },
            bottom_right: Coordinate {
                x: area.top_left.x,
                y: area.bottom_right.y - overlap,
            },
        };

        let right = BoxExtends {
            top_left: Coordinate {
                x: area.bottom_right.x,
                y: area.top_left.y + overlap,
            },
            bottom_right: Coordinate {
                x: area.bottom_right.x + scan_distance,
                y: area.bottom_right.y - overlap,
            },
        };

        vec![above, below, left, right]
    }
}

impl Euclidian for BoxExtends {
    fn distance_to<T>(&self, other: T) -> f32
    where
        T: Euclidian,
    {
        self.position().distance(other.position())
    }

    fn position(&self) -> Coordinate {
        let delta_x = self.bottom_right.x - self.top_left.x;
        let delta_y = self.bottom_right.y - self.top_left.y;

        Coordinate {
            x: delta_x / 2 + self.top_left.x,
            y: delta_y / 2 + self.top_left.y,
        }
    }
}

type Range = (usize, usize);

#[derive(Debug, Default, Clone)]
pub struct Room {
    pub extends: BoxExtends,
    pub spawn_table: Option<HashMap<&'static str, Range>>,
    pub door_locations: Vec<Coordinate>,
}

impl Room {
    pub fn new(extends: BoxExtends) -> Self {
        Room {
            extends,
            spawn_table: None,
            door_locations: vec![],
        }
    }

    fn spawn_doors(&self, ecs: &mut ECS, depth: usize) {
        for coord in &self.door_locations {
            if ecs.is_blocked_by_entity(*coord) {
                continue;
            }
            spawning::make_door(ecs, *coord, depth);
        }
    }

    fn adjacent_to_door(&self, coord: Coordinate) -> bool {
        self.door_locations
            .iter()
            .find(|door| coord.distance(**door) <= 1.0)
            .is_some()
    }

    fn get_free_coordinate(
        &self,
        occupied: &HashSet<Coordinate>,
        rng: &mut ThreadRng,
        x_min: i32,
        x_max: i32,
        y_min: i32,
        y_max: i32,
    ) -> Coordinate {
        let mut coord = Coordinate {
            x: rng.gen_range(x_min..=x_max),
            y: rng.gen_range(y_min..=y_max),
        };
        // Ensure location is unoccupied. TODO: doesn't terminate if area is full
        while occupied.contains(&coord) || self.adjacent_to_door(coord) {
            coord = Coordinate {
                x: rng.gen_range(x_min..=x_max),
                y: rng.gen_range(y_min..=y_max),
            };
        }
        coord
    }

    pub fn spawn_entities(&self, ecs: &mut ECS, depth: usize) {
        let mut rng = thread_rng();
        let mut occupied = HashSet::<Coordinate>::new();

        // Floor area coordinate bounds
        let x_min = self.extends.top_left.x + 1;
        let x_max = self.extends.bottom_right.x - 1;

        let y_min = self.extends.top_left.y + 1;
        let y_max = self.extends.bottom_right.y - 1;

        if let Some(table) = &self.spawn_table {
            // Process spawning table
            for (&name, &(min, max)) in table.iter() {
                if name == "Player" && ecs.has_player() {
                    let coord =
                        self.get_free_coordinate(&occupied, &mut rng, x_min, x_max, y_min, y_max);
                    ecs.set_player_position(coord);
                    continue;
                }
                // Look for matching spawn function
                if let Some(spawn_func) = OBJECT_SPAWN_NAMES.get(name) {
                    // Generate amount
                    let amount = rng.gen_range(min..=max);
                    for _ in 0..amount {
                        // Initial location to spawn
                        let coord = self
                            .get_free_coordinate(&occupied, &mut rng, x_min, x_max, y_min, y_max);
                        (spawn_func)(ecs, coord, depth);
                        occupied.insert(coord);
                    }
                }
            }
        }

        self.spawn_doors(ecs, depth);
    }
}

/*

template
{
    "monster name": 4-5
    "chest": loot table class
}


*/
