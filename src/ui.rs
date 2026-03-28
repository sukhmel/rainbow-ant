use crate::state::CELL_COUNT;
use crate::state::State;
use iced::border::Radius;
use iced::keyboard::key;
use iced::widget::button::Style;
use iced::widget::pane_grid::Axis;
use iced::widget::text::Wrapping;
use iced::widget::{
    Button, Column, Grid, PaneGrid, Row, TextInput, button, center, center_y, container,
    mouse_area, opaque, operation, pane_grid, responsive, scrollable, stack, text,
};
use iced::{Background, Border, Color, Element, Padding, Subscription, Theme, futures};
use iced::{Event, keyboard, time};
use iced::{Fill, Task, event, exit};
use iced_aw::color_picker;
use std::iter::once;
use std::time::Duration;

pub const CELL_SIZE: f32 = 5.0;
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

pub struct App {
    state: State,
    palette: Palette,
    settings: AppSettings,
    panes: pane_grid::State<Pane>,
    color_selector: Option<usize>,
    ant_color: Color,
    ctrl_pressed: bool,
}

enum Pane {
    Ants,
    Field,
    Palette,
    Settings,
    Instructions,
}

impl Default for App {
    fn default() -> Self {
        let (mut panes, field_pane) = pane_grid::State::new(Pane::Field);
        let (palette_pane, field_vs_right_split) = panes
            .split(Axis::Vertical, field_pane, Pane::Palette)
            .unwrap();
        let (instructions_pane, palette_vs_instruction_split) = panes
            .split(Axis::Horizontal, palette_pane, Pane::Instructions)
            .unwrap();
        let (ants_pane, instructions_vs_ants_split) = panes
            .split(Axis::Horizontal, instructions_pane, Pane::Ants)
            .unwrap();
        let (_settings_pane, ants_vs_settings_split) = panes
            .split(Axis::Horizontal, ants_pane, Pane::Settings)
            .unwrap();
        panes.resize(field_vs_right_split, 0.885);
        panes.resize(palette_vs_instruction_split, 0.2);
        panes.resize(instructions_vs_ants_split, 0.4);
        panes.resize(ants_vs_settings_split, 0.75);
        Self {
            state: State::default(),
            palette: Palette::default(),
            settings: AppSettings::default(),
            panes,
            color_selector: None,
            ant_color: Color::from_rgb(0.0, 1.0, 0.2),
            ctrl_pressed: false,
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

impl App {
    pub fn after(steps: usize) -> Self {
        Self {
            state: State::after(steps),
            ..Default::default()
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        let pane_grid = PaneGrid::new(&self.panes, |_id, pane, _is_maximized| {
            let title = match pane {
                Pane::Ants => "Ants",
                Pane::Field => "Field",
                Pane::Palette => "Palette",
                Pane::Settings => "Settings",
                Pane::Instructions => "Instructions",
            };

            let title_bar = pane_grid::TitleBar::new(text!("{title}").size(24)).padding(10);

            pane_grid::Content::new(
                center_y(responsive(move |_size| match pane {
                    Pane::Ants => self.view_ants(),
                    Pane::Field => self.view_field(),
                    Pane::Palette => self.view_palette(),
                    Pane::Settings => self.view_settings(),
                    Pane::Instructions => self.view_instructions(),
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

    fn view_ants(&self) -> Element<'_, Message> {
        scrollable(
            Row::with_children(self.state.ants.iter().map(|ant| {
                Grid::from_vec(vec![
                    center_y(text!("x")).into(),
                    TextInput::new("x", &ant.start_position.x.to_string())
                        .width(50)
                        .into(),
                    center_y(text!("y")).into(),
                    TextInput::new("y", &ant.start_position.y.to_string())
                        .width(50)
                        .into(),
                    center_y(text!("start")).into(),
                    Button::new(
                        text!("{}", ant.start_position.orientation)
                            .size(16)
                            .center(),
                    )
                    .into(),
                    center_y(text!("moves")).into(),
                    Button::new(text!("{}", ant.instruction).size(16).center()).into(),
                ])
                .columns(2)
                .height(128.0)
                .into()
            }))
            .padding(Padding::ZERO.right(20))
            .spacing(10)
            .wrap(),
        )
        .direction(scrollable::Direction::Vertical(Default::default()))
        .into()
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
                                [self.state.field_at(x, y) % self.palette.colors.len()];
                            // TODO: optimize this
                            let style = if self.state.is_ant(x, y) {
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

    fn view_palette(&self) -> Element<'_, Message> {
        scrollable(
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
                                    theme.extended_palette().background.weak.color.into(),
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
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        scrollable(
            text!(
                "ms per draw: {}\nsteps per draw: {}\nstate: {}\nsteps: {}",
                self.settings.ms_per_tick,
                self.settings.steps_per_tick,
                if self.settings.paused {
                    "paused"
                } else {
                    "running"
                },
                self.state.generation(),
            )
            .size(16),
        )
        .direction(scrollable::Direction::Vertical(Default::default()))
        .into()
    }

    fn view_instructions(&self) -> Element<'_, Message> {
        scrollable(
            text!("TODO: move set goes here")
                .wrapping(Wrapping::Word)
                .size(16),
        )
        .direction(scrollable::Direction::Vertical(Default::default()))
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        if matches!(
            message,
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Control),
                ..
            }))
        ) {
            self.ctrl_pressed = true;
        }
        if matches!(
            message,
            Message::Event(Event::Keyboard(keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(key::Named::Control),
                ..
            }))
        ) {
            self.ctrl_pressed = false;
        }

        match message {
            Message::Click(x, y) => {
                if self.ctrl_pressed {
                    self.state.remove_ant(x, y);
                } else {
                    self.state.add_ant(x, y, 0);
                }
                self.state.recalculate()
            }
            Message::Tick => {
                self.state.step(self.settings.steps_per_tick);
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
                    physical_key: key::Physical::Code(key::Code::KeyR),
                    ..
                }) => {
                    self.state.reset();
                }
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
