#![warn(clippy::all)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::env;
use std::io::Write;

use anyhow::Error;
use chrono::Local;
use egui::vec2;
use env_logger::{Builder as env_builder, fmt::Color, WriteStyle};
use log::{Level, LevelFilter};
use tokio::runtime::Builder;
use crate::ui::main_ui::ChatAnalyser;

mod config;
mod ui;

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
    let options = eframe::NativeOptions {
        initial_window_size: Some(vec2(main_con.main_win_config.window_width, main_con.main_win_config.window_height)),
        initial_window_pos: Some(egui::Pos2::new(main_con.main_win_config.window_position_x, main_con.main_win_config.window_position_y)),
        min_window_size: Some(vec2(250.0, 250.0)),
        decorated: true,
        ..Default::default()
    };

    let runtime = Builder::new_multi_thread().thread_name("analyser_runtime").enable_all().build().unwrap();
    let handle = runtime.handle().clone();

    runtime.block_on(async move {
        let t = ChatAnalyser::default();
        eframe::run_native(&format!("Chat Analyser {}", env!("CARGO_PKG_VERSION")), options, Box::new(|cc| Box::new(t.new(handle, main_con, cc))));
    });
    runtime.shutdown_background();
    log::info!("Goodbye");
    Ok(())
}
