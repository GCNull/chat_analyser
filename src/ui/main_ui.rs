use std::collections::VecDeque;
use std::ops::Not;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use anyhow::Error;
use dashmap::DashMap;
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

enum RunMode {
    Reactive,
    Continuous,
}
