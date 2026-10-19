use std::ops::RangeInclusive;

use crate::procgen::scaffold::{RoomGraph, RoomType, SystemFlags};
use crate::procgen::tile::TileType;
use crate::testing::debug_map::{render_debug_map, render_passability_map};
use crate::testing::map_report::MapReport;

/// Builder-style map contract tester.
///
/// The point is to describe generated map requirements at a high level.
/// Every failed assertion is collected, so a failing test can print one
/// complete map autopsy.
#[derive(Debug)]
pub struct MapAssert<'a> {
    test_name: String,
    graph: &'a RoomGraph,
    grid: &'a [Vec<TileType>],
    report: MapReport,
}

impl<'a> MapAssert<'a> {
    pub fn new(test_name: impl Into<String>, grid: &'a [Vec<TileType>], graph: &'a RoomGraph) -> Self {
        Self {
            test_name: test_name.into(),
            graph,
            grid,
            report: MapReport::new(),
        }
    }

    fn grid_width(&self) -> usize {
        self.grid.first().map(|row| row.len()).unwrap_or(0)
    }

    fn grid_height(&self) -> usize {
        self.grid.len()
    }

    fn passable_neighbor_count(&self, x: usize, y: usize) -> usize {
        let width = self.grid_width();
        let height = self.grid_height();

        let mut count = 0;

        // North.
        if y + 1 < height && self.grid[y + 1][x].is_passable() {
            count += 1;
        }

        // South.
        if y > 0 && self.grid[y - 1][x].is_passable() {
            count += 1;
        }

        // East.
        if x + 1 < width && self.grid[y][x + 1].is_passable() {
            count += 1;
        }

        // West.
        if x > 0 && self.grid[y][x - 1].is_passable() {
            count += 1;
        }

        count
    }

    pub fn size(mut self, width: usize, height: usize) -> Self {
        let actual_width = self.grid_width();
        let actual_height = self.grid_height();

        if actual_width == width && actual_height == height {
            self.report.pass("dimensions");
        } else {
            self.report.fail(
                "dimensions",
                format!(
                    "expected {width}x{height}, got {actual_width}x{actual_height}"
                ),
            );
        }

        self
    }

    pub fn no_orphan_floor(mut self) -> Self {
        for y in 0..self.grid_height() {
            for x in 0..self.grid_width() {
                let tile = self.grid[y][x];

                if tile.is_passable() && self.passable_neighbor_count(x, y) == 0 {
                    self.report.fail(
                        "no_orphan_floor",
                        format!("isolated passable tile at ({x}, {y})"),
                    );
                    return self;
                }
            }
        }

        self.report.pass("no_orphan_floor");
        self
    }

    pub fn passable_connected(mut self) -> Self {
        let width = self.grid_width();
        let height = self.grid_height();

        let mut passable_count = 0;
        let mut start = None;

        for y in 0..height {
            for x in 0..width {
                if self.grid[y][x].is_passable() {
                    passable_count += 1;
                    if start.is_none() {
                        start = Some((x, y));
                    }
                }
            }
        }

        if passable_count == 0 {
            self.report.fail("passable_connected", "no passable tiles found");
            return self;
        }

        let (start_x, start_y) = start.unwrap();
        let mut visited = vec![vec![false; width]; height];
        let mut stack = vec![(start_x, start_y)];
        let mut visited_count = 0;

        while let Some((x, y)) = stack.pop() {
            if visited[y][x] {
                continue;
            }

            visited[y][x] = true;
            visited_count += 1;

            // North.
            if y + 1 < height && self.grid[y + 1][x].is_passable() {
                stack.push((x, y + 1));
            }

            // South.
            if y > 0 && self.grid[y - 1][x].is_passable() {
                stack.push((x, y - 1));
            }

            // East.
            if x + 1 < width && self.grid[y][x + 1].is_passable() {
                stack.push((x + 1, y));
            }

            // West.
            if x > 0 && self.grid[y][x - 1].is_passable() {
                stack.push((x - 1, y));
            }
        }

        if visited_count == passable_count {
            self.report.pass("passable_connected");
        } else {
            self.report.fail(
                "passable_connected",
                format!(
                    "only {visited_count}/{passable_count} passable tiles are connected"
                ),
            );
        }

        self
    }

    pub fn room_graph_connected(mut self) -> Self {
        if self.graph.is_connected() {
            self.report.pass("room_graph_connected");
        } else {
            self.report.fail("room_graph_connected", "room graph is disconnected");
        }

        self
    }

    pub fn no_orphan_doors(mut self) -> Self {
        let mut failures = Vec::new();

        for y in 0..self.grid_height() {
            for x in 0..self.grid_width() {
                if matches!(self.grid[y][x], TileType::Door(_)) {
                    let passable_neighbors = self.passable_neighbor_count(x, y);

                    if passable_neighbors < 2 {
                        failures.push(format!("({x}, {y}) has {passable_neighbors} passable neighbors"));
                    }
                }
            }
        }

        if failures.is_empty() {
            self.report.pass("no_orphan_doors");
        } else {
            self.report.fail("no_orphan_doors", failures.join("; "));
        }

        self
    }

    pub fn has_room_type(mut self, room_type: RoomType) -> Self {
        if self.graph.nodes().any(|node| node.room_type == room_type) {
            self.report.pass(format!("has_room_type({room_type:?})"));
        } else {
            self.report.fail(
                format!("has_room_type({room_type:?})"),
                "room type is missing",
            );
        }

        self
    }

    pub fn room_count_between(mut self, range: RangeInclusive<usize>) -> Self {
        let count = self.graph.nodes().count();

        if range.contains(&count) {
            self.report.pass(format!("room_count_between({range:?})"));
        } else {
            self.report.fail(
                format!("room_count_between({range:?})"),
                format!("got {count} rooms"),
            );
        }

        self
    }

    pub fn room_has_system(mut self, room_type: RoomType, flags: SystemFlags) -> Self {
        let Some(room) = self.graph.nodes().find(|node| node.room_type == room_type) else {
            self.report.fail(
                format!("room_has_system({room_type:?}, {flags:?})"),
                "room type is missing",
            );
            return self;
        };

        if room.system_flags.contains(flags) {
            self.report.pass(format!("room_has_system({room_type:?}, {flags:?})"));
        } else {
            self.report.fail(
                format!("room_has_system({room_type:?}, {flags:?})"),
                format!("room flags are {:?}", room.system_flags),
            );
        }

        self
    }

    pub fn system_density(mut self, flags: SystemFlags, range: RangeInclusive<f32>) -> Self {
        let total = self.grid_width() * self.grid_height();
        if total == 0 {
            self.report.fail("system_density", "grid is empty");
            return self;
        }

        let system_count = self
            .grid
            .iter()
            .flatten()
            .filter(|tile| {
                if flags.contains(SystemFlags::ELECTRICAL) {
                    matches!(tile, TileType::Conduit)
                } else if flags.contains(SystemFlags::FLUIDICS) {
                    matches!(tile, TileType::Pipe)
                } else {
                    false
                }
            })
            .count();

        let density = system_count as f32 / total as f32;

        if range.contains(&density) {
            self.report.pass(format!("system_density({flags:?}, {range:?})"));
        } else {
            self.report.fail(
                format!("system_density({flags:?}, {range:?})"),
                format!("density is {density:.3}"),
            );
        }

        self
    }

    pub fn system_line_connectivity(mut self, flags: SystemFlags, max_components: usize) -> Self {
        let width = self.grid_width();
        let height = self.grid_height();

        if width == 0 || height == 0 {
            self.report.fail("system_line_connectivity", "grid is empty");
            return self;
        }

        let mut visited = vec![vec![false; width]; height];
        let mut components = 0usize;

        for y in 0..height {
            for x in 0..width {
                if visited[y][x] {
                    continue;
                }

                let tile = self.grid[y][x];
                let is_system_tile = flags.contains(SystemFlags::ELECTRICAL)
                    && matches!(tile, TileType::Conduit)
                    || flags.contains(SystemFlags::FLUIDICS)
                        && matches!(tile, TileType::Pipe);

                if !is_system_tile {
                    continue;
                }

                components += 1;
                visited[y][x] = true;

                let mut stack = vec![(x, y)];

                while let Some((cx, cy)) = stack.pop() {
                    let neighbor_dirs = [
                        (0isize, 1isize),
                        (0isize, -1isize),
                        (1isize, 0isize),
                        (-1isize, 0isize),
                    ];

                    for (dx, dy) in neighbor_dirs {
                        let nx = cx as isize + dx;
                        let ny = cy as isize + dy;

                        if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                            continue;
                        }

                        let nx = nx as usize;
                        let ny = ny as usize;

                        if visited[ny][nx] {
                            continue;
                        }

                        let neighbor_tile = self.grid[ny][nx];
                        let is_neighbor_system_tile =
                            flags.contains(SystemFlags::ELECTRICAL)
                                && matches!(neighbor_tile, TileType::Conduit)
                                || flags.contains(SystemFlags::FLUIDICS)
                                    && matches!(neighbor_tile, TileType::Pipe);

                        if is_neighbor_system_tile {
                            visited[ny][nx] = true;
                            stack.push((nx, ny));
                        }
                    }
                }
            }
        }

        if components <= max_components {
            self.report.pass(format!(
                "system_line_connectivity({flags:?}, max_components={max_components})"
            ));
        } else {
            self.report.fail(
                format!("system_line_connectivity({flags:?}, max_components={max_components})"),
                format!("found {components} connected system-tile components"),
            );
        }

        self
    }

    pub fn forbidden_2x2(
        mut self,
        patterns: &[[[TileType; 2]; 2]],
    ) -> Self {
        let height = self.grid_height();
        let width = self.grid_width();

        let mut found = false;
        let mut details = Vec::new();

        for y in 0..height.saturating_sub(1) {
            for x in 0..width.saturating_sub(1) {
                for pattern in patterns {
                    if self.grid[y][x] == pattern[0][0]
                        && self.grid[y][x + 1] == pattern[0][1]
                        && self.grid[y + 1][x] == pattern[1][0]
                        && self.grid[y + 1][x + 1] == pattern[1][1]
                    {
                        found = true;
                        details.push(format!("({x}, {y})"));
                    }
                }
            }
        }

        if found {
            self.report.fail("forbidden_2x2", details.join("; "));
        } else {
            self.report.pass("forbidden_2x2");
        }

        self
    }

    pub fn assert(&self) {
        if !self.report.has_failures() {
            return;
        }

        let mut output = String::new();
        output.push_str(&format!("[FAIL] MapAssert {}\n", self.test_name));
        output.push_str("\nMap:\n");
        output.push_str(&render_debug_map(self.grid));
        output.push_str("\nPassability:\n");
        output.push_str(&render_passability_map(self.grid));
        output.push_str("\nInvariant summary:\n");
        output.push_str(&self.report.format_summary());

        panic!("{output}");
    }
}
