use std::collections::HashMap;
use std::ops::{Add, AddAssign};

pub const CELL_COUNT: usize = 256;

#[derive(Debug, Clone)]
struct Field {
    values: Vec<Vec<u8>>,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            values: vec![vec![0; CELL_COUNT]; CELL_COUNT],
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
enum Direction {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
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
struct Instruction {
    /// Map from current palette index to next palette index and direction
    map: HashMap<u8, (u8, Option<Direction>)>,
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
struct Position {
    x: usize,
    y: usize,
    orientation: Direction,
}

#[derive(Clone, Debug)]
struct Ant {
    position: Position,
    start_position: Position,
    instruction: usize,
}

impl Ant {
    fn travel(&mut self, direction: Direction) {
        self.position.orientation += direction;

        match self.position.orientation {
            Direction::North | Direction::NorthEast | Direction::NorthWest => {
                if self.position.y == 0 {
                    self.position.y = CELL_COUNT - 1;
                } else {
                    self.position.y -= 1;
                }
            }
            Direction::South | Direction::SouthEast | Direction::SouthWest => {
                self.position.y = (self.position.y + 1) % CELL_COUNT;
            }
            Direction::East | Direction::West => {}
        }
        match self.position.orientation {
            Direction::West | Direction::NorthWest | Direction::SouthWest => {
                if self.position.x == 0 {
                    self.position.x = CELL_COUNT - 1;
                } else {
                    self.position.x = (self.position.x - 1) % CELL_COUNT;
                }
            }
            Direction::East | Direction::SouthEast | Direction::NorthEast => {
                self.position.x = (self.position.x + 1) % CELL_COUNT;
            }
            Direction::North | Direction::South => {}
        }
    }
}

impl Default for Ant {
    fn default() -> Self {
        let x0 = CELL_COUNT / 2;
        let y0 = CELL_COUNT / 2;
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

pub struct State {
    generation: usize,
    ants: Vec<Ant>,
    field: Field,
    instructions: Vec<Instruction>,
}

impl Default for State {
    fn default() -> Self {
        Self {
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

    pub fn step(&mut self, count: usize) {
        self.generation += count;
        for _ in 0..count {
            for ant in &mut self.ants {
                let next = &self.instructions[ant.instruction].map
                    [&self.field.values[ant.position.x][ant.position.y]];
                self.field.values[ant.position.x][ant.position.y] = next.0;
                if let Some(direction) = next.1 {
                    ant.travel(direction);
                }
            }
        }
    }

    pub fn is_ant(&self, x: usize, y: usize) -> bool {
        self.ants
            .iter()
            .any(|ant| ant.position.x == x && ant.position.y == y)
    }

    pub fn field_at(&self, x: usize, y: usize) -> usize {
        self.field.values[x % CELL_COUNT][y % CELL_COUNT] as usize
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn add_ant(&mut self, x: usize, y: usize, instruction: usize) {
        let x = x % CELL_COUNT;
        let y = y % CELL_COUNT;
        if self.ants.iter().any(|ant| ant.start_position.x == x && ant.start_position.y == y) {
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
        self.field = Field::default();
        for ant in &mut self.ants {
            ant.position = ant.start_position.clone();
        }
        steps
    }

    pub fn recalculate(&mut self) {
        let steps = self.reset();
        self.step(steps);
    }
}
