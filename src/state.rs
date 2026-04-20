use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign};

pub const MAX_CELL_COUNT: usize = 1024;
pub const DEFAULT_SIZE: (usize, usize) = (64, 64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridType {
    Square,
    SquareDiagonal,
    Hexagonal,
    Triangular,
}

#[derive(Debug, Clone)]
struct Field {
    width: usize,
    height: usize,
    values: Vec<Vec<u8>>,
}

impl Default for Field {
    fn default() -> Self {
        let width = DEFAULT_SIZE.0;
        let height = DEFAULT_SIZE.1;
        Self {
            width,
            height,
            values: vec![vec![0; MAX_CELL_COUNT]; MAX_CELL_COUNT],
        }
    }
}

/// For square grid directions are:
/// ```no_run
///   7 | 0 | 1
///  -----------
///   6 |   | 2
///  -----------
///   5 | 4 | 3
///```
///
/// for hexagonal grid directions are:
///```no_run
///          _ _
///         /     \
///    _ _ /   0   \ _ _
///  /     \       /     \
/// /   7   \ _ _ /   1   \
/// \       /     \       /
///  \ _ _ /       \ _ _ /
///  /     \       /     \
/// /   5   \ _ _ /   3   \
/// \       /     \       /
///  \ _ _ /   4   \ _ _ /
///        \       /
///         \ _ _ /
/// ```
///
/// for triangular grid?:
/// ```no_run
///      ._ _ _.
///     / \ 0 / \
///   ./_ _\./_ _\.
///  / \ 7 / \ 1 / \
/// /_ _\./_ _\./_ _\
/// \ 5 / \ 4 / \ 3 /
///  \./_ _\./_ _\ /
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::North => f.write_str("↑"),
            Direction::NorthEast => f.write_str("↗"),
            Direction::East => f.write_str("→"),
            Direction::SouthEast => f.write_str("↘"),
            Direction::South => f.write_str("↓"),
            Direction::SouthWest => f.write_str("↙"),
            Direction::West => f.write_str("←"),
            Direction::NorthWest => f.write_str("↖"),
        }
    }
}

impl Add for Direction {
    type Output = Direction;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result += rhs;
        result
    }
}

impl AddAssign for Direction {
    fn add_assign(&mut self, rhs: Self) {
        *self = Direction::from(*self as u8 + rhs as u8);
    }
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value % 8 {
            0 => Direction::North,
            1 => Direction::NorthEast,
            2 => Direction::East,
            3 => Direction::SouthEast,
            4 => Direction::South,
            5 => Direction::SouthWest,
            6 => Direction::West,
            7 => Direction::NorthWest,
            _ => unreachable!(),
        }
    }
}

impl Direction {
    fn to_hexagon_u8(&self) -> u8 {
        match self {
            Direction::North => 0,
            Direction::NorthEast | Direction::East => 1,
            Direction::SouthEast => 2,
            Direction::South => 3,
            Direction::SouthWest | Direction::West => 4,
            Direction::NorthWest => 5,
        }
    }

    fn from_hexagon_u8(value: u8) -> Direction {
        match value % 6 {
            0 => Direction::North,
            1 => Direction::NorthEast,
            2 => Direction::SouthEast,
            3 => Direction::South,
            4 => Direction::SouthWest,
            5 => Direction::NorthWest,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Instruction {
    /// Map from current palette index to next palette index and direction
    pub map: BTreeMap<u8, (u8, Option<Direction>)>,
}

impl Default for Instruction {
    fn default() -> Self {
        Self {
            map: [
                (0, (1, Some(Direction::East))),
                (1, (2, Some(Direction::West))),
                (2, (3, Some(Direction::North))),
                (3, (4, Some(Direction::East))),
                (4, (5, Some(Direction::West))),
                (5, (6, Some(Direction::South))),
                (6, (7, Some(Direction::East))),
                (7, (0, Some(Direction::West))),
            ]
            .into_iter()
            .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
    pub orientation: Direction,
}

#[derive(Clone, Debug)]
pub struct Ant {
    pub start_position: Position,
    pub instruction: usize,
    position: Position,
}

impl Ant {
    fn travel(&mut self, grid_type: GridType, direction: Direction, width: usize, height: usize) {
        match grid_type {
            GridType::SquareDiagonal | GridType::Square => {
                self.position.orientation += effective_direction(grid_type, direction);

                match self.position.orientation {
                    Direction::North | Direction::NorthEast | Direction::NorthWest => {
                        if self.position.y == 0 {
                            self.position.y = height - 1;
                        } else {
                            self.position.y -= 1;
                        }
                    }
                    Direction::South | Direction::SouthEast | Direction::SouthWest => {
                        self.position.y = (self.position.y + 1) % height;
                    }
                    Direction::East | Direction::West => {}
                }
                match self.position.orientation {
                    Direction::West | Direction::NorthWest | Direction::SouthWest => {
                        if self.position.x == 0 {
                            self.position.x = width - 1;
                        } else {
                            self.position.x = (self.position.x - 1) % width;
                        }
                    }
                    Direction::East | Direction::SouthEast | Direction::NorthEast => {
                        self.position.x = (self.position.x + 1) % width;
                    }
                    Direction::North | Direction::South => {}
                }
            }
            GridType::Hexagonal => {
                let current = self.position.orientation.to_hexagon_u8();
                let direction = effective_direction(grid_type, direction).to_hexagon_u8();
                let next = (current + direction) % 6;
                self.position.orientation = Direction::from_hexagon_u8(next);
                println!("current = {:?}, direction = {:?}, next = {:?}, orientation={:?}", current, direction, next, self.position.orientation);
                match self.position.orientation {
                    Direction::North => {
                        if self.position.y == 0 {
                            self.position.y = height - 1;
                        } else {
                            self.position.y -= 1;
                        }
                    },
                    Direction::NorthEast | Direction::NorthWest if self.position.x % 2 == 0  => {
                        if self.position.y == 0 {
                            self.position.y = height - 1;
                        } else {
                            self.position.y -= 1;
                        }
                    }
                    Direction::South => {
                        self.position.y = (self.position.y + 1) % height;
                    }
                    Direction::SouthEast | Direction::SouthWest if self.position.x % 2 != 0 => {
                        self.position.y = (self.position.y + 1) % height;
                    }
                    _ => {}
                }
                match self.position.orientation {
                    Direction::NorthWest | Direction::SouthWest => {
                        if self.position.x == 0 {
                            self.position.x = width - 1;
                        } else {
                            self.position.x -= 1;
                        }
                    },
                    Direction::SouthEast | Direction::NorthEast => {
                        self.position.x = (self.position.x + 1) % width;
                    }
                    _ => {}
                }
            }
            GridType::Triangular => {
                match (
                    effective_direction(grid_type, direction),
                    self.position.orientation,
                ) {
                    (Direction::South, Direction::SouthWest)
                    | (Direction::East, Direction::North)
                    | (Direction::West, Direction::SouthEast) => {
                        self.position.orientation = Direction::NorthEast;
                        self.position.x = (self.position.x + 1) % width;
                    }
                    (Direction::South, Direction::North)
                    | (Direction::East, Direction::SouthEast)
                    | (Direction::West, Direction::SouthWest) => {
                        self.position.orientation = Direction::South;
                        self.position.y = (self.position.y + 1) % height;
                    }
                    (Direction::South, Direction::NorthEast)
                    | (Direction::East, Direction::South)
                    | (Direction::West, Direction::NorthWest) => {
                        self.position.orientation = Direction::SouthWest;
                        if self.position.x == 0 {
                            self.position.x = width - 1;
                        } else {
                            self.position.x = (self.position.x - 1) % width;
                        }
                    }
                    (Direction::South, Direction::SouthEast)
                    | (Direction::East, Direction::SouthWest)
                    | (Direction::West, Direction::North) => {
                        self.position.orientation = Direction::NorthWest;
                        if self.position.x == 0 {
                            self.position.x = width - 1;
                        } else {
                            self.position.x = (self.position.x - 1) % width;
                        }
                    }
                    (Direction::South, Direction::South)
                    | (Direction::East, Direction::NorthWest)
                    | (Direction::West, Direction::NorthEast) => {
                        self.position.orientation = Direction::North;
                        if self.position.y == 0 {
                            self.position.y = height - 1;
                        } else {
                            self.position.y -= 1;
                        }
                    }
                    (Direction::South, Direction::NorthWest)
                    | (Direction::East, Direction::NorthEast)
                    | (Direction::West, Direction::South) => {
                        self.position.orientation = Direction::SouthEast;
                        self.position.x = (self.position.x + 1) % width;
                    }
                    (_, Direction::West) | (_, Direction::East) => {}
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl Default for Ant {
    fn default() -> Self {
        let x0 = DEFAULT_SIZE.0 / 2;
        let y0 = DEFAULT_SIZE.1 / 2;
        let position = Position {
            x: x0,
            y: y0,
            orientation: Direction::North,
        };

        Self {
            start_position: position.clone(),
            position,
            instruction: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    snapshots: BTreeMap<usize, State>,
    generation: usize,
    pub ants: Vec<Ant>,
    field: Field,
    pub instructions: Vec<Instruction>,
    pub grid_type: GridType,
}

impl Default for State {
    fn default() -> Self {
        Self {
            snapshots: Default::default(),
            generation: 0,
            ants: vec![Ant::default()],
            field: Field::default(),
            instructions: vec![Instruction::default()],
            grid_type: GridType::SquareDiagonal,
        }
    }
}

impl State {
    pub fn after(steps: usize) -> Self {
        let mut result = Self::default();
        result.step(steps);
        result
    }

    pub fn go_to_step(&mut self, step: usize) {
        if let Some(diff) = step.checked_sub(self.generation) {
            self.step(diff);
        } else if let Some((record, snapshot)) =
            self.snapshots.iter().filter(|(a, _)| **a <= step).last()
        {
            self.generation = *record;
            self.field = snapshot.field.clone();
            self.ants = snapshot.ants.clone();
            self.instructions = snapshot.instructions.clone();
            self.snapshots.retain(|a, _| *a < step);
            self.step(step - self.generation);
        }
    }

    pub fn step(&mut self, count: usize) {
        for _ in 0..count {
            if self.generation % 100_000 == 0 {
                self.snapshots.insert(
                    self.generation,
                    State {
                        snapshots: Default::default(),
                        ..self.clone()
                    },
                );
            }
            self.generation += 1;
            for ant in &mut self.ants {
                let next = &self.instructions[ant.instruction].map
                    [&self.field.values[ant.position.x][ant.position.y]];
                self.field.values[ant.position.x][ant.position.y] = next.0;
                if let Some(direction) = next.1 {
                    ant.travel(
                        self.grid_type,
                        direction,
                        self.field.width,
                        self.field.height,
                    );
                }
            }
        }
    }

    /// Iterator over ant positions.
    pub fn ants(&self) -> impl Iterator<Item = (usize, usize)> {
        self.ants.iter().map(|ant| (ant.position.x, ant.position.y))
    }

    pub fn field_size(&self) -> (usize, usize) {
        (self.field.width, self.field.height)
    }

    pub fn field_at(&self, x: usize, y: usize) -> usize {
        self.field.values[x % self.field.width][y % self.field.height] as usize
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn add_ant(&mut self, x: usize, y: usize, instruction: usize) {
        let x = x % self.field.width;
        let y = y % self.field.height;
        if self
            .ants
            .iter()
            .any(|ant| ant.start_position.x == x && ant.start_position.y == y)
        {
            return;
        }
        let position = Position {
            x,
            y,
            orientation: Direction::North,
        };
        self.ants.push(Ant {
            start_position: position.clone(),
            position,
            instruction: instruction % self.instructions.len(),
        });
    }

    pub fn remove_ant(&mut self, x: usize, y: usize) -> bool {
        if let Some((i, _)) = self
            .ants
            .iter()
            .enumerate()
            .find(|(_, ant)| ant.position.x == x && ant.position.y == y)
        {
            self.ants.remove(i);
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) -> usize {
        let steps = self.generation;
        self.generation = 0;
        self.field.values = vec![vec![0; MAX_CELL_COUNT]; MAX_CELL_COUNT];
        for ant in &mut self.ants {
            ant.position = ant.start_position.clone();
        }
        steps
    }

    pub fn recalculate(&mut self) {
        let steps = self.reset();
        self.step(steps);
    }

    pub fn set_width(&mut self, width: usize) {
        for ant in &mut self.ants {
            ant.start_position.x =
                (ant.start_position.x as f64 / self.field.width as f64 * width as f64) as usize;
        }
        self.field.width = width;
    }

    pub fn set_height(&mut self, height: usize) {
        for ant in &mut self.ants {
            ant.start_position.y =
                (ant.start_position.y as f64 / self.field.height as f64 * height as f64) as usize;
        }
        self.field.height = height;
    }
}

pub fn prev_direction(grid_type: GridType, original: Option<Direction>) -> Option<Direction> {
    match grid_type {
        GridType::SquareDiagonal => match original {
            None => Some(Direction::NorthWest),
            Some(Direction::North) => None,
            Some(direction) => Some(direction + Direction::NorthWest),
        },
        GridType::Square => match original {
            Some(Direction::North) => None,
            Some(direction) => Some(direction + Direction::West),
            None => Some(Direction::West),
        },
        GridType::Hexagonal => match original {
            Some(Direction::North) => None,
            Some(Direction::NorthWest) => Some(Direction::SouthWest),
            Some(Direction::SouthEast) => Some(Direction::NorthEast),
            Some(direction) => Some(direction + Direction::NorthWest),
            None => Some(Direction::NorthWest),
        },
        GridType::Triangular => match original {
            None => Some(Direction::West),
            Some(Direction::West) => Some(Direction::South),
            Some(Direction::South) => Some(Direction::East),
            _ => None,
        },
    }
}

pub fn next_direction(grid_type: GridType, original: Option<Direction>) -> Option<Direction> {
    match grid_type {
        GridType::SquareDiagonal => match original {
            Some(Direction::NorthWest) => None,
            Some(direction) => Some(direction + Direction::NorthEast),
            None => Some(Direction::North),
        },
        GridType::Square => match original {
            Some(Direction::West) => None,
            Some(direction) => Some(direction + Direction::East),
            None => Some(Direction::North),
        },
        GridType::Hexagonal => match original {
            Some(Direction::NorthWest) => None,
            Some(Direction::SouthWest) => Some(Direction::NorthWest),
            Some(Direction::NorthEast) => Some(Direction::SouthEast),
            Some(direction) => Some(direction + Direction::NorthEast),
            None => Some(Direction::North),
        },
        GridType::Triangular => match original {
            None => Some(Direction::East),
            Some(Direction::East) => Some(Direction::South),
            Some(Direction::South) => Some(Direction::West),
            _ => None,
        },
    }
}

pub fn next_grid_type(grid_type: GridType) -> GridType {
    match grid_type {
        GridType::Square => GridType::SquareDiagonal,
        GridType::SquareDiagonal => GridType::Hexagonal,
        GridType::Hexagonal => GridType::Triangular,
        GridType::Triangular => GridType::Square,
    }
}

pub fn effective_direction(grid_type: GridType, direction: Direction) -> Direction {
    match grid_type {
        GridType::SquareDiagonal => direction,
        GridType::Square => match direction {
            Direction::NorthEast | Direction::SouthEast => Direction::East,
            Direction::SouthWest | Direction::NorthWest => Direction::West,
            direction => direction,
        },
        GridType::Hexagonal => match direction {
            Direction::West => Direction::NorthWest,
            Direction::East => Direction::NorthEast,
            direction => direction,
        },
        GridType::Triangular => match direction {
            Direction::North => Direction::South,
            Direction::NorthEast | Direction::SouthEast => Direction::East,
            Direction::SouthWest | Direction::NorthWest => Direction::West,
            direction => direction,
        },
    }
}
