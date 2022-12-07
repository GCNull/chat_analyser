#![warn(clippy::all)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::env;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Error;
use chrono::Local;
use env_logger::{Builder as env_builder, fmt::Color, WriteStyle};
use iced::{Application, Command, Element, Settings, Theme};
use iced::executor;
use iced::widget::{Column, Container, Row, Text};
use iced_native::{Event, Subscription, window};
use log::{Level, LevelFilter};
use tokio::runtime::Builder;

use structs_enums::*;

mod config;
mod ui;
mod socket;
mod parse_twitch_data;
mod structs_enums;

type Res<T> = Result<T, Error>;

#[cfg(not(target_family = "unix"))]
fn main() -> Res<()> {
    panic!("This app is built for Linux devices only")
}

#[cfg(target_family = "unix")]
fn main() {
    if cfg!(debug_assertions) {
        log::warn!("Running in debug mode! Run in release mode for optimisations!")
    }
    env::set_var("RUST_BACKTRACE", "full");
    env_builder::new().write_style(WriteStyle::Always).format(move |buf, record| {
        let mut style = buf.style();
        let mut ts_style = buf.style();
        let ts = ts_style.set_color(Color::Rgb(64, 205, 170)).value(Local::now().format("%F %H:%M:%S%.6f"));
        let src_file = record.file().unwrap();
        let src_file_split: Vec<_> = src_file.split('/').collect();
        let src_file = src_file_split.last().unwrap();
        style.set_intense(true);
        match record.level() {
            Level::Error=> style.set_color(Color::Red),
            Level::Warn=> style.set_color(Color::Yellow),
            Level::Info=> style.set_color(Color::Green),
            Level::Debug=> style.set_color(Color::Blue),
            Level::Trace => style.set_color(Color::Magenta),
        };
        writeln!(buf, "[{} {}:{}] {}: {}", ts, src_file, record.line().unwrap(), style.value(record.level()), record.args())
    })
        .filter_module(env!("CARGO_PKG_NAME"), LevelFilter::Trace)
        // .filter_module("iced", LevelFilter::Info)
        .init();

    log::trace!("{}", std::process::id());

    // Create app root and folders it needs
    config::ConfigFile::create_folders();

    let main_con = config::ConfigFile::new().unwrap();

    let res = Tirc::run(Settings {
        window: iced::window::Settings {
            size: (main_con.main_win_config.window_width, main_con.main_win_config.window_height),
            min_size: Some((250, 250)),
            ..Default::default()
        },
        exit_on_close_request: false,
        antialiasing: true,
        text_multithreading: true,
        ..Default::default()
    });

    log::info!("Goodbye {:?}", res);
}

async fn test() -> () {
    log::warn!("from test");
    println!("from test");
}

impl Application for Tirc {
    type Executor = executor::Default;
    type Message = AppMessages;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Tirc, Command<Self::Message>) {

        (Tirc {
            should_exit: false
        }, Command::none())
    }

    fn title(&self) -> String {
        format!("TIRC {}", env!("CARGO_PKG_VERSION"))
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessages::RuntimeEvent(event) => {

                match event {
                    Event::Window(w) => {
                        dbg!(w);
                        // if let Event::Window(window::Event::CloseRequested) = event {
                        //     self.should_exit = true;
                        // }
                    }
                    _ => {}
                }
            }
            _ => {
                dbg!(message);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let row = Row::new();
        Container::new(row).center_x().center_y().width(iced::Length::Fill).height(iced::Length::Fill).into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events().map(AppMessages::RuntimeEvent)
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }
}
