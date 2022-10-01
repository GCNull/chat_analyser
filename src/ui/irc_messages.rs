#[derive(Debug, Clone)]
pub struct Privmsg {
    pub _turbo: String,
    pub display_name: String,
    pub _id: String,
    pub _flags: String,
    pub _badge_info: String,
    pub _emotes: String,
    pub _tmi_sent_ts: String,
    pub badges: String,
    pub _subscriber: String,
    pub _mod: String,
    pub user_id: String,
    pub user_type: String,
    pub _colour: String,
    pub _room_id: String,
}

pub struct Notice;