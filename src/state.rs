use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign};

pub const MAX_CELL_COUNT: usize = 1024;
pub const DEFAULT_SIZE: (usize, usize) = (256, 256);

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
    fn travel(&mut self, direction: Direction, width: usize, height: usize) {
        self.position.orientation += direction;

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
}

impl Default for Ant {
    fn default() -> Self {
        let x0 = DEFAULT_SIZE.0 / 2;
        let y0 = DEFAULT_SIZE.1 / 2;
        let position = Position {
            x: x0,
            y: y0,
            orientation: Direction::West,
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
}

impl Default for State {
    fn default() -> Self {
        Self {
            snapshots: Default::default(),
            generation: 0,
            ants: vec![Ant::default()],
            field: Field::default(),
            instructions: vec![Instruction::default()],
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
            self.snapshots.iter().filter(|(a, _)| **a < step).last()
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
                    ant.travel(direction, self.field.width, self.field.height);
                }
            }
        }
    }

    pub fn is_ant(&self, x: usize, y: usize) -> bool {
        self.ants
            .iter()
            .any(|ant| ant.position.x == x && ant.position.y == y)
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
            orientation: Direction::West,
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
