use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::{IntoNodeReferences, Visitable};
use petgraph::{algo, Undirected};
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::Cloned;

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
        let mut graph: Graph<Room, (), Undirected>;
        loop {
            graph = MapBuilder::binary_space_partitioning(size_x, size_y, 4);
            graph = MapBuilder::make_rooms_from_bsp(&graph);
            graph = MapBuilder::prune_small_rooms(&graph, 5);
            graph = MapBuilder::make_connected_graph(&graph, 3);
            graph = MapBuilder::prune_edges(&graph, 4);

            let islands = algo::connected_components(&graph);
            if islands == 1 {
                break;
            }
        }

        let map = MapBuilder::draw_rooms_to_map(&graph, size_x, size_y, depth);
        let map = MapBuilder::flood_fill_spawn_tables(&map, 8, 25);
        let map = MapBuilder::add_doors_to_rooms(&map);
        map
    }

    fn binary_space_partitioning(size_x: usize, size_y: usize, max_depth: usize) -> RoomGraph {
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

    fn make_rooms_from_bsp(bsp_tree: &RoomGraph) -> RoomGraph {
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

    fn leaves_from_bsp<'a>(graph: &'a RoomGraph) -> impl Iterator<Item = NodeIndex> + 'a {
        graph
            .node_indices()
            .filter(|index| graph.neighbors_undirected(*index).count() == 1)
    }

    fn make_connected_graph(room_graph: &RoomGraph, max_scan_distance: i32) -> RoomGraph {
        // Takes a graph of nodes, removes original edges and supplies edges between geographic neighbors.

        let mut new_graph = RoomGraph::default();
        new_graph.clone_from(room_graph);
        new_graph.clear_edges();

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
                BoxExtends::make_edge_vicinity_boxes(&current_area.extends, max_scan_distance, 2);

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

    fn prune_small_rooms(graph: &RoomGraph, threshold: i32) -> RoomGraph {
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

    fn prune_edges(graph: &RoomGraph, edge_threshold: usize) -> RoomGraph {
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

    fn draw_rooms_to_map(graph: &RoomGraph, size_x: usize, size_y: usize, depth: usize) -> GameMap {
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
        } else {
            // println!("Corridor not drawn");
            // println!("From Location: {}, {}", box_a.top_left.x, box_a.top_left.y);
            // println!("From dimensions: {}, {}", box_a.bottom_right.x - box_a.top_left.x, box_a.bottom_right.y - box_a.top_left.y);
            // println!("To Location: {}, {}", box_b.top_left.x, box_b.top_left.y);
            // println!("To dimensions: {}, {}", box_b.bottom_right.x - box_b.top_left.x, box_b.bottom_right.y - box_b.top_left.y);
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
                spawn_table = get_spawn_table(SMALL_ROOMS, map.depth);
            } else if new_graph[index].extends.get_inner_area() >= upper_size_threshold {
                spawn_table = get_spawn_table(HUGE_ROOMS, map.depth);
            } else {
                spawn_table = get_spawn_table(GENERIC_ROOMS, map.depth);
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

fn get_spawn_table<const W: usize, const H: usize>(
    templates: [RoomTemplate<W>; H],
    depth: usize,
) -> HashMap<&'static str, (usize, usize)> {
    let mut spawn_table: HashMap<&'static str, (usize, usize)> = HashMap::new();
    let eligible_tables: Vec<RoomTemplate<W>> = templates
        .iter()
        .filter_map(|template| {
            if template.depth_requirement <= depth {
                Some(*template)
            } else {
                None
            }
        })
        .collect();
    let template_id = thread_rng().gen_range(0..eligible_tables.len());
    for SpawnEntry(name, range) in &eligible_tables[template_id] {
        spawn_table.insert(name, range);
    }
    spawn_table
}

#[derive(Debug, Clone, Copy, Default)]
struct SpawnEntry(&'static str, (usize, usize));

#[derive(Debug, Clone, Copy)]
struct RoomTemplate<const C: usize> {
    spawn_entries: [SpawnEntry; C],
    depth_requirement: usize,
}

impl<const C: usize> RoomTemplate<C> {
    const fn new(spawn_entries: [SpawnEntry; C], depth_requirement: usize) -> Self {
        Self {
            spawn_entries,
            depth_requirement,
        }
    }

    fn from_smaller<const D: usize>(other: &RoomTemplate<D>) -> Self {
        assert!(C > D);
        let mut new_spawn_entries = [SpawnEntry::default(); C];
        for (empty, old) in new_spawn_entries.iter_mut().zip(other.spawn_entries.iter()) {
            *empty = *old;
        }
        Self {
            spawn_entries: new_spawn_entries,
            depth_requirement: other.depth_requirement,
        }
    }
}

impl<const C: usize> Default for RoomTemplate<C> {
    fn default() -> Self {
        RoomTemplate {
            spawn_entries: [SpawnEntry::default(); C],
            depth_requirement: usize::default(),
        }
    }
}

impl<const C: usize> IntoIterator for RoomTemplate<C> {
    type Item = SpawnEntry;
    type IntoIter = std::array::IntoIter<SpawnEntry, C>;
    fn into_iter(self) -> Self::IntoIter {
        self.spawn_entries.into_iter()
    }
}

impl<const C: usize> IntoIterator for &RoomTemplate<C> {
    type Item = SpawnEntry;
    type IntoIter = std::array::IntoIter<SpawnEntry, C>;
    fn into_iter(self) -> Self::IntoIter {
        self.spawn_entries.into_iter()
    }
}

const SMALL_ROOMS: [RoomTemplate<2>; 6] = [
    RoomTemplate::new(
        [
            // Doggo room
            SpawnEntry("Doggo", (1, 1)),
            SpawnEntry("Corpse", (0, 1)),
        ],
        1,
    ),
    RoomTemplate::new(
        [
            // Skelly room
            SpawnEntry("Pewpewpet", (1, 1)),
            SpawnEntry("", (0, 0)),
        ],
        1,
    ),
    RoomTemplate::new(
        [
            // empty room
            SpawnEntry("Corpse", (0, 1)),
            SpawnEntry("", (0, 0)),
        ],
        1,
    ),
    RoomTemplate::new(
        [
            // More skelly room
            SpawnEntry("Pewpewpet", (1, 2)),
            SpawnEntry("Corpse", (0, 1)),
        ],
        2,
    ),
    RoomTemplate::new(
        [
            // Pewpew in a small room
            SpawnEntry("Pewpew", (1, 1)),
            SpawnEntry("Chest", (0, 1)),
        ],
        4,
    ),
    RoomTemplate::new(
        [
            // Heavy room can now be in small room
            SpawnEntry("Heavy", (1, 1)),
            SpawnEntry("Corpse", (1, 3)),
        ],
        5,
    ),
];

const GENERIC_ROOMS: [RoomTemplate<4>; 6] = [
    RoomTemplate::new(
        [
            // DOGGO room
            SpawnEntry("Doggo", (1, 2)),
            SpawnEntry("Corpse", (0, 2)),
            SpawnEntry("Gold", (0, 1)),
            SpawnEntry("", (0, 0)),
        ],
        1,
    ),
    RoomTemplate::new(
        [
            // Generic skelly room
            SpawnEntry("Pewpewpet", (1, 2)),
            SpawnEntry("Corpse", (0, 1)),
            SpawnEntry("Gold", (0, 1)),
            SpawnEntry("", (0, 0)),
        ],
        1,
    ),
    RoomTemplate::new(
        [
            // Empty chest room
            SpawnEntry("Chest", (0, 1)),
            SpawnEntry("Corpse", (0, 1)),
            SpawnEntry("", (0, 0)),
            SpawnEntry("", (0, 0)),
        ],
        2,
    ),
    RoomTemplate::new(
        [
            // Single heavy room
            SpawnEntry("Heavy", (1, 1)),
            SpawnEntry("Corpse", (2, 3)),
            SpawnEntry("Gold", (0, 1)),
            SpawnEntry("", (0, 0)),
        ],
        2,
    ),
    RoomTemplate::new(
        [
            // Medium cultist room
            SpawnEntry("Pewpewpet", (1, 2)),
            SpawnEntry("Pewpew", (0, 1)),
            SpawnEntry("Chest", (0, 1)),
            SpawnEntry("Gold", (0, 1)),
        ],
        3,
    ),
    RoomTemplate::new(
        [
            // Heavy room
            SpawnEntry("Heavy", (1, 1)),
            SpawnEntry("Corpse", (2, 3)),
            SpawnEntry("Gold", (0, 1)),
            SpawnEntry("", (0, 0)),
        ],
        3,
    ),
];

const HUGE_ROOMS: [RoomTemplate<4>; 5] = [
    RoomTemplate::new(
        [
            // DOGGO room
            SpawnEntry("Doggo", (2, 3)),
            SpawnEntry("Chest", (0, 1)),
            SpawnEntry("Corpse", (1, 2)),
            SpawnEntry("", (0, 0)),
        ],
        1,
    ),
    RoomTemplate::new(
        [
            // Big cultist room
            SpawnEntry("Pewpew", (1, 1)),
            SpawnEntry("Pewpewpet", (2, 3)),
            SpawnEntry("Corpse", (1, 2)),
            SpawnEntry("Chest", (1, 2)),
        ],
        2,
    ),
    RoomTemplate::new(
        [
            // Animal heavy room
            SpawnEntry("Heavy", (1, 1)),
            SpawnEntry("Doggo", (1, 2)),
            SpawnEntry("Corpse", (1, 3)),
            SpawnEntry("Gold", (0, 1)),
        ],
        4,
    ),
    RoomTemplate::new(
        [
            // Double heavy animal room
            SpawnEntry("Heavy", (2, 2)),
            SpawnEntry("Doggo", (0, 1)),
            SpawnEntry("Corpse", (1, 3)),
            SpawnEntry("", (0, 0)),
        ],
        6,
    ),
    RoomTemplate::new(
        [
            // Double cultist room
            SpawnEntry("Pewpew", (2, 3)),
            SpawnEntry("Pewpewpet", (1, 3)),
            SpawnEntry("Corpse", (1, 2)),
            SpawnEntry("Chest", (2, 2)),
        ],
        5,
    ),
];
