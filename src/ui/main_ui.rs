use anyhow::Error;
use eframe::emath::Align;
use egui::{Color32, Context};
use tokio::runtime::Handle;
use tokio::sync::broadcast::Sender;
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
    dark_mode: bool,
    is_exiting: bool,
    can_exit: bool,
}

impl ChatAnalyser {
    pub fn init(self, _rt_handle: Handle, config_file: ConfigFile, cc: &eframe::CreationContext<'_>) -> Self {
        dbg!(&config_file);
        if config_file.main_win_config.dark_mode {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }
        // let socket_send_buffer: (crossbeam_channel::Sender<String>, crossbeam_channel::Receiver<String>) = unbounded();

        ChatAnalyser {
            dark_mode: config_file.main_win_config.dark_mode,
            ..Default::default()
        }
    }

    fn render_tabs_header(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("header").resizable(false).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
                    let theme_button = ui.add(egui::Button::new(if self.dark_mode { "🌙" } else { "☀" } ));
                    let add_channel_button = ui.add(egui::Button::new("➕"));

                    if theme_button.clicked() { self.dark_mode = !self.dark_mode; }

                    if theme_button.hovered() && self.dark_mode {
                        theme_button.on_hover_text("Switch to light theme");
                    } else if theme_button.hovered() && !self.dark_mode {
                        theme_button.on_hover_text("Switch to dark theme");
                    }

                    if add_channel_button.clicked() {
                        egui::Window::new("Join a channel").collapsible(false).resizable(false).auto_sized().show(ctx, |_ui| {
                            _ui.colored_label(Color32::RED, "POGG")
                        });
                    }
                    if add_channel_button.hovered() {
                        add_channel_button.on_hover_text("Join a channel");
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("footer").resizable(false).show(ctx, |ui| {
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                ui.hyperlink_to("Github", "https://github.com/GCNull/chat_analyser");
            });
        });
    }
    fn render_t(&mut self, ctx: &Context) {
        egui::SidePanel::left("Info").resizable(true).show(ctx, |ui| {
            let mut buff = String::new();
            let inp = egui::TextEdit::singleline(&mut buff).hint_text("TEST");
            let r = ui.add_sized(ui.available_size(), inp);
            if r.changed() {
                log::debug!("change 1")
            }
            if r.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                log::debug!("Sent");
                r.request_focus();
            }

        });
        egui::CentralPanel::default().show(ctx, |_ui| {});
    }
}

impl Default for ChatAnalyser {
    fn default() -> Self {
        Self {
            // live_msgs: Default::default(),
            dark_mode: true,
            is_exiting: false,
            can_exit: false,
        }
    }
}

impl eframe::App for ChatAnalyser {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
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

        self.render_tabs_header(&ctx);
        self.render_t(&ctx);

        if self.is_exiting {
            frame.close();
            config::MainWindowConfig::save_window_to_json(frame.info().window_info);
            self.can_exit = true;
        }
        ctx.request_repaint();
    }

    fn on_close_event(&mut self) -> bool {
        self.is_exiting = true;
        self.can_exit
    }

    fn warm_up_enabled(&self) -> bool {
        true
    }
}