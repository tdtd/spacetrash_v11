use std::collections::{HashMap, HashSet, VecDeque};

pub type TileId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TileAdjacency {
    pub north: HashSet<TileId>,
    pub south: HashSet<TileId>,
    pub east: HashSet<TileId>,
    pub west: HashSet<TileId>,
}

impl TileAdjacency {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug)]
pub struct WfcTileSet {
    pub tiles: HashSet<TileId>,
    pub adjacency: HashMap<TileId, TileAdjacency>,
}

impl WfcTileSet {
    pub fn new(tiles: HashSet<TileId>, adjacency: HashMap<TileId, TileAdjacency>) -> Self {
        Self { tiles, adjacency }
    }

    pub fn allowed_in_direction(&self, tile: TileId, dir: Direction) -> Option<&HashSet<TileId>> {
        let adj = self.adjacency.get(&tile)?;
        Some(match dir {
            Direction::North => &adj.north,
            Direction::South => &adj.south,
            Direction::East => &adj.east,
            Direction::West => &adj.west,
        })
    }
}

#[derive(Clone, Debug)]
pub struct WfcSettings {
    pub seed: u64,
}

impl Default for WfcSettings {
    fn default() -> Self {
        Self { seed: 0 }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WfcBalanceProfile {
    pub tile_weight_multipliers: HashMap<TileId, f32>,
}

impl WfcBalanceProfile {
    fn apply_to_weights(&self, base_weights: &mut HashMap<TileId, f32>) {
        for (tile_id, multiplier) in &self.tile_weight_multipliers {
            if let Some(weight) = base_weights.get_mut(tile_id) {
                *weight *= multiplier;
            }
        }
    }
}

struct WfcRng(u64);

impl WfcRng {
    fn new(seed: u64) -> Self {
        let mut rng = Self(seed.max(1));

        // Warm up the generator. Without this, small seeds produce
        // non-full-width first outputs, making `next_f32()` return 0.0.
        for _ in 0..8 {
            rng.next_u64();
        }

        rng
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / ((1u64 << 24) as f32)
    }
}

#[derive(Clone, Debug, Default)]
pub struct WfcGrid {
    width: usize,
    height: usize,
    cells: Vec<Vec<HashSet<TileId>>>,
}

impl WfcGrid {
    pub fn new(width: usize, height: usize, all_tiles: &HashSet<TileId>) -> Self {
        let cells = vec![vec![all_tiles.clone(); width]; height];
        Self {
            width,
            height,
            cells,
        }
    }

    /// Creates a WFC grid with per-cell initial domains.
    pub fn new_with_domains(width: usize, height: usize, domains: Vec<Vec<HashSet<TileId>>>) -> Self {
        assert_eq!(domains.len(), height);
        assert!(domains.iter().all(|row| row.len() == width));

        Self {
            width,
            height,
            cells: domains,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&HashSet<TileId>> {
        self.cells.get(y).and_then(|row| row.get(x))
    }

    pub fn set(&mut self, x: usize, y: usize, tiles: HashSet<TileId>) {
        self.cells[y][x] = tiles;
    }

    pub fn cell_entropy(&self, x: usize, y: usize) -> usize {
        self.cells[y][x].len()
    }

    pub fn is_collapsed(&self) -> bool {
        self.cells
            .iter()
            .all(|row| row.iter().all(|cell| cell.len() == 1))
    }

    pub fn collapsed_tile(&self, x: usize, y: usize) -> Option<TileId> {
        let cell = self.get(x, y)?;
        if cell.len() == 1 {
            cell.iter().next().copied()
        } else {
            None
        }
    }

    /// Returns the coordinates of the cell with minimum entropy > 1.
    pub fn min_entropy_cell(&self) -> Option<(usize, usize)> {
        let mut min_entropy = usize::MAX;
        let mut best: Option<(usize, usize)> = None;
        for y in 0..self.height {
            for x in 0..self.width {
                let entropy = self.cell_entropy(x, y);
                if entropy > 1 && entropy < min_entropy {
                    min_entropy = entropy;
                    best = Some((x, y));
                }
            }
        }
        best
    }
}

#[derive(Clone, Debug)]
pub struct WfcResult {
    pub tiles: Vec<Vec<TileId>>,
}

/// Runs the Wave Function Collapse algorithm on the provided grid.
///
/// The algorithm is deterministic: when multiple tiles are possible,
/// the smallest tile ID is selected. Propagation continues until all
/// cells are collapsed or a contradiction is detected.
pub fn run_wfc(grid: &mut WfcGrid, tile_set: &WfcTileSet) -> Result<WfcResult, String> {
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    while !grid.is_collapsed() {
        let Some((x, y)) = grid.min_entropy_cell() else {
            break;
        };

        // Choose deterministically: smallest tile ID.
        let chosen = *grid
            .get(x, y)
            .unwrap()
            .iter()
            .min()
            .expect("min entropy cell must have at least one tile");
        grid.set(x, y, HashSet::from([chosen]));
        queue.push_back((x, y));

        while let Some((cx, cy)) = queue.pop_front() {
            let collapsed = grid.collapsed_tile(cx, cy).unwrap();

            let neighbor_dirs = [
                (Direction::North, 0, 1),
                (Direction::South, 0, -1),
                (Direction::East, 1, 0),
                (Direction::West, -1, 0),
            ];

            for (dir, dx, dy) in neighbor_dirs {
                let nx = cx as isize + dx;
                let ny = cy as isize + dy;
                if nx < 0 || ny < 0 || nx >= grid.width as isize || ny >= grid.height as isize {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;

                if grid.cell_entropy(nx, ny) == 1 {
                    continue;
                }

                let allowed = tile_set
                    .allowed_in_direction(collapsed, dir)
                    .ok_or_else(|| {
                        format!(
                            "Missing adjacency for tile {} direction {:?}",
                            collapsed, dir
                        )
                    })?;

                let mut new_poss = grid.get(nx, ny).unwrap().clone();
                new_poss.retain(|&t| allowed.contains(&t));

                if new_poss.is_empty() {
                    return Err(format!("Contradiction at cell ({}, {})", nx, ny));
                }

                let updated = new_poss.len() != grid.cell_entropy(nx, ny);
                if updated {
                    grid.set(nx, ny, new_poss);
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    let mut tiles = Vec::with_capacity(grid.height);
    for y in 0..grid.height {
        let mut row = Vec::with_capacity(grid.width);
        for x in 0..grid.width {
            row.push(
                grid.collapsed_tile(x, y)
                    .expect("all cells collapsed after algorithm"),
            );
        }
        tiles.push(row);
    }

    Ok(WfcResult { tiles })
}

/// Runs WFC with configurable random seed and tile weight multipliers.
///
/// `balance` allows game-balance tuning without changing the existing
/// deterministic `run_wfc` smallest-tile behavior.
pub fn run_wfc_with_settings(
    grid: &mut WfcGrid,
    tile_set: &WfcTileSet,
    settings: &WfcSettings,
    balance: &WfcBalanceProfile,
) -> Result<WfcResult, String> {
    let mut rng = WfcRng::new(settings.seed);
    let mut weights: HashMap<TileId, f32> = tile_set
        .tiles
        .iter()
        .map(|&tile| (tile, 1.0))
        .collect();

    balance.apply_to_weights(&mut weights);

    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    while !grid.is_collapsed() {
        let Some((x, y)) = grid.min_entropy_cell() else {
            break;
        };

        let possibilities = grid.get(x, y).unwrap().clone();
        let mut candidate_tiles: Vec<TileId> = possibilities.into_iter().collect();
        candidate_tiles.sort_unstable();

        let mut total_weight = 0.0f32;
        for &tile in &candidate_tiles {
            let weight = weights.get(&tile).copied().unwrap_or(1.0);
            if weight <= 0.0 {
                return Err(format!("non-positive weight for tile {}", tile));
            }
            total_weight += weight;
        }

        let mut target = rng.next_f32() * total_weight;
        let mut chosen = candidate_tiles[0];

        for &tile in &candidate_tiles {
            let weight = weights.get(&tile).copied().unwrap_or(1.0);
            if target < weight {
                chosen = tile;
                break;
            }
            target -= weight;
        }

        grid.set(x, y, HashSet::from([chosen]));
        queue.push_back((x, y));

        while let Some((cx, cy)) = queue.pop_front() {
            let collapsed = grid.collapsed_tile(cx, cy).unwrap();

            let neighbor_dirs = [
                (Direction::North, 0, 1),
                (Direction::South, 0, -1),
                (Direction::East, 1, 0),
                (Direction::West, -1, 0),
            ];

            for (dir, dx, dy) in neighbor_dirs {
                let nx = cx as isize + dx;
                let ny = cy as isize + dy;

                if nx < 0 || ny < 0 || nx >= grid.width as isize || ny >= grid.height as isize {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;

                if grid.cell_entropy(nx, ny) == 1 {
                    continue;
                }

                let allowed = tile_set
                    .allowed_in_direction(collapsed, dir)
                    .ok_or_else(|| {
                        format!(
                            "missing adjacency for tile {} direction {:?}",
                            collapsed, dir
                        )
                    })?;

                let mut new_poss = grid.get(nx, ny).unwrap().clone();
                new_poss.retain(|&tile| allowed.contains(&tile));

                if new_poss.is_empty() {
                    return Err(format!("contradiction at cell ({}, {})", nx, ny));
                }

                if new_poss.len() != grid.cell_entropy(nx, ny) {
                    grid.set(nx, ny, new_poss);
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    let mut tiles = Vec::with_capacity(grid.height);
    for y in 0..grid.height {
        let mut row = Vec::with_capacity(grid.width);
        for x in 0..grid.width {
            row.push(
                grid.collapsed_tile(x, y)
                    .expect("all cells collapsed after algorithm"),
            );
        }
        tiles.push(row);
    }

    Ok(WfcResult { tiles })
}
