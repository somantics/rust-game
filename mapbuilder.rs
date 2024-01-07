use std::collections::HashSet;

use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::IntoNodeReferences;
use petgraph::algo;

use crate::map::Euclidian;
use crate::tile::{self, WALL_TILE_ID};
use crate::{
    boxextends::BoxExtends,
    map::{Coordinate, GameMap},
    tile::{GameTile, TileID},
};

// TODO: clean this up
/* Algorithm sketch
make tree
  split till correct level

populate leafs with rooms
  rooms are defined with box extends of coordinates

connect rooms with corridors
  use collisionboxes to detect nearby rooms

  trim down w/o breaking access to each room
    select random room w. atleast 3 edges
    trim edge
    check connectivity
    confirm trim

    repeat til ?

tier rooms from start to finish
  select start and end
  base this on distance to start

populate rooms with encounters
  put connected encounters in same or one step earlier tier (will this be necessary on small map?)
  ensure room distance is small enough

*/

pub enum Axis {
    Horizontal,
    Vertical,
}

pub struct MapBuilder {
    // this a bit is awkward, should I remove the struct?
}

impl MapBuilder {
    pub fn generate_new(size_x: u32, size_y: u32) -> GameMap {
        let bsp_tree = MapBuilder::binary_space_partitioning(size_x, size_y, 4);
        let graph = MapBuilder::make_rooms_from_bsp(&bsp_tree);
        let graph = MapBuilder::prune_small_rooms(&graph, 5);
        let graph = MapBuilder::make_connected_graph(&graph, 3);
        let graph = MapBuilder::prune_edges(&graph, 4);

        MapBuilder::draw_rooms_to_map(&graph, size_x, size_y)
    }

    fn binary_space_partitioning(
        size_x: u32,
        size_y: u32,
        max_depth: usize,
    ) -> Graph<BoxExtends, (), petgraph::Undirected> {
        // Recursive algorithm for generating a binary space partitioning on BoxExtends. 
        // Allows overlapping walls.
        let mut graph = Graph::<BoxExtends, (), petgraph::Undirected>::new_undirected();
        let map_box = BoxExtends {
            top_left: Coordinate::default(),
            bottom_right: Coordinate {
                x: (size_x - 1) as i32,
                y: (size_y - 1) as i32,
            },
        };

        let origin = graph.add_node(map_box);
        MapBuilder::split_branch(origin, &mut graph, 0, max_depth);

        graph
    }

    fn split_branch(
        parent: NodeIndex,
        graph: &mut Graph<BoxExtends, (), petgraph::Undirected>,
        current_depth: usize,
        max_depth: usize,
    ) {
        // Inner recursive function, adds nodes to 'graph' directly.
        if current_depth >= max_depth {
            return;
        }

        let parent_box = graph.node_weight(parent).unwrap();
        let (a, b) = BoxExtends::split_box(parent_box);
        let branch_a = graph.add_node(a);
        let branch_b = graph.add_node(b);

        graph.extend_with_edges(&[(parent, branch_a), (parent, branch_b)]);

        MapBuilder::split_branch(branch_a, graph, current_depth + 1, max_depth);

        MapBuilder::split_branch(branch_b, graph, current_depth + 1, max_depth);
    }

    fn make_rooms_from_bsp(
        bsp_tree: &Graph<BoxExtends, (), petgraph::Undirected>,
    ) -> Graph<BoxExtends, (), petgraph::Undirected> {
        // Generates rooms inside the partitioned areas. Returned as a new graph.
        let bsp_leaves = MapBuilder::leaves_from_bsp(&bsp_tree);
        let mut graph = Graph::<BoxExtends, (), petgraph::Undirected>::default();

        for index in bsp_leaves {
            let room_box = match bsp_tree.node_weight(index) {
                Some(extends) => BoxExtends::random_subbox(extends, 0.3, 3),
                None => continue,
            };

            graph.add_node(room_box);
        }

        graph
    }

    fn leaves_from_bsp<'a>(
        graph: &'a Graph<BoxExtends, (), petgraph::Undirected>,
    ) -> impl Iterator<Item = NodeIndex> + 'a {
        graph
            .node_indices()
            .filter(|index| graph.neighbors_undirected(*index).count() == 1)
    }

    fn make_connected_graph(
        room_graph: &Graph<BoxExtends, (), petgraph::Undirected>,
        max_scan_distance: i32,
    ) -> Graph<BoxExtends, (), petgraph::Undirected> {
        // Takes a graph of nodes w. no edges and supplies edges between geographic neighbors.

        let mut new_graph = Graph::<BoxExtends, (), petgraph::Undirected>::default();
        new_graph.clone_from(room_graph);

        let mut unprocessed = room_graph.node_references(); // this moves room_graph
        let mut opened: Vec<(NodeIndex, &BoxExtends)> = vec![];
        let mut closed: Vec<NodeIndex> = vec![];

        let mut current_node: NodeIndex;
        let mut current_area: &BoxExtends;

        loop {
            // Select next node to process
            if opened.len() == 0 {
                // if none in open list, get from  unprocessed
                (current_node, current_area) = match unprocessed.next() {
                    Some(tuple) => tuple,
                    None => break,
                };
            } else {
                // take from opened list
                (current_node, current_area) = match opened.pop() {
                    Some(tuple) => tuple,
                    None => break,
                };
            }
            closed.push(current_node);

            // find neighbors using collision boxes to the top, bottom, right, left
            let collision_boxes: Vec<BoxExtends> =
                BoxExtends::make_edge_vicinity_boxes(&current_area, max_scan_distance);

            let neighbors = unprocessed
                .clone()
                .filter(|(_, area)| {
                    collision_boxes
                        .iter()
                        .any(|collision| area.overlaps(collision))
                })
                .filter(|(index, _)| !closed.contains(index));
            
            // add hits to opened list
            opened.extend(neighbors.clone());
            
            // make new edges
            new_graph.extend_with_edges(neighbors.map(|(index, _)| (current_node, index)));
        }

        new_graph
    }

    fn prune_small_rooms(
        graph: &Graph<BoxExtends, (), petgraph::Undirected>,
        threshold: i32,
    ) -> Graph<BoxExtends, (), petgraph::Undirected> {
        // Rebuilds graph without rooms w. floor area less than the threshold.
        let mut pruned_graph = Graph::<BoxExtends, (), petgraph::Undirected>::default();
        let filtered_rooms = graph
            .node_indices()
            .map(|index| graph.node_weight(index).unwrap())
            .filter(|area| area.get_inner_area() > threshold);

        for weight in filtered_rooms {
            pruned_graph.add_node(*weight);
        }

        pruned_graph
    }

    fn prune_edges(
        graph: &Graph<BoxExtends, (), petgraph::Undirected>,
        edge_threshold: usize,
    ) -> Graph<BoxExtends, (), petgraph::Undirected> {
        // Attempts to prune edges from rooms with edge_count over the threshold.
        // Tries to maintain connectivity throughout the graph.
        let mut pruned_graph = Graph::<BoxExtends, (), petgraph::Undirected>::default();
        pruned_graph.clone_from(graph);

        for room in pruned_graph.node_indices() {
            let neighbor_count = pruned_graph.neighbors(room).count();
            if !(neighbor_count >= edge_threshold) {
                continue;
            }

            let best_connected_neighbor = pruned_graph
                .neighbors(room)
                .reduce(|max, new| {
                    if graph.neighbors(max).count() >= graph.edges(new).count() {
                        max
                    } else {
                        new
                    }
                })
                .unwrap();

            let edge_candidate = pruned_graph
                .find_edge(room, best_connected_neighbor)
                .unwrap();
            
            pruned_graph.remove_edge(edge_candidate);
            
            // Do not prune if connectivity is compromised.
            if algo::connected_components(&pruned_graph) != 1 {
                pruned_graph.add_edge(room, best_connected_neighbor, ());
            }
        }

        pruned_graph
    }

    fn draw_rooms_to_map(
        graph: &Graph<BoxExtends, (), petgraph::Undirected>,
        size_x: u32,
        size_y: u32,
    ) -> GameMap {
        let mut map = GameMap::create_empty(size_x, size_y);
        let leaves = graph.node_indices();

        // Drawing empty rooms
        for index in leaves {
            let room_box: BoxExtends = match graph.node_weight(index) {
                Some(weight) => *weight,
                None => continue,
            };

            MapBuilder::draw_room(room_box, &mut map, size_x, size_y);
        }

        // Drawing corridors
        let neighbor_pairs = graph
            .edge_indices()
            .map(|index| graph.edge_endpoints(index).unwrap());

        for (room_a, room_b) in neighbor_pairs {
            MapBuilder::draw_path_between_rooms(
                &mut map,
                graph.node_weight(room_a).unwrap(),
                graph.node_weight(room_b).unwrap(),
            )
        }

        map
    }

    fn draw_room(
        room_box: BoxExtends,
        map: &mut GameMap,
        size_x: u32,
        size_y: u32,
    ) {
        let (left, top) = (room_box.top_left.x, room_box.top_left.y);
        let (right, bottom) = (room_box.bottom_right.x, room_box.bottom_right.y);

        for x in left..=right {
            // top row
            map.set_game_tile(
                Coordinate { x: x, y: top },
                GameTile {
                    root_tile: tile::FLOOR_TILE_ID,
                },
            );

            // bottom row
            map.set_game_tile(
                Coordinate { x: x, y: bottom },
                GameTile {
                    root_tile: tile::WALL_TILE_ID,
                },
            );

            for y in (top + 1)..bottom {
                let floor = GameTile {
                    root_tile: tile::FLOOR_TILE_ID,
                };
                let wall = GameTile {
                    root_tile: WALL_TILE_ID,
                };

                let tile;

                if x == left || x == right {
                    tile = wall;
                } else {
                    tile = floor;
                }

                map.set_game_tile(Coordinate { x: x, y: y }, tile);
            }
        }
    }

    fn draw_path_between_rooms(map: &mut GameMap, box_a: &BoxExtends, box_b: &BoxExtends) {
        // case overlap in x
        let a_x_range: HashSet<i32> =
            HashSet::from_iter(box_a.top_left.x + 1..box_a.bottom_right.x);
        let b_x_range: HashSet<i32> =
            HashSet::from_iter(box_b.top_left.x + 1..box_b.bottom_right.x);
        let x_range_overlap: HashSet<i32> =
            a_x_range.intersection(&b_x_range).map(|i| *i).collect();

        if x_range_overlap.len() > 0 {
            let corridor_x = *x_range_overlap.iter().next().unwrap();
            let corridor_start = Coordinate {
                x: corridor_x,
                y: box_a.position().y,
            };
            let corridor_end = Coordinate {
                x: corridor_x,
                y: box_b.position().y,
            };

            MapBuilder::draw_vertical_corridor(corridor_start, corridor_end, map);

            return;
        }

        // case overlap in y
        let a_y_range: HashSet<i32> =
            HashSet::from_iter(box_a.top_left.y + 1..box_a.bottom_right.y);
        let b_y_range: HashSet<i32> =
            HashSet::from_iter(box_b.top_left.y + 1..box_b.bottom_right.y);
        let y_range_overlap: HashSet<i32> =
            a_y_range.intersection(&b_y_range).map(|i| *i).collect();

        if y_range_overlap.len() > 0 {
            let corridor_y = *y_range_overlap.iter().next().unwrap();
            let corridor_start = Coordinate {
                x: box_a.position().x,
                y: corridor_y,
            };
            let corridor_end = Coordinate {
                x: box_b.position().x,
                y: corridor_y,
            };

            MapBuilder::draw_horizontal_corridor(corridor_start, corridor_end, map);
        }
    }

    fn draw_vertical_corridor(start: Coordinate, end: Coordinate, map: &mut GameMap) {
        let vertical = |y| Coordinate { x: start.x, y: y };
        let left_of = |coord: Coordinate| Coordinate {
            x: coord.x - 1,
            ..coord
        };
        let right_of = |coord: Coordinate| Coordinate {
            x: coord.x + 1,
            ..coord
        };

        let (low_y, high_y) = if start.y < end.y {
            (start.y, end.y)
        } else {
            (end.y, start.y)
        };

        for y in low_y..=high_y {
            match map.get_game_tile(vertical(y)) {
                Some(GameTile {
                    root_tile: tile::WALL_TILE_ID,
                }) => {
                    map.set_game_tile(
                        vertical(y),
                        GameTile {
                            root_tile: tile::FLOOR_TILE_ID,
                        },
                    );
                }
                Some(_) => {}
                None => {
                    map.set_game_tile(
                        vertical(y),
                        GameTile {
                            root_tile: tile::FLOOR_TILE_ID,
                        },
                    );
                    map.set_game_tile(
                        left_of(vertical(y)),
                        GameTile {
                            root_tile: tile::WALL_TILE_ID,
                        },
                    );
                    map.set_game_tile(
                        right_of(vertical(y)),
                        GameTile {
                            root_tile: tile::WALL_TILE_ID,
                        },
                    );
                }
            }
        }
    }

    fn draw_horizontal_corridor(start: Coordinate, end: Coordinate, map: &mut GameMap) {
        let center = |x| Coordinate { x: x, y: start.y };
        let above = |coord: Coordinate| Coordinate {
            y: coord.y - 1,
            ..coord
        };
        let below = |coord: Coordinate| Coordinate {
            y: coord.y + 1,
            ..coord
        };

        let (low_x, high_x) = if start.x < end.x {
            (start.x, end.x)
        } else {
            (end.x, start.x)
        };

        for x in low_x..=high_x {
            match map.get_game_tile(center(x)) {
                Some(GameTile {
                    root_tile: tile::WALL_TILE_ID,
                }) => {
                    map.set_game_tile(
                        center(x),
                        GameTile {
                            root_tile: tile::FLOOR_TILE_ID,
                        },
                    );
                }
                Some(_) => {}
                None => {
                    map.set_game_tile(
                        center(x),
                        GameTile {
                            root_tile: tile::FLOOR_TILE_ID,
                        },
                    );
                    map.set_game_tile(
                        above(center(x)),
                        GameTile {
                            root_tile: tile::WALL_TILE_ID,
                        },
                    );
                    map.set_game_tile(
                        below(center(x)),
                        GameTile {
                            root_tile: tile::WALL_TILE_ID,
                        },
                    );
                }
            }
        }
    }
}
