pub struct Tirc;

#[derive(Debug)]
pub enum AppMessages {
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
