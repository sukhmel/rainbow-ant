use iced::border::Radius;
use iced::font::Weight;
use iced::keyboard::key;
use iced::widget::button::Style;
use iced::widget::pane_grid::Axis;
use iced::widget::{
    Column, PaneGrid, Row, button, center, center_y, container, mouse_area, opaque,
    operation, pane_grid, responsive, scrollable, stack, text,
};
use iced::{Background, Border, Color, Element, Font, Settings, Subscription, Theme, futures, window, Padding};
use iced::{Event, keyboard, time};
use iced::{Fill, Task, event, exit};
use iced_aw::color_picker;
use std::collections::HashMap;
use std::iter::once;
use std::ops::{Add, AddAssign};
use std::time::Duration;

pub const CELL_SIZE: f32 = 5.0;
pub const CELL_COUNT: usize = 256;
pub const FIELD_SIZE: f32 = CELL_SIZE * (CELL_COUNT as f32);
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

pub fn main() -> iced::Result {
    let window_settings = window::Settings {
        size: iced::Size {
            width: 1500.0,
            height: 1372.0,
        },
        // icon: Some(window::icon::from_file("www/favicon.png").unwrap()),
        resizable: true,
        decorations: true,
        ..Default::default()
    };
    let settings: Settings = Settings {
        default_font: Font {
            weight: Weight::Bold,
            ..Default::default()
        },
        ..Default::default()
    };

    iced::application(|| State::after(0), State::update, State::view)
        .settings(settings)
        .window(window_settings)
        .subscription(State::subscription)
        .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    Blink,
    Click(usize, usize),
    Tick,
    Event(Event),
    Resized(pane_grid::ResizeEvent),
    ChooseColor(usize),
    SubmitColor(Color),
    CancelColor,
    AddColor,
}

struct Palette {
    colors: Vec<Color>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: vec![
                Color::from_rgb(0.0, 0.0, 0.0),
                Color::from_rgb(0.0, 0.0, 1.0),
                Color::from_rgb(0.0, 1.0, 0.0),
                Color::from_rgb(1.0, 0.0, 0.0),
                Color::from_rgb(1.0, 1.0, 0.0),
                Color::from_rgb(1.0, 0.0, 1.0),
                Color::from_rgb(0.0, 1.0, 1.0),
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
            orientation: Direction::West,
            instruction: Instruction::default(),
        }
    }
}

struct State {
    generation: usize,
    ants: Vec<Ant>,
    field: Field,
    palette: Palette,
    settings: AppSettings,
    panes: pane_grid::State<Pane>,
    color_selector: Option<usize>,
    ant_color: Color,
}

enum Pane {
    Field,
    Palette,
    Settings,
    Instructions,
}

impl Default for State {
    fn default() -> Self {
        let (mut panes, field_pane) = pane_grid::State::new(Pane::Field);
        let (palette_pane, field_split) = panes
            .split(Axis::Vertical, field_pane, Pane::Palette)
            .unwrap();
        let (instructions_pane, palette_split) = panes
            .split(Axis::Horizontal, palette_pane, Pane::Instructions)
            .unwrap();
        let (_settings_pane, instructions_split) = panes
            .split(Axis::Horizontal, instructions_pane, Pane::Settings)
            .unwrap();
        panes.resize(field_split, 0.885);
        panes.resize(palette_split, 0.2);
        panes.resize(instructions_split, 0.8);
        Self {
            generation: 0,
            ants: vec![Ant::default()],
            field: Field::default(),
            palette: Palette::default(),
            settings: AppSettings::default(),
            panes,
            color_selector: None,
            ant_color: Color::from_rgb(0.0, 1.0, 0.2),
        }
    }
}

#[derive(Debug, Clone)]
struct AppSettings {
    paused: bool,
    steps_per_tick: usize,
    ms_per_tick: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            paused: true,
            steps_per_tick: 10,
            ms_per_tick: 10,
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
                let next = &ant.instruction.map[&self.field.values[ant.x][ant.y]];
                self.field.values[ant.x][ant.y] = next.0;
                if let Some(direction) = next.1 {
                    ant.travel(direction);
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let pane_grid = PaneGrid::new(&self.panes, |_id, pane, _is_maximized| {
            let title = match pane {
                Pane::Field => "Field",
                Pane::Palette => "Palette",
                Pane::Settings => "Settings",
                Pane::Instructions => "Instructions",
            };

            let title_bar = pane_grid::TitleBar::new(text!("{title}").size(24)).padding(10);

            pane_grid::Content::new(
                center_y(responsive(move |_size| match pane {
                    Pane::Field => self.view_field(),
                    Pane::Palette => scrollable(
                        Row::with_children(
                            self.palette
                                .colors
                                .iter()
                                .enumerate()
                                .map(|(i, color)| {
                                    let color_button = button(" ")
                                        .style(|_theme, _status| Style {
                                            background: Some(Background::Color(color.clone())),
                                            ..Default::default()
                                        })
                                        .width(32.0)
                                        .height(32.0)
                                        .on_press(Message::ChooseColor(i));
                                    color_button.into()
                                })
                                .chain(once(
                                    button(text!("+").size(28).center())
                                        .style(|theme: &Theme, _status| Style {
                                            background: Some(Background::Color(
                                                theme
                                                    .extended_palette()
                                                    .background
                                                    .weak
                                                    .color
                                                    .into(),
                                            )),
                                            ..Default::default()
                                        })
                                        .width(32.0)
                                        .height(32.0)
                                        .on_press(Message::AddColor)
                                        .into(),
                                )),
                        )

                        .spacing(10)
                        .padding(Padding::ZERO.right(20))
                        .wrap(),
                    )
                    .direction(scrollable::Direction::Vertical(Default::default()))
                    .into(),
                    Pane::Settings => text!(
                        "ms per draw: {}\nsteps per draw: {}\nstate: {}",
                        self.settings.ms_per_tick,
                        self.settings.steps_per_tick,
                        if self.settings.paused {
                            "paused"
                        } else {
                            "running"
                        }
                    )
                    .size(16)
                    .into(),
                    Pane::Instructions => text!("Instructions").size(16).into(),
                }))
                .padding(10),
            )
            .title_bar(title_bar)
            .style(|theme| {
                let palette = theme.extended_palette();

                container::Style {
                    background: Some(palette.background.weak.color.into()),
                    border: Border {
                        width: 2.0,
                        color: palette.secondary.strong.color,
                        ..Border::default()
                    },
                    ..Default::default()
                }
            })
        })
        .width(Fill)
        .height(Fill)
        .on_resize(10, Message::Resized)
        .spacing(5);

        if let Some(color_selector) = self.color_selector {
            let picker = color_picker(
                true,
                self.palette.colors[color_selector],
                container(text!("Choose a color")).width(100).height(100),
                Message::CancelColor,
                Message::SubmitColor,
            );
            modal(pane_grid, picker, Message::CancelColor)
        } else {
            container(pane_grid).padding(5).into()
        }
    }

    fn view_field(&self) -> Element<'_, Message> {
        let default_button_style: Style = Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: DEFAULT_BORDER,
            ..Style::default()
        };
        let ant_button_style: Style = Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: self.ant_color,
                width: 1.5,
                radius: Default::default(),
            },
            ..Style::default()
        };

        let field = Column::with_children((0..CELL_COUNT).flat_map(|y| {
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
                            let style = if self.ants.iter().any(|ant| ant.x == x && ant.y == y) {
                                ant_button_style
                            } else {
                                default_button_style
                            };
                            Style {
                                background: Some(Background::Color(color.clone())),
                                ..style
                            }
                        }),
                ));

                children.into_iter()
            })));
            children.push(row);

            children.into_iter()
        }))
        .width(FIELD_SIZE)
        .height(FIELD_SIZE);

        center_y(scrollable(field).direction(scrollable::Direction::Both {
            vertical: Default::default(),
            horizontal: Default::default(),
        }))
        .padding(5)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Click(x, y) => {
                if let Some((i, _)) = self
                    .ants
                    .iter()
                    .enumerate()
                    .find(|(_, ant)| ant.x == x && ant.y == y)
                {
                    self.ants.remove(i);
                } else {
                    self.ants.push(Ant {
                        x,
                        y,
                        orientation: Direction::North,
                        instruction: Instruction::default(),
                    });
                }
            }
            Message::Tick => {
                self.step(self.settings.steps_per_tick);
            }
            Message::Blink => {
                self.ant_color = if self.ant_color == Color::from_rgb(0.0, 1.0, 0.2) {
                    Color::from_rgb(0.0, 0.2, 1.0)
                } else {
                    Color::from_rgb(0.0, 1.0, 0.2)
                };
            }
            Message::Event(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Escape),
                    ..
                })
                | Event::Keyboard(keyboard::Event::KeyPressed {
                    physical_key: key::Physical::Code(key::Code::KeyQ),
                    ..
                }) => return exit(),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::ArrowUp),
                    modifiers,
                    ..
                }) => {
                    let target = if modifiers.control() {
                        &mut self.settings.ms_per_tick
                    } else {
                        &mut self.settings.steps_per_tick
                    };
                    if *target > 0 {
                        *target *= 2;
                    } else {
                        *target = 1;
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::ArrowDown),
                    modifiers,
                    ..
                }) => {
                    let target = if modifiers.control() {
                        &mut self.settings.ms_per_tick
                    } else {
                        &mut self.settings.steps_per_tick
                    };
                    *target /= 2;
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::ArrowLeft),
                    modifiers,
                    ..
                }) => {
                    let target = if modifiers.control() {
                        &mut self.settings.ms_per_tick
                    } else {
                        &mut self.settings.steps_per_tick
                    };
                    if *target > 10 {
                        *target -= 10;
                    } else {
                        *target = 1;
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::ArrowRight),
                    modifiers,
                    ..
                }) => {
                    let target = if modifiers.control() {
                        &mut self.settings.ms_per_tick
                    } else {
                        &mut self.settings.steps_per_tick
                    };
                    *target += 10;
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Space),
                    ..
                }) => {
                    self.settings.paused = !self.settings.paused;
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Tab),
                    modifiers,
                    ..
                }) => {
                    return if modifiers.shift() {
                        operation::focus_previous()
                    } else {
                        operation::focus_next()
                    };
                }
                _ => {}
            },
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
            }
            Message::ChooseColor(index) => {
                self.color_selector = Some(index);
                return operation::focus_next();
            }
            Message::SubmitColor(color) => {
                if let Some(index) = self.color_selector.take() {
                    self.palette.colors[index] = color;
                }
            }
            Message::CancelColor => {
                self.color_selector = None;
            }
            Message::AddColor => self.palette.colors.push(Color::from_rgb(0.5, 0.5, 0.5)),
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            event::listen().map(Message::Event),
            time::repeat(
                || futures::future::ready(Message::Blink),
                Duration::from_millis(200),
            ),
        ];
        if !self.settings.paused && self.settings.steps_per_tick > 0 {
            subscriptions.push(time::repeat(
                || futures::future::ready(Message::Tick),
                Duration::from_millis(self.settings.ms_per_tick as u64),
            ));
        }
        Subscription::batch(subscriptions)
    }
}

fn modal<'a, Message>(
    parent: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    cancel: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        parent.into(),
        opaque(mouse_area(center(text!(" "))).on_press(cancel)),
        center(opaque(content)).style(|_theme| {
            container::Style {
                background: Some(
                    Color {
                        a: 0.8,
                        ..Color::BLACK
                    }
                    .into(),
                ),
                ..container::Style::default()
            }
        })
    ]
    .into()
}
