use std::collections::VecDeque;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Error;
use crossbeam_channel::unbounded;
use eframe::emath::Align;
use flume::unbounded as flu;
use tokio::runtime::Handle;
use tokio::sync::broadcast::Sender;
use tokio::time::sleep as tok_sleep;

use crate::config;
use crate::config::ConfigFile;

// use crate::modules::extract_tags::extract_tags;
// use crate::socket;

type _Res = Result<(), Error>;
// type InnerThreadsArc = Mutex<DashMap<String, JoinHandle<()>>>;

// pub static THREADS: Lazy<Arc<InnerThreadsArc>> = Lazy::new(|| {
//     Arc::new(Mutex::new(DashMap::new()))
// });

pub struct ChatAnalyser {
    user_input: String,
    thread_comms: (crossbeam_channel::Sender<String>, crossbeam_channel::Receiver<String>),
    chat_history: VecDeque<String>,
    join_chan_win: bool,
    settings_win: bool,
    run_mode: RunMode,
    dark_mode: bool,
    is_exiting: bool,
    can_exit: bool,
}

impl ChatAnalyser {
    pub fn init(self, rt_handle: Handle, config_file: ConfigFile, cc: &eframe::CreationContext<'_>) -> Self {
        if config_file.main_win_config.dark_mode {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }

        ChatAnalyser {
            dark_mode: config_file.main_win_config.dark_mode,
            ..Default::default()
        }
    }

    fn fetch_mes(&mut self) {
        match self.thread_comms.1.try_recv() {
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

    fn render_tabs_header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").resizable(false).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
                    let theme_button = ui.add(egui::Button::new(if self.dark_mode { "🌙" } else { "☀" } ));
                    let add_channel_button = ui.add(egui::Button::new("➕"));

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
        let mut n = 0;
        for i in &self.chat_history.clone() {
            ui.add(egui::TextEdit::singleline(&mut i.as_str()).text_color(if self.dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK }));
            n += 1;
            if n == 850 {
                self.chat_history.pop_front();
            }
        }
    }

    fn join_chan_popup(&mut self, ctx: &egui::Context) {
        egui::Window::new(egui::WidgetText::from("Join a channel").strong()).collapsible(false).resizable(false).auto_sized().open(&mut self.join_chan_win).show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(Align::LEFT), |ui| {
                let r = ui.add(egui::TextEdit::singleline(&mut self.user_input).hint_text("Channel name"));
                if r.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    if let Err(e) = self.thread_comms.0.clone().try_send(self.user_input.clone()) {
                        log::error!("Failed to send message: {:?}", e);
                    } else {
                        log::debug!("Sent {:?}", self.user_input);
                    }
                }
            });
        });
    }
}

impl Default for ChatAnalyser {
    fn default() -> Self {
        Self {
            user_input: String::new(),
            thread_comms: unbounded(),
            chat_history: VecDeque::with_capacity(850),
            join_chan_win: false,
            settings_win: false,
            run_mode: RunMode::Continuous,
            dark_mode: true,
            is_exiting: false,
            can_exit: false,
        }
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
        self.render_main(ctx);
        self.render_footer(ctx);
        self.fetch_mes();

        if self.join_chan_win {
            self.join_chan_popup(ctx);
        }

        if self.is_exiting {
            frame.close();
            config::MainWindowConfig::save_window_to_json(frame.info().window_info, ctx.style().visuals.dark_mode);
            self.can_exit = true;
        }

        match self.run_mode {
            RunMode::Continuous => ctx.request_repaint(),
            RunMode::Reactive => ctx.request_repaint_after(Duration::from_secs(5)),
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
