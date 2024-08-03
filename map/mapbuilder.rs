use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::algo;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::IntoNodeReferences;
use rand::{thread_rng, Rng};

use super::boxextends::{BoxExtends, Room};
use super::Euclidian;
use crate::{
    map::tile::GameTile,
    map::{Coordinate, GameMap},
};

pub type RoomGraph = Graph<Room, (), petgraph::Undirected>;

pub enum Axis {
    Horizontal,
    Vertical,
}

pub struct MapBuilder {
    // this a bit is awkward, should I remove the struct?
}

impl MapBuilder {
    pub fn generate_new(size_x: usize, size_y: usize, depth: usize) -> GameMap {
        let graph = MapBuilder::binary_space_partitioning(size_x, size_y, 4);
        let graph = MapBuilder::make_rooms_from_bsp(&graph);
        let graph = MapBuilder::prune_small_rooms(&graph, 5);
        let graph = MapBuilder::make_connected_graph(&graph, 3);
        let graph = MapBuilder::prune_edges(&graph, 4);

        let map = MapBuilder::draw_rooms_to_map(&graph, size_x, size_y, depth);
        // add quest enabled encounters
        let map = MapBuilder::flood_fill_spawn_tables(&map, 8, 25);
        let map = MapBuilder::add_doors_to_rooms(&map);

        map
    }

    fn binary_space_partitioning(
        size_x: usize,
        size_y: usize,
        max_depth: usize,
    ) -> RoomGraph {
        // Recursive algorithm for generating a binary space partitioning on BoxExtends.
        // Allows overlapping walls.
        let mut graph = RoomGraph::new_undirected();
        let map_box = BoxExtends {
            top_left: Coordinate::default(),
            bottom_right: Coordinate {
                x: (size_x - 1) as i32,
                y: (size_y - 1) as i32,
            },
        };
        let map_room = Room::new(map_box);
        let origin = graph.add_node(map_room);
        MapBuilder::split_branch(origin, &mut graph, 0, max_depth);

        graph
    }

    fn split_branch(
        parent: NodeIndex,
        graph: &mut RoomGraph,
        current_depth: usize,
        max_depth: usize,
    ) {
        // Inner recursive function, adds nodes to 'graph' directly.
        if current_depth >= max_depth {
            return;
        }

        let parent_box = graph.node_weight(parent).unwrap().extends;
        let (a, b) = BoxExtends::split_box(&parent_box);
        let branch_a = graph.add_node(Room::new(a));
        let branch_b = graph.add_node(Room::new(b));

        graph.extend_with_edges(&[(parent, branch_a), (parent, branch_b)]);

        MapBuilder::split_branch(branch_a, graph, current_depth + 1, max_depth);

        MapBuilder::split_branch(branch_b, graph, current_depth + 1, max_depth);
    }

    fn make_rooms_from_bsp(
        bsp_tree: &RoomGraph,
    ) -> RoomGraph {
        // Generates rooms inside the partitioned areas. Returned as a new graph.
        let bsp_leaves = MapBuilder::leaves_from_bsp(&bsp_tree);
        let mut graph = Graph::<Room, (), petgraph::Undirected>::default();

        for index in bsp_leaves {
            let room_box = match bsp_tree.node_weight(index) {
                Some(room) => BoxExtends::random_subbox(&room.extends, 0.3, 3),
                None => continue,
            };

            // Removes any existing room data beyond extends
            graph.add_node(Room::new(room_box));
        }

        graph
    }

    fn leaves_from_bsp<'a>(
        graph: &'a RoomGraph,
    ) -> impl Iterator<Item = NodeIndex> + 'a {
        graph
            .node_indices()
            .filter(|index| graph.neighbors_undirected(*index).count() == 1)
    }

    fn make_connected_graph(
        room_graph: &RoomGraph,
        max_scan_distance: i32,
    ) -> RoomGraph {
        // Takes a graph of nodes w. no edges and supplies edges between geographic neighbors.

        let mut new_graph = RoomGraph::default();
        new_graph.clone_from(room_graph);

        let mut unprocessed = room_graph.node_references(); // this moves room_graph
        let mut opened: Vec<(NodeIndex, &Room)> = vec![];
        let mut closed: Vec<NodeIndex> = vec![];

        let mut current_node: NodeIndex;
        let mut current_area: &Room;

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
                BoxExtends::make_edge_vicinity_boxes(&current_area.extends, max_scan_distance);

            let neighbors = unprocessed
                .clone()
                .filter(|(_, area)| {
                    collision_boxes
                        .iter()
                        .any(|collision| area.extends.overlaps(collision))
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
        graph: &RoomGraph,
        threshold: i32,
    ) -> RoomGraph {
        // Rebuilds graph without rooms w. floor area less than the threshold.
        let mut pruned_graph = RoomGraph::default();
        let filtered_rooms = graph
            .node_indices()
            .map(|index| graph.node_weight(index).unwrap())
            .filter(|room: &&Room| room.extends.get_inner_area() > threshold);

        for weight in filtered_rooms {
            pruned_graph.add_node(weight.clone());
        }

        pruned_graph
    }

    fn prune_edges(
        graph: &RoomGraph,
        edge_threshold: usize,
    ) -> RoomGraph {
        // Attempts to prune edges from rooms with edge_count over the threshold.
        // Tries to maintain connectivity throughout the graph.
        let mut pruned_graph = RoomGraph::default();
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
        graph: &RoomGraph,
        size_x: usize,
        size_y: usize,
        depth: usize,
    ) -> GameMap {
        let mut map = GameMap::create_empty(size_x, size_y);
        map.graph = graph.clone();
        map.depth = depth;
        let leaves = graph.node_indices();

        // Drawing empty rooms
        for index in leaves {
            let room_box: BoxExtends = match graph.node_weight(index) {
                Some(weight) => weight.extends,
                None => continue,
            };

            MapBuilder::draw_room(room_box, &mut map);
        }

        // Drawing corridors
        let neighbor_pairs = graph
            .edge_indices()
            .map(|index| graph.edge_endpoints(index).unwrap());

        for (room_a, room_b) in neighbor_pairs {
            MapBuilder::draw_path_between_rooms(
                &mut map,
                &graph.node_weight(room_a).unwrap().extends,
                &graph.node_weight(room_b).unwrap().extends,
            )
        }
        map
    }

    fn draw_room(room_box: BoxExtends, map: &mut GameMap) {
        let (left, top) = (room_box.top_left.x, room_box.top_left.y);
        let (right, bottom) = (room_box.bottom_right.x, room_box.bottom_right.y);

        for x in left..=right {
            // top row
            map.set_game_tile(
                Coordinate { x: x, y: top },
                GameTile {
                    root_tile: super::tile::WALL_TILE_ID,
                },
            );

            // bottom row
            map.set_game_tile(
                Coordinate { x: x, y: bottom },
                GameTile {
                    root_tile: super::tile::WALL_TILE_ID,
                },
            );

            for y in (top + 1)..bottom {
                let floor = GameTile {
                    root_tile: super::tile::FLOOR_TILE_ID,
                };
                let wall = GameTile {
                    root_tile: super::tile::WALL_TILE_ID,
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
        let center = |y| Coordinate { x: start.x, y: y };
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
            match map.get_game_tile(center(y)) {
                Some(GameTile {
                    root_tile: super::tile::WALL_TILE_ID,
                }) => {
                    map.set_game_tile(
                        center(y),
                        GameTile {
                            root_tile: super::tile::FLOOR_TILE_ID,
                        },
                    );
                }
                Some(_) => {}
                None => {
                    map.set_game_tile(
                        center(y),
                        GameTile {
                            root_tile: super::tile::FLOOR_TILE_ID,
                        },
                    );
                    map.set_game_tile(
                        left_of(center(y)),
                        GameTile {
                            root_tile: super::tile::WALL_TILE_ID,
                        },
                    );
                    map.set_game_tile(
                        right_of(center(y)),
                        GameTile {
                            root_tile: super::tile::WALL_TILE_ID,
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
                    root_tile: super::tile::WALL_TILE_ID,
                }) => {
                    map.set_game_tile(
                        center(x),
                        GameTile {
                            root_tile: super::tile::FLOOR_TILE_ID,
                        },
                    );
                }
                Some(_) => {}
                None => {
                    map.set_game_tile(
                        center(x),
                        GameTile {
                            root_tile: super::tile::FLOOR_TILE_ID,
                        },
                    );
                    map.set_game_tile(
                        above(center(x)),
                        GameTile {
                            root_tile: super::tile::WALL_TILE_ID,
                        },
                    );
                    map.set_game_tile(
                        below(center(x)),
                        GameTile {
                            root_tile: super::tile::WALL_TILE_ID,
                        },
                    );
                }
            }
        }
    }

    fn check_door_conditions(coord: Coordinate, map: &GameMap) -> bool {
        if !map.is_tile_passable(coord) {
            return false;
        }
        if !map.is_tile_passable(coord + Coordinate { x: 0, y: 1 })
            && !map.is_tile_passable(coord + Coordinate { x: 0, y: -1 })
        {
            return true;
        }
        if !map.is_tile_passable(coord + Coordinate { x: 1, y: 0 })
            && !map.is_tile_passable(coord + Coordinate { x: -1, y: 0 })
        {
            return true;
        }
        false
    }

    fn add_doors_to_rooms(map: &GameMap) -> GameMap {
        let mut new_graph: RoomGraph = Graph::default();
        new_graph.clone_from(&map.graph);

        for (node, room) in map.graph.node_references() {
            let (left, top) = (room.extends.top_left.x, room.extends.top_left.y);
            let (right, bottom) = (room.extends.bottom_right.x, room.extends.bottom_right.y);
            let mut door_locations = vec![];

            // Horizontal walls
            for x in left + 1..right {
                let top_coord = Coordinate { x, y: top };
                let bottom_coord = Coordinate { x, y: bottom };

                if MapBuilder::check_door_conditions(top_coord, map) {
                    door_locations.push(top_coord);
                }
                if MapBuilder::check_door_conditions(bottom_coord, map) {
                    door_locations.push(bottom_coord);
                }
            }
            // Vertical walls
            for y in top + 1..bottom {
                let left_coord = Coordinate { x: left, y };
                let right_coord = Coordinate { x: right, y };

                if MapBuilder::check_door_conditions(left_coord, map) {
                    door_locations.push(left_coord);
                }
                if MapBuilder::check_door_conditions(right_coord, map) {
                    door_locations.push(right_coord);
                }
            }

            let new_room = Room {
                door_locations,
                ..room.clone()
            };
            new_graph[node] = new_room;
        }
        let mut new_map = map.clone();
        new_map.graph = new_graph;
        new_map
    }

    fn flood_fill_spawn_tables(
        map: &GameMap,
        lower_size_threshold: i32,
        upper_size_threshold: i32,
    ) -> GameMap {
        let mut new_graph: RoomGraph = Graph::default();
        new_graph.clone_from(&map.graph);

        let mut top_left_corners: Vec<(NodeIndex, Coordinate)> = new_graph
            .node_references()
            .map(|(index, room)| (index, room.extends.top_left))
            .collect();
        top_left_corners.sort_unstable_by_key(|(_, coord)| *coord);
        let (start_index, _) = top_left_corners[0];
        let mut visited: HashSet<NodeIndex> = HashSet::new();
        let mut fill_queue: VecDeque<NodeIndex> = VecDeque::new();

        fill_queue.push_front(start_index);
        while let Some(index) = fill_queue.pop_back() {
            visited.insert(index);
            
            let unvisited_neighbors = new_graph
                .neighbors(index)
                .filter_map(|idx| (!visited.contains(&idx)).then(move || idx));

            for unvisited_index in unvisited_neighbors {
                fill_queue.push_front(unvisited_index);
            }

            let mut spawn_table: HashMap<&str, (usize, usize)> = HashMap::new();
            if index == start_index {
                spawn_table.insert("Player", (1, 1));
            } else if new_graph[index].extends.get_inner_area() <= lower_size_threshold {
                // SMALL ROOMS
                let room_templates = vec![
                    // Doggo scavenger rooms
                    vec![("Doggo", (1, 1)), ("Corpse", (0, 1))],
                    // Skelly room
                    vec![("Pewpewpet", (1, 2))],
                    // Empty room
                    vec![("Corpse", (0, 1))],
                ];
                let template_id = thread_rng().gen_range(0..room_templates.len());
                for (name, range) in &room_templates[template_id] {
                    spawn_table.insert(name, *range);
                }
            } else if new_graph[index].extends.get_inner_area() >= upper_size_threshold {
                // HUGE ROOMS
                let room_templates = vec![
                    // Animal heavy room
                    vec![("Heavy", (1, 1)), ("Doggo", (1, 2)), ("Corpse", (1, 3))],
                    // Big Cultist room
                    vec![
                        ("Pewpew", (1, 2)),
                        ("Pewpewpet", (2, 3)),
                        ("Corpse", (1, 2)),
                        ("Chest", (1, 2)),
                    ],
                ];
                let template_id = thread_rng().gen_range(0..room_templates.len());
                for (name, range) in &room_templates[template_id] {
                    spawn_table.insert(name, *range);
                }
            } else {
                // GENERIC TEMPLATES
                let room_templates = vec![
                    // Heavy Animal room
                    vec![("Heavy", (1, 1)), ("Corpse", (2, 3))],
                    // Doggo Animal room
                    vec![("Doggo", (1, 3)), ("Chest", (0, 1))],
                    // Medium cultist room
                    vec![
                        ("Pewpewpet", (1, 2)), 
                        ("Pewpew", (0, 1)), 
                        ("Chest", (0, 1))],
                    // Empty room
                    vec![("Chest", (1, 1)), ("Corpse", (0, 1))],
                ];
                let template_id = thread_rng().gen_range(0..room_templates.len());
                for (name, range) in &room_templates[template_id] {
                    spawn_table.insert(name, *range);
                }
            }

            if fill_queue.is_empty() {
                spawn_table.insert("StairsDown", (1, 1));
            }
            
            new_graph[index] = Room {
                spawn_table: Some(spawn_table),
                ..new_graph[index].clone()
            };
        }

        GameMap {
            graph: new_graph,
            ..map.clone()
        }
    }
}
