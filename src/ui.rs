use crate::state;
use crate::state::{Direction, GridType, Instruction, MAX_CELL_COUNT};
use crate::state::{Position, State};
use crate::ui::button::Style;
use iced::alignment::Vertical;
use iced::keyboard::key;
use iced::mouse::Cursor;
use iced::widget::canvas::path::lyon_path::PathEvent;
use iced::widget::canvas::{Geometry, Path, Stroke};
use iced::widget::pane_grid::Axis;
use iced::widget::text::{Alignment, LineHeight};
use iced::widget::{
    Action, Grid, PaneGrid, Row, TextInput, button, canvas, center, center_y, column, container,
    mouse_area, opaque, operation, pane_grid, responsive, row, scrollable, space, stack, text,
};
use iced::{
    Background, Border, Color, Element, Padding, Pixels, Point, Rectangle, Renderer, Size,
    Subscription, Theme, futures, mouse,
};
use iced::{Event, keyboard, time};
use iced::{Fill, Task, event, exit};
use iced_aw::{NumberInput, color_picker};
use std::collections::BTreeMap;
use std::iter::once;
use std::ops::ControlFlow;
use std::thread::{JoinHandle, spawn};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum Message {
    RequestStep(usize),
    RequestWidth(usize),
    RequestHeight(usize),
    ApplyStep,
    ApplySize,
    Blink,
    Click(usize, usize, bool),
    Tick,
    PollUpdate,
    SkipForward,
    Event(Event),
    Resized(pane_grid::ResizeEvent),
    ChooseColor(usize),
    SubmitColor(Color),
    CancelColor,
    AddColor,
    AddInstruction,
    SetUpAnt {
        ant_index: usize,
        start_position: Position,
        instruction: usize,
    },
    RotateAnt(usize),
    RotateInstructionDirection {
        instruction_index: usize,
        palette_index: u8,
    },
    RotateInstructionPalette {
        instruction_index: usize,
        palette_index: u8,
    },
    Pause,
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
    step_requested: Option<usize>,
    size_requested: Option<(usize, usize)>,
    scale: f32,
    last_tick: Option<Instant>,
    last_tick_duration: Option<Duration>,
    update_join_handle: Option<JoinHandle<(State, Duration, Instant)>>,
}

impl canvas::Program<Message> for App {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Action<Message>> {
        let Event::Mouse(mouse::Event::ButtonReleased(button)) = event else {
            return None;
        };

        let (width, height) = self.state.field_size();
        let Some(point) = cursor.position_in(bounds) else {
            return None;
        };

        if let Some((x, y)) = self.point_to_logical_coordinates(point)
            && x <= width
            && y <= height
        {
            if *button == mouse::Button::Left {
                return Some(Action::publish(Message::Click(x, y, true).into()));
            } else if *button == mouse::Button::Right {
                return Some(Action::publish(Message::Click(x, y, false).into()));
            }
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let scale = self.scale;
        let (width, height) = self.state.field_size();
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        for x in 0..width {
            for y in 0..height {
                let color =
                    self.palette.colors[self.state.field_at(x, y) % self.palette.colors.len()];
                match self.state.grid_type {
                    GridType::Square | GridType::SquareDiagonal => {
                        frame.fill_rectangle(
                            Point::new(x as f32 * scale, y as f32 * scale),
                            Size::new(scale, scale),
                            color,
                        );
                    }
                    GridType::Hexagonal => frame.fill(&hexagon_path(scale, x, y, 0.0), color),
                    GridType::Triangular => frame.fill(&triangle_path(scale, x, y, 0.0), color),
                }
            }
        }

        if let Some(point) = cursor.position_in(bounds) {
            if let Some((x, y)) = self.point_to_logical_coordinates(point)
                && x <= width
                && y <= height
            {
                match self.state.grid_type {
                    GridType::Triangular | GridType::Hexagonal => {
                        let target = if self.state.grid_type == GridType::Triangular {
                            triangle_path(scale, x, y, 0.0)
                        } else {
                            hexagon_path(scale, x, y, 0.0)
                        };
                        frame.stroke(
                            &target,
                            Stroke {
                                style: canvas::Style::Solid(Color::from_rgb(0.9, 0.9, 0.3)),
                                width: 2.0,
                                line_cap: Default::default(),
                                line_join: Default::default(),
                                line_dash: Default::default(),
                            },
                        );
                    }
                    GridType::Square | GridType::SquareDiagonal => frame.stroke_rectangle(
                        Point::new(x as f32 * scale + 1.0, y as f32 * scale + 1.0),
                        Size::new(scale - 2.0, scale - 2.0),
                        Stroke {
                            style: canvas::Style::Solid(Color::from_rgb(0.9, 0.9, 0.3)),
                            width: 2.0,
                            line_cap: Default::default(),
                            line_join: Default::default(),
                            line_dash: Default::default(),
                        },
                    ),
                }
                frame.fill_text(canvas::Text {
                    content: format!("({x:.0}, {y:.0})"),
                    position: point,
                    max_width: 200.0,
                    color: Color::WHITE,
                    size: Pixels(14.0),
                    line_height: LineHeight::Absolute(Pixels(2.0)),
                    font: Default::default(),
                    align_x: Alignment::Left,
                    align_y: Vertical::Bottom,
                    shaping: Default::default(),
                });
            }
        }

        for (x, y) in self.state.ants() {
            match self.state.grid_type {
                GridType::Square | GridType::SquareDiagonal => frame.stroke_rectangle(
                    Point::new(x as f32 * scale + 1.0, y as f32 * scale + 1.0),
                    Size::new(scale - 2.0, scale - 2.0),
                    Stroke {
                        style: canvas::Style::Solid(self.ant_color),
                        width: 2.0,
                        line_cap: Default::default(),
                        line_join: Default::default(),
                        line_dash: Default::default(),
                    },
                ),
                GridType::Triangular | GridType::Hexagonal => {
                    let target = if self.state.grid_type == GridType::Triangular {
                        triangle_path(scale, x, y, 1.0)
                    } else {
                        hexagon_path(scale, x, y, 1.0)
                    };
                    frame.stroke(
                        &target,
                        Stroke {
                            style: canvas::Style::Solid(self.ant_color),
                            width: 2.0,
                            line_cap: Default::default(),
                            line_join: Default::default(),
                            line_dash: Default::default(),
                        },
                    )
                }
            }
        }

        vec![frame.into_geometry()]
    }
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
            step_requested: None,
            size_requested: None,
            last_tick: None,
            last_tick_duration: None,
            scale: 20.0,
            update_join_handle: None,
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
        responsive(move |size| {
            scrollable(
                Row::with_children(self.state.ants.iter().enumerate().map(|(ant_index, ant)| {
                    let instruction = ant.instruction;
                    let start_position = ant.start_position.clone();
                    let start_position_for_x = ant.start_position.clone();
                    let start_position_for_y = ant.start_position.clone();
                    Grid::from_vec(vec![
                        center_y(text!("x")).into(),
                        NumberInput::new(
                            &ant.start_position.x,
                            0..self.state.field_size().0,
                            move |x| Message::SetUpAnt {
                                ant_index,
                                start_position: Position {
                                    x,
                                    ..start_position_for_x
                                },
                                instruction,
                            },
                        )
                        .into(),
                        center_y(text!("y")).into(),
                        NumberInput::new(
                            &ant.start_position.y,
                            0..self.state.field_size().1,
                            move |y| Message::SetUpAnt {
                                ant_index,
                                start_position: Position {
                                    y,
                                    ..start_position_for_y
                                },
                                instruction,
                            },
                        )
                        .into(),
                        center_y(text!("start")).into(),
                        button(
                            text!(
                                "{}",
                                state::effective_direction(
                                    self.state.grid_type,
                                    ant.start_position.orientation
                                )
                            )
                            .size(16)
                            .center(),
                        )
                        .on_press(Message::RotateAnt(ant_index))
                        .into(),
                        center_y(text!("rules")).into(),
                        if self.state.instructions.len() > 1 {
                            NumberInput::new(
                                &ant.instruction,
                                0..self.state.instructions.len(),
                                move |instruction| Message::SetUpAnt {
                                    ant_index,
                                    start_position: start_position.clone(),
                                    instruction,
                                },
                            )
                            .into()
                        } else {
                            TextInput::new("x", "0").into()
                        },
                    ])
                    .columns(2)
                    .height(128.0)
                    .into()
                }))
                .padding(Padding::ZERO.right(20))
                .spacing(10)
                .wrap(),
            )
            .height(size.height)
            .width(size.width)
            .direction(scrollable::Direction::Vertical(Default::default()))
            .into()
        })
        .into()
    }

    fn view_field(&self) -> Element<'_, Message> {
        // A scrollable provides correct boundaries for the canvas but fails to allow for scrolling;
        // it should be handled manually with canvas, it seems.
        responsive(move |size| {
            scrollable(
                mouse_area(
                    canvas(self)
                        .width(self.scale * self.state.field_size().0 as f32)
                        .height(self.scale * self.state.field_size().1 as f32),
                )
                .interaction(mouse::Interaction::Crosshair),
            )
            .width(size.width)
            .height(size.height)
            .direction(scrollable::Direction::Both {
                vertical: Default::default(),
                horizontal: Default::default(),
            })
            .into()
        })
        .into()
    }

    fn pause(&mut self) {
        self.settings.paused = !self.settings.paused;
        if !self.settings.paused {
            self.step_requested = None;
        } else {
            self.last_tick = None;
        }
    }

    fn poll_update(&mut self) -> ControlFlow<(), ()> {
        if let Some(join_handle) = self.update_join_handle.take() {
            if !join_handle.is_finished() {
                self.update_join_handle = Some(join_handle);
                return ControlFlow::Break(());
            } else {
                let (state, duration, tick) = join_handle.join().unwrap();
                self.state = state;
                self.last_tick_duration = Some(duration);
                self.last_tick = Some(tick);
            }
        }
        ControlFlow::Continue(())
    }

    fn defer_update(&mut self, f: impl FnOnce(&mut State) + Send + 'static) {
        let mut state = self.state.clone();

        self.update_join_handle = Some(spawn(move || {
            let start = Instant::now();
            f(&mut state);
            (state, start.elapsed(), Instant::now())
        }));
    }

    fn view_palette(&self) -> Element<'_, Message> {
        responsive(move |size| {
            scrollable(
                Row::with_children(
                    self.palette
                        .colors
                        .iter()
                        .enumerate()
                        .map(|(i, color)| {
                            button(" ")
                                .style(|_theme, _status| Style {
                                    background: Some(Background::Color(color.clone())),
                                    ..Default::default()
                                })
                                .width(32.0)
                                .height(32.0)
                                .on_press(Message::ChooseColor(i))
                                .into()
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
            .height(size.height)
            .width(size.width)
            .direction(scrollable::Direction::Vertical(Default::default()))
            .into()
        })
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        let generation = self
            .step_requested
            .unwrap_or_else(|| self.state.generation());

        let (width, height) = self
            .size_requested
            .unwrap_or_else(|| self.state.field_size());

        responsive(move |size| {
            scrollable(
                column([
                    text!("grid: {:?}", self.state.grid_type,).size(16).into(),
                    text!(
                        "draw ms: {} dps: {:.2}",
                        self.last_tick_duration
                            .map(|t| t.as_millis())
                            .unwrap_or_default(),
                        self.last_tick_duration
                            .map(|t| 1.0 / t.as_secs_f64())
                            .unwrap_or_default(),
                    )
                    .size(16)
                    .into(),
                    text!(
                        "ms per draw: {}\nsteps per draw: {}",
                        self.settings.ms_per_tick,
                        self.settings.steps_per_tick,
                    )
                    .size(16)
                    .into(),
                    row([
                        center_y(text!("state").size(16)).into(),
                        space().width(10).into(),
                        button(text!("{}", if self.settings.paused { "⏸" } else { "▶" }))
                            .on_press(Message::Pause)
                            .into(),
                        space().width(10).into(),
                        button(text!("{}", "⏩︎")).on_press(Message::Tick).into(),
                        space().width(10).into(),
                        button(text!("{}", "⏭︎"))
                            .on_press(Message::SkipForward)
                            .into(),
                    ])
                    .into(),
                    row([
                        center_y(text!("steps").size(16)).into(),
                        space().width(10).into(),
                        NumberInput::new(&generation, 0.., Message::RequestStep).into(),
                        button(text!("go")).on_press(Message::ApplyStep).into(),
                    ])
                    .into(),
                    row([
                        center_y(text!("size").size(16)).into(),
                        space().width(10).into(),
                        NumberInput::new(&width, 1..MAX_CELL_COUNT, Message::RequestWidth)
                            .width(70.0)
                            .into(),
                        center_y(text!("×").size(16)).into(),
                        NumberInput::new(&height, 1..MAX_CELL_COUNT, Message::RequestHeight)
                            .width(70.0)
                            .into(),
                        button(text!("go")).on_press(Message::ApplySize).into(),
                    ])
                    .into(),
                ])
                .padding(Padding::ZERO.bottom(20)),
            )
            .height(size.height)
            .width(size.width)
            .direction(scrollable::Direction::Both {
                vertical: Default::default(),
                horizontal: Default::default(),
            })
            .into()
        })
        .into()
    }

    fn view_instructions(&self) -> Element<'_, Message> {
        responsive(move |size| {
            scrollable(
                Row::with_children(
                    self.state
                        .instructions
                        .iter()
                        .enumerate()
                        .flat_map(|(index, instruction)| {
                            let children = [
                                text!("#").size(16).center().into(),
                                text!("{}", index).size(16).center().into(),
                                text!("").size(16).center().into(),
                                text!("").size(16).center().into(),
                            ]
                            .into_iter()
                            .chain(instruction.map.iter().flat_map(|(from, (to, direction))| {
                                [
                                    button(" ")
                                        .style(|_theme, _status| Style {
                                            background: Some(Background::Color(
                                                self.palette.colors[*from as usize].clone(),
                                            )),
                                            ..Default::default()
                                        })
                                        .into(),
                                    text!(
                                        "{}",
                                        if instruction.map.iter().any(|(_, (to, _))| to == from) {
                                            ">"
                                        } else {
                                            "!"
                                        }
                                    )
                                    .size(16)
                                    .center()
                                    .into(),
                                    button(" ")
                                        .style(|_theme, _status| Style {
                                            background: Some(Background::Color(
                                                self.palette.colors[*to as usize].clone(),
                                            )),
                                            ..Default::default()
                                        })
                                        .on_press(Message::RotateInstructionPalette {
                                            instruction_index: index,
                                            palette_index: *from,
                                        })
                                        .into(),
                                    button(
                                        text!(
                                            "{}",
                                            direction
                                                .map(|d| state::effective_direction(
                                                    self.state.grid_type,
                                                    d
                                                )
                                                .to_string())
                                                .unwrap_or(String::from("⊙"))
                                        )
                                        .size(16)
                                        .center(),
                                    )
                                    .on_press(Message::RotateInstructionDirection {
                                        instruction_index: index,
                                        palette_index: *from,
                                    })
                                    .into(),
                                ]
                                .into_iter()
                            }));
                            [
                                Grid::with_children(children)
                                    .columns(4)
                                    .spacing(5)
                                    .width(120.0)
                                    .into(),
                                iced::widget::rule::vertical(3.).into(),
                            ]
                            .into_iter()
                        })
                        .chain(once(
                            button(text!("+").size(28).center())
                                .style(|theme: &Theme, _status| Style {
                                    background: Some(Background::Color(
                                        theme.extended_palette().background.weak.color.into(),
                                    )),
                                    ..Default::default()
                                })
                                .width(20.0)
                                .height(20.0)
                                .on_press(Message::AddInstruction)
                                .into(),
                        )),
                )
                .padding(Padding::ZERO.right(20).bottom(20))
                .spacing(10),
            )
            .height(size.height)
            .width(size.width)
            .direction(scrollable::Direction::Both {
                vertical: Default::default(),
                horizontal: Default::default(),
            })
            .into()
        })
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
            Message::Click(x, y, add) => {
                if add {
                    self.state.add_ant(x, y, 0);
                } else {
                    self.state.remove_ant(x, y);
                }
                self.defer_update(|state| state.recalculate());
            }
            Message::PollUpdate => {
                let _ = self.poll_update();
            }
            Message::Tick => {
                if self.poll_update().is_break() {
                    return Task::none();
                }

                let steps = self.settings.steps_per_tick;
                self.defer_update(move |state| {
                    state.step(steps);
                });
            }
            Message::Blink => {
                self.ant_color = if self.ant_color == Color::from_rgb(0.0, 1.0, 0.2) {
                    Color::from_rgb(0.0, 0.2, 1.0)
                } else {
                    Color::from_rgb(0.0, 1.0, 0.2)
                };
            }
            Message::Pause => self.pause(),
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
                    self.settings.paused = true;
                    self.update_join_handle.take();
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    physical_key: key::Physical::Code(key::Code::KeyG),
                    ..
                }) => {
                    self.state.grid_type = state::next_grid_type(self.state.grid_type);
                    self.defer_update(|state| state.recalculate());
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    physical_key: key::Physical::Code(key::Code::Minus),
                    ..
                }) => {
                    if self.scale > 1.0 {
                        self.scale -= 1.0;
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    physical_key: key::Physical::Code(key::Code::Equal),
                    ..
                }) => {
                    if self.scale < 40.0 {
                        self.scale += 1.0;
                    }
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
                }) => self.pause(),
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
            Message::AddColor => {
                self.palette.colors.push(Color::from_rgb(0.5, 0.5, 0.5));
                for instruction in &mut self.state.instructions {
                    instruction
                        .map
                        .insert(self.palette.colors.len() as u8 - 1, (0, None));
                }
            }
            Message::AddInstruction => {
                let mut map: BTreeMap<u8, (u8, Option<Direction>)> = (0..self.palette.colors.len()
                    as u8)
                    .map(|i| (i, (0, None)))
                    .collect();
                map.insert(0, (1, Some(Direction::East)));
                map.insert(1, (0, Some(Direction::West)));

                self.state.instructions.push(Instruction { map });
            }
            Message::SetUpAnt {
                ant_index,
                start_position: position,
                instruction,
            } => {
                if let Some(ant) = self.state.ants.get_mut(ant_index) {
                    ant.start_position = position;
                    ant.instruction = instruction;
                    self.defer_update(|state| state.recalculate());
                }
            }
            Message::RotateAnt(index) => {
                if let Some(ant) = self.state.ants.get_mut(index) {
                    if self.ctrl_pressed {
                        ant.start_position.orientation =
                            ant.start_position.orientation + Direction::NorthWest;
                    } else {
                        ant.start_position.orientation =
                            ant.start_position.orientation + Direction::NorthEast;
                    }
                    self.defer_update(|state| state.recalculate());
                }
            }
            Message::RotateInstructionDirection {
                instruction_index,
                palette_index,
            } => {
                if let Some(instruction) = self.state.instructions.get_mut(instruction_index) {
                    if let Some((_, target)) = instruction.map.get_mut(&palette_index) {
                        if self.ctrl_pressed {
                            *target = state::prev_direction(self.state.grid_type, *target);
                        } else {
                            *target = state::next_direction(self.state.grid_type, *target);
                        }
                    }

                    self.defer_update(|state| state.recalculate());
                }
            }
            Message::RotateInstructionPalette {
                instruction_index,
                palette_index,
            } => {
                if let Some(instruction) = self.state.instructions.get_mut(instruction_index) {
                    if let Some((target, _)) = instruction.map.get_mut(&palette_index) {
                        if self.ctrl_pressed {
                            if *target == 0 {
                                *target = self.palette.colors.len() as u8 - 1;
                            } else {
                                *target -= 1;
                            }
                        } else {
                            *target = (*target + 1) % self.palette.colors.len() as u8;
                        }
                    }

                    self.defer_update(|state| state.recalculate());
                }
            }
            Message::RequestStep(step) => {
                self.step_requested = Some(step);
            }
            Message::ApplyStep => {
                if let Some(step) = self.step_requested.take() {
                    self.state.go_to_step(step);
                }
            }
            Message::RequestWidth(width) => {
                self.size_requested
                    .get_or_insert_with(|| self.state.field_size())
                    .0 = width;
            }
            Message::RequestHeight(height) => {
                self.size_requested
                    .get_or_insert_with(|| self.state.field_size())
                    .1 = height;
            }
            Message::ApplySize => {
                if let Some((width, height)) = self.size_requested.take() {
                    self.state.set_width(width);
                    self.state.set_height(height);
                    self.defer_update(|state| state.recalculate());
                }
            }
            Message::SkipForward => {
                self.state
                    .go_to_step((self.state.generation() / 100_000 + 1) * 100_000);
                self.defer_update(|state| state.recalculate());
                self.step_requested = None;
            }
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

        let running = !self.settings.paused && self.settings.steps_per_tick > 0;
        let polling = self.update_join_handle.is_some();
        if running {
            subscriptions.push(time::repeat(
                || futures::future::ready(Message::Tick),
                Duration::from_millis(self.settings.ms_per_tick as u64),
            ));
        } else if polling {
            subscriptions.push(time::repeat(
                || futures::future::ready(Message::PollUpdate),
                Duration::from_secs(1),
            ));
        }

        Subscription::batch(subscriptions)
    }

    fn point_to_logical_coordinates(&self, point: Point) -> Option<(usize, usize)> {
        let scale = self.scale;
        match self.state.grid_type {
            GridType::Square | GridType::SquareDiagonal => {
                Some(((point.x / scale) as usize, (point.y / scale) as usize))
            }
            // TODO
            GridType::Hexagonal => {
                // TODO: the discrepancy increases with absolute value, and also it needs to check PIP
                let y = (2.0 * point.y / f32::sqrt(3.0) / scale) as usize;
                let x = (4.0 * point.x / 3.0 / scale) as usize;
                Some((x, y))
            }
            GridType::Triangular => {
                let y = (point.y * 2.0 / scale / f32::sqrt(3.0)) as usize;
                let x = (point.x / scale) as usize * 2;
                let x_left = x.saturating_sub(2);
                let x_right = x;
                let x_center = x.saturating_sub(1);
                let triangle_left = self.triangle_at(x_left, y);
                let triangle_right = self.triangle_at(x_right, y);
                let triangle_center = self.triangle_at(x_center, y);

                if is_point_in_triangle(point, &triangle_left) {
                    Some((x_left, y))
                } else if is_point_in_triangle(point, &triangle_center) {
                    Some((x_center, y))
                } else if is_point_in_triangle(point, &triangle_right) {
                    Some((x_right, y))
                } else {
                    None
                }
            }
        }
    }

    fn triangle_at(&self, x: usize, y: usize) -> Vec<(Point, Point)> {
        triangle_path(self.scale, x, y, 0.0)
            .raw()
            .iter()
            .filter_map(|p| match p {
                PathEvent::Line { from, to } => {
                    Some((Point::new(from.x, from.y), Point::new(to.x, to.y)))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
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

fn hexagon_path(scale: f32, x: usize, y: usize, padding: f32) -> Path {
    Path::new(|builder| {
        let shift = scale * f32::sqrt(3.0) / 4.0 * if x % 2 == 0 { 1.0 } else { 2.0 };
        let x = x as f32 * 3.0 * scale / 4.0;
        let y = y as f32 * f32::sqrt(3.0) * scale / 2.0 + shift;
        builder.move_to(Point::new(x + padding, y));
        builder.line_to(Point::new(
            x + scale / 4.0 + padding,
            y - f32::sqrt(3.0) * scale / 4.0 + padding,
        ));
        builder.line_to(Point::new(
            x + 3.0 * scale / 4.0 - padding,
            y - f32::sqrt(3.0) * scale / 4.0 + padding,
        ));
        builder.line_to(Point::new(x + scale - padding, y));
        builder.line_to(Point::new(
            x + 3.0 * scale / 4.0 - padding,
            y + f32::sqrt(3.0) * scale / 4.0 - padding,
        ));
        builder.line_to(Point::new(
            x + scale / 4.0 + padding,
            y + f32::sqrt(3.0) * scale / 4.0 - padding,
        ));
        builder.close();
    })
}

fn triangle_path(scale: f32, x: usize, y: usize, padding: f32) -> Path {
    Path::new(|builder| {
        let inverted = (x + y) % 2 == 0;
        let shift = if y % 2 == 0 {
            0.0
        } else {
            0.5 * scale * if (x % 2) == 0 { 1.0 } else { -1.0 }
        };
        let y = f32::sqrt(3.0) / 2.0 * (y as f32) * scale;
        if !inverted {
            let x = shift + (((x + 1) / 2) as f32) * scale;
            builder.move_to(Point::new(x + padding, y + padding));
            builder.line_to(Point::new(x + scale - padding, y + padding));
            builder.line_to(Point::new(
                x + scale / 2.0,
                y + (scale / 2.0) * f32::sqrt(3.0) - padding,
            ));
        } else {
            let x = shift + (((x + 1) / 2 + 1) as f32) * scale;
            builder.move_to(Point::new(x, y + padding));
            builder.line_to(Point::new(
                x + scale / 2.0 - padding,
                y + (scale / 2.0) * f32::sqrt(3.0) - padding,
            ));
            builder.line_to(Point::new(
                x - scale / 2.0 + padding,
                y + (scale / 2.0) * f32::sqrt(3.0) - padding,
            ));
        }
        builder.close();
    })
}

fn is_point_in_triangle(point: Point, triangle: &[(Point, Point)]) -> bool {
    let [(a, b), (_b, c)] = triangle else {
        return false;
    };
    let v0 = *c - *a;
    let v1 = *b - *a;
    let v2 = point - *a;

    let dot00 = v0.x * v0.x + v0.y * v0.y;
    let dot01 = v0.x * v1.x + v0.y * v1.y;
    let dot02 = v0.x * v2.x + v0.y * v2.y;
    let dot11 = v1.x * v1.x + v1.y * v1.y;
    let dot12 = v1.x * v2.x + v1.y * v2.y;

    let denom = dot00 * dot11 - dot01 * dot01;
    let u = (dot11 * dot02 - dot01 * dot12) / denom;
    let v = (dot00 * dot12 - dot01 * dot02) / denom;

    u >= 0.0 && v >= 0.0 && u + v < 1.0
}
