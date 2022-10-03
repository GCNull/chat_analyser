use dashmap::DashMap;

pub fn extract_tags(tags: &str) -> DashMap<String, String> {
    let irc_tags = tags.trim_start_matches('@').trim_end_matches("PRIVMSG");
    irc_tags.split(';').flat_map(|tag| {
        if let Some(so) = tag.split_once('=') {
            Some((so.0.to_string(), so.1.to_string()))
        } else { Some(("".to_string(), "".to_string())) }
    }).collect()
}
