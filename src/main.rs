use iced::border::Radius;
use iced::font::Weight;
use iced::time;
use iced::widget::button::{Style};
use iced::widget::{Column, Row, button};
use iced::{
    Background, Border, Color, Element, Font, Pixels, Settings, Subscription, futures, window,
};
use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Click(usize, usize),
    Tick,
}

pub const CELL_SIZE: f32 = 5.0;
pub const CELL_COUNT: usize = 100;
pub const WINDOW_SIZE: f32 = CELL_SIZE * (CELL_COUNT as f32);
const DEFAULT_BORDER: Border = Border {
    color: Color::from_rgb(0.6, 0.6, 0.6),
    width: 0.1,
    radius: Radius {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    },
};

struct Palette {
    colors: Vec<Color>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: vec![
                Color::from_rgb(0.0, 0.0, 0.0),
                Color::from_rgb(1.0, 1.0, 1.0),
            ],
        }
    }
}

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
            map: [(0, (1, Some(Direction::East))), (1, (0, Some(Direction::West)))]
                .into_iter()
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
struct Ant {
    x: usize,
    y: usize,
    orientation: Direction,
    instruction: Instruction,
}

impl Ant {
    fn travel(&mut self, direction: Direction) {
        self.orientation += direction;

        match self.orientation {
            Direction::North | Direction::NorthEast | Direction::NorthWest => {
                if self.y == 0 {
                    self.y = CELL_COUNT - 1;
                } else {
                    self.y -= 1;
                }
            }
            Direction::South | Direction::SouthEast | Direction::SouthWest => {
                self.y = (self.y + 1) % CELL_COUNT;
            }
            Direction::East | Direction::West => {}
        }
        match self.orientation {
            Direction::West | Direction::NorthWest | Direction::SouthWest => {
                if self.x == 0 {
                    self.x = CELL_COUNT - 1;
                } else {
                    self.x = (self.x - 1) % CELL_COUNT;
                }
            }
            Direction::East | Direction::SouthEast | Direction::NorthEast => {
                self.x = (self.x + 1) % CELL_COUNT;
            }
            Direction::North | Direction::South => {}
        }
    }
}

impl Default for Ant {
    fn default() -> Self {
        Self {
            x: CELL_COUNT / 2,
            y: CELL_COUNT / 2,
            orientation: Direction::North,
            instruction: Instruction::default(),
        }
    }
}

struct State {
    ants: Vec<Ant>,
    field: Field,
    palette: Palette,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ants: vec![Ant::default()],
            field: Field::default(),
            palette: Palette::default(),
        }
    }
}

pub fn main() -> iced::Result {
    let window_settings = window::Settings {
        size: iced::Size {
            width: WINDOW_SIZE,
            height: WINDOW_SIZE,
        },
        // icon: Some(window::icon::from_file("www/favicon.png").unwrap()),
        resizable: false,
        decorations: true,
        ..Default::default()
    };
    let settings: Settings = Settings {
        default_text_size: Pixels(CELL_SIZE / 1.75),
        default_font: Font {
            weight: Weight::Bold,
            ..Default::default()
        },
        ..Default::default()
    };

    iced::application(|| State::after(1000000), State::update, State::view)
        .settings(settings)
        .window(window_settings)
        .subscription(State::subscription)
        .run()
}

impl State {
    pub fn after(steps: usize) -> Self {
        let mut result = Self::default();
        for _ in 0..steps {
            result.step();
        }
        result
    }

    pub fn step(&mut self) {
        for ant in &mut self.ants {
            let next = &ant.instruction.map[&self.field.values[ant.x][ant.y]];
            self.field.values[ant.x][ant.y] = next.0;
            if let Some(direction) = next.1 {
                ant.travel(direction);
            }
        }
    }

    pub fn view(&self) -> Column<'_, Message> {
        let default_button_style: Style = Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: DEFAULT_BORDER,
            ..Style::default()
        };

        Column::with_children((0..CELL_COUNT).flat_map(|y| {
            let mut children = vec![];

            let row = Element::from(Row::with_children((0..CELL_COUNT).flat_map(move |x| {
                let mut children = vec![];

                children.push(Element::from(
                    button("")
                        .on_press(Message::Click(x, y))
                        .width(CELL_SIZE)
                        .height(CELL_SIZE)
                        // .padding([5, 16])
                        .style(move |_, _status| {
                            let color = self.palette.colors
                                [self.field.values[x][y] as usize % self.palette.colors.len()];
                            Style {
                                background: Some(Background::Color(color.clone())),
                                ..default_button_style.clone()
                            }
                        }),
                ));

                children.into_iter()
            })));
            children.push(row);

            children.into_iter()
        }))
        .width(WINDOW_SIZE)
        .height(WINDOW_SIZE)
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Click(x, y) => self.ants.push(Ant {
                x,
                y,
                orientation: Direction::North,
                instruction: Instruction::default(),
            }),
            Message::Tick => {
                self.step();
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::repeat(
            || futures::future::ready(Message::Tick),
            Duration::from_millis(1),
        )
    }
}
