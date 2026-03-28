mod state;
mod ui;

use crate::ui::App;
use iced::font::Weight;
use iced::{Font, Settings, window};

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

    iced::application(|| App::after(0), App::update, App::view)
        .settings(settings)
        .window(window_settings)
        .subscription(App::subscription)
        .run()
}
