#[derive(Debug, Clone)]
pub struct Privmsg {
    pub turbo: String,
    pub display_name: String,
    pub id: String,
    pub flags: String,
    pub badge_info: String,
    pub emotes: String,
    pub tmi_sent_ts: String,
    pub badges: String,
    pub subscriber: String,
    pub _mod: String,
    pub user_id: String,
    pub user_type: String,
    pub colour: String,
    pub room_id: String,
}

pub struct Notice;