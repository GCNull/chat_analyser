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
use log::{Level, LevelFilter};
use tokio::runtime::Builder;

mod config;
mod ui;
mod socket;
mod parse_twitch_data;

type Res<T> = Result<T, Error>;

#[cfg(not(target_family = "unix"))]
fn main() -> Res<()> {
    panic!("This app is built for Linux devices only")
}

#[cfg(target_family = "unix")]
fn main() -> Res<()> {
    log::trace!("{}", std::process::id());
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
        .init();

    std::panic::set_hook(Box::new(move |panic| {
        log::error!("{}", panic);
    }));

    // Create app root and folders it needs
    config::ConfigFile::create_folders();

    let main_con = config::ConfigFile::new()?;

    Counter::run(Settings {
        window: iced::window::Settings {
            size: (main_con.main_win_config.window_width, main_con.main_win_config.window_height),
            min_size: Some((250, 250)),
            ..Default::default()
        },
        ..Default::default()
    }).unwrap();
    log::info!("Goodbye");
    Ok(())
}

struct Counter;

impl Application for Counter {
    type Executor = executor::Default;
    type Message = Recv;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Counter, Command<Self::Message>) {
        (Counter, Command::none())
    }

    fn title(&self) -> String {
        format!("TIRC {}", env!("CARGO_PKG_VERSION"))
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Recv::Notice => {}
            Recv::Privmsg => {}
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

    fn should_exit(&self) -> bool {
        true
    }
}

#[derive(Debug)]
enum Recv {
    Notice,
    Privmsg,
}