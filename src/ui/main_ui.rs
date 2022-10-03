use std::collections::VecDeque;
use std::ops::Not;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use anyhow::Error;
use dashmap::DashMap;
use eframe::emath::Align;
use egui::{RichText, TextStyle, Widget};
use egui_dock::{DockArea, NodeIndex, Style, Tree};
use flume::unbounded;
use once_cell::sync::Lazy;
use tokio::runtime::Handle;
use tokio::task::JoinHandle;
use tokio::time::sleep as tok_sleep;

use crate::config;
use crate::socket;

// use crate::modules::extract_tags::extract_tags;
// use crate::socket;

type _Res = Result<(), Error>;
type InnerThreadsArc = Mutex<DashMap<String, JoinHandle<()>>>;

pub static THREADS: Lazy<Arc<InnerThreadsArc>> = Lazy::new(|| {
    Arc::new(Mutex::new(DashMap::new()))
});

pub struct ChatAnalyser {
    user_input: String,
    thread_comms_1: Option<(flume::Sender<String>, flume::Receiver<String>)>,
    thread_comms_2: Option<(flume::Sender<String>, flume::Receiver<String>)>,
    chat_history: VecDeque<String>,
    join_chan_win: bool,
    settings_win: bool,
    run_mode: RunMode,
    run_toggle: bool,
    dark_mode: bool,
    is_exiting: bool,
    can_exit: bool,
}

impl Default for ChatAnalyser {
    fn default() -> Self {
        let max_history: u16 = 850;
        Self {
            user_input: String::new(),
            thread_comms_1: None,
            thread_comms_2: None,
            chat_history: VecDeque::with_capacity(max_history.into()),
            join_chan_win: false,
            settings_win: false,
            run_mode: RunMode::Continuous,
            run_toggle: false,
            dark_mode: true,
            is_exiting: false,
            can_exit: false,
        }
    }
}

impl ChatAnalyser {
    pub fn new(self, rt_handle: Handle, config_file: config::ConfigFile, cc: &eframe::CreationContext<'_>) -> Self {
        if config_file.main_win_config.dark_mode {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }

        let rtclone = rt_handle.clone();
        let (t, r) = unbounded();
        let t2 = t.clone();

        // THREADS.lock().unwrap().insert("test".to_owned(), rt_handle.spawn(async move {
        //     for i in 1..1000 {
        //         if let Err(e) = t2.try_send(i.to_string()) {
        //             log::error!("{:?}", e);
        //         }
        //         tok_sleep(Duration::from_millis(30)).await;
        //     }
        // }));

        THREADS.lock().unwrap().insert("test".to_owned(), rt_handle.spawn(async move {
            loop {
                if let Err(e) = socket::Socket::new_socket(t2.clone(), rtclone.clone()).await {
                    log::error!("OOFUS: {:?}", e);
                }
                tok_sleep(Duration::from_secs(1)).await;
            }
        }));

        ChatAnalyser {
            dark_mode: config_file.main_win_config.dark_mode,
            thread_comms_1: Some((t, r)),
            ..Default::default()
        }
    }

    fn render_tabs_header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").resizable(false).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(Align::LEFT), |ui| {
                    let theme_button = ui.add(egui::Button::new(if self.dark_mode { "🌙" } else { "☀" } ));
                    let add_channel_button = ui.add(egui::Button::new("➕"));
                    let settings_button = ui.add(egui::Button::new("🛠"));

                    if theme_button.clicked() { self.dark_mode = !self.dark_mode; }
                    else if theme_button.hovered() && self.dark_mode {
                        theme_button.on_hover_text("Switch to light theme");
                    } else if theme_button.hovered() && !self.dark_mode {
                        theme_button.on_hover_text("Switch to dark theme");
                    }

                    if add_channel_button.clicked() {
                        self.join_chan_win = true;
                    } else if add_channel_button.hovered() {
                        add_channel_button.on_hover_text("Join a channel");
                    }

                    if settings_button.clicked() {
                        self.settings_win = !self.settings_win;
                    } else if settings_button.hovered() {
                        settings_button.on_hover_text("Open the settings menu");
                    }
                });
            });
        });
    }

    fn render_main(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                self.render_msg(ui);
            });
        });
    }

    fn render_footer(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("footer").resizable(false).show(ctx, |ui| {
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                ui.hyperlink_to(egui::special_emojis::GITHUB.to_string(), "https://github.com/GCNull/chat_analyser");
            });
        });
    }

    fn render_msg(&mut self, ui: &mut egui::Ui) {
        let mut n: u16 = 0;
        for i in &self.chat_history.clone() {
            let l = RichText::new(i).text_style(TextStyle::Monospace);
            ui.add(egui::Label::new(l));
            if n >= 850 {
                self.chat_history.pop_front();
                n = 850;
            }
            n += 1;
        }
    }

    fn fetch_mes(&mut self) {
        if let Some(rx) = &self.thread_comms_1 {
            match rx.1.try_recv() {
                Ok(r) => {
                    self.chat_history.push_back(r);
                }
                Err(e) => {
                    if !e.to_string().to_lowercase().contains("empty") {
                        log::error!("{:?}", e)
                    }
                },
            }
        }
    }

    fn join_chan_popup(&mut self, ctx: &egui::Context) {
        egui::Window::new(egui::WidgetText::from("Join a channel").strong()).collapsible(false).resizable(false).auto_sized().open(&mut self.join_chan_win).show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(Align::LEFT), |ui| {
                let r = ui.add(egui::TextEdit::singleline(&mut self.user_input).hint_text("Channel name"));
                if r.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    if let Some(t) = &self.thread_comms_1 {
                        if let Err(e) = t.0.try_send(self.user_input.clone()) {
                            log::error!("Failed to send message: {:?}", e);
                        } else {
                            log::debug!("Sent {:?}", self.user_input);
                        }
                    }
                }
            });
        });
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("settings_pane").show(ctx, |ui| {
            let sixfps = ui.checkbox(&mut self.run_toggle, "Toggle always-on 60 fps");
            if sixfps.sense.click {
                log::trace!("CLCED");
                match self.run_toggle {
                    true => {
                        self.run_mode = RunMode::Continuous;
                    },
                    false => {
                        self.run_mode = RunMode::Reactive;
                    },
                }
            }
        });
    }
}

impl eframe::App for ChatAnalyser {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut style: egui::Style = (*ctx.style()).clone();
        if self.dark_mode {
            style.visuals.override_text_color = Some(egui::Color32::WHITE);
            ctx.set_style(style);
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            style.visuals.override_text_color = Some(egui::Color32::BLACK);
            ctx.set_style(style);
            ctx.set_visuals(egui::Visuals::light());
        }

        self.render_tabs_header(ctx);
        self.render_footer(ctx);
        self.render_main(ctx);
        self.fetch_mes();

        if self.join_chan_win {
            self.join_chan_popup(ctx);
        }

        if self.settings_win {
            self.settings_window(ctx);
        }

        if self.is_exiting {
            frame.close();
            config::MainWindowConfig::save_window_to_json(frame.info().window_info, ctx.style().visuals.dark_mode);
            self.can_exit = true;
        }

        match self.run_mode {
            RunMode::Continuous => ctx.request_repaint(),
            RunMode::Reactive => return,
        }
    }

    fn on_close_event(&mut self) -> bool {
        self.is_exiting = true;
        self.can_exit
    }

    fn warm_up_enabled(&self) -> bool {
        true
    }
}

enum RunMode {
    Reactive,
    Continuous,
}
