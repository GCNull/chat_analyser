#[derive(Debug, Clone)]
pub struct Tirc {
    pub(crate) should_exit: bool
}

#[derive(Debug)]
pub enum AppMessages {
    RuntimeEvent(iced_native::Event),
    ChannelMessage,
    Clearchat,
    Clearmsg,
    GlobalMessage,
    Globaluserstate,
    Hosttarget,
    Join,
    Notice,
    Part,
    Privmsg,
    Reconnect,
    Roomstate,
    Usernotice,
    Userstate,
    Whisper,
}
