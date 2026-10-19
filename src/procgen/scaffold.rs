use std::collections::{HashMap, HashSet};

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

pub type RoomId = u32;

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct SystemFlags: u8 {
        const ELECTRICAL = 1 << 0;
        const THERMAL    = 1 << 1;
        const ATMOSPHERICS = 1 << 2;
        const FLUIDICS   = 1 << 3;
        const COMPUTER   = 1 << 4;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomType {
    Storage,
    Engineering,
    Reactor,
    LifeSupport,
    Hydroponics,
    ComputerCore,
    Bridge,
    /// Main airlock / docking vestibule.
    Airlock,
}

impl RoomType {
    /// Returns the system flags introduced by this room type.
    pub fn system_flags(&self) -> SystemFlags {
        match self {
            RoomType::Storage => SystemFlags::empty(),
            RoomType::Engineering => SystemFlags::ELECTRICAL,
            RoomType::Reactor => SystemFlags::ELECTRICAL | SystemFlags::THERMAL,
            RoomType::LifeSupport => SystemFlags::ATMOSPHERICS,
            RoomType::Hydroponics => SystemFlags::FLUIDICS | SystemFlags::ATMOSPHERICS,
            RoomType::ComputerCore => SystemFlags::COMPUTER,
            RoomType::Bridge => SystemFlags::COMPUTER | SystemFlags::ELECTRICAL,
            // Airlocks are physical connectors; they don't introduce a new system type.
            RoomType::Airlock => SystemFlags::empty(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoomNode {
    pub id: RoomId,
    pub room_type: RoomType,
    pub shape: Rect,
    pub system_flags: SystemFlags,
}

impl RoomNode {
    pub fn new(id: RoomId, room_type: RoomType, shape: Rect) -> Self {
        Self {
            id,
            system_flags: room_type.system_flags(),
            room_type,
            shape,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeKind {
    Door,
    Corridor,
    Airlock,
    Umbilical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgePlacement {
    North,
    South,
    East,
    West,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoomEdge {
    pub kind: EdgeKind,
    pub from: RoomId,
    pub to: RoomId,
    pub location: EdgePlacement,
}

#[derive(Clone, Debug, Default)]
pub struct RoomGraph {
    nodes: HashMap<RoomId, RoomNode>,
    edges: Vec<RoomEdge>,
    adjacency: HashMap<RoomId, Vec<usize>>,
}

impl RoomGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
        }
    }

    /// Adds a room node to the graph.
    pub fn add_node(&mut self, node: RoomNode) {
        self.nodes.insert(node.id, node);
    }

    /// Adds an undirected edge between two existing room nodes.
    /// Returns `false` if either end node is missing.
    pub fn add_edge(&mut self, edge: RoomEdge) -> bool {
        if !self.nodes.contains_key(&edge.from) || !self.nodes.contains_key(&edge.to) {
            return false;
        }

        let from = edge.from;
        let to = edge.to;

        let idx = self.edges.len();
        self.edges.push(edge);

        self.adjacency.entry(from).or_default().push(idx);
        self.adjacency.entry(to).or_default().push(idx);
        true
    }

    pub fn get_node(&self, id: RoomId) -> Option<&RoomNode> {
        self.nodes.get(&id)
    }

    /// Returns an iterator over all room nodes in arbitrary order.
    pub fn nodes(&self) -> impl Iterator<Item = &RoomNode> {
        self.nodes.values()
    }

    /// Returns all edges in insertion order.
    pub fn edges(&self) -> &[RoomEdge] {
        &self.edges
    }

    /// Returns the IDs of all neighbors of `id`.
    pub fn neighbors(&self, id: RoomId) -> Vec<RoomId> {
        let mut result = Vec::new();
        if let Some(indices) = self.adjacency.get(&id) {
            for &edge_idx in indices {
                let edge = &self.edges[edge_idx];
                let other = if edge.from == id { edge.to } else { edge.from };
                result.push(other);
            }
        }
        result
    }

    /// Returns `true` if all nodes are reachable from every other node.
    pub fn is_connected(&self) -> bool {
        if self.nodes.is_empty() {
            return true;
        }

        let start = *self.nodes.keys().next().unwrap();
        let mut visited = HashSet::new();
        let mut stack = vec![start];

        while let Some(id) = stack.pop() {
            if visited.insert(id) {
                for neighbor in self.neighbors(id) {
                    stack.push(neighbor);
                }
            }
        }

        visited.len() == self.nodes.len()
    }
}
