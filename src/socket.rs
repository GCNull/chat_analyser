use std::error::Error;
use std::process;

use chrono::Local;
use dashmap::DashMap;
use flume::unbounded;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep, timeout};
use tokio_native_tls::{native_tls, TlsConnector};
use tokio_stream::wrappers::BroadcastStream;

use crate::parse_twitch_data::extract_tags;

type Res = Result<(), anyhow::Error>;

static LAST_DATA: Lazy<Mutex<i64>> = Lazy::new(|| {
    Mutex::new(0_i64)
});

#[derive(Debug)]
struct User {
    user_id: String,
    user_type: String,
    vip: Option<String>,
}
pub struct Socket;

impl Socket {
    pub async fn new_socket(sender: flume::Sender<String>, runtime: Handle) -> Res {
        log::warn!("Starting new socket connection!");
        let (sock_send, _sock_recv) = unbounded();
        let s2 = sock_send.clone(); // socket recv time
        let s3 = sock_send.clone(); // ping server
        let s4 = sock_send.clone(); // ping client

        let mut bot_threads: Vec<tokio::task::JoinHandle<_>> = Vec::new();

        // Check last socket recv time
        bot_threads.push(runtime.spawn(async move {
            loop {
                sleep(Duration::from_secs(25)).await;
                let t = LAST_DATA.lock().await;
                if Local::now().timestamp() - *t >= 25 {
                    if let Err(e) = s2.send("forced kys".to_string()) {
                        log::error!("Failed to send kill command to bots: {}", e);
                    }
                    panic!("25 seconds passed with no data from socket!");
                }
            }
        }));

        match timeout(Duration::from_millis(5000), TcpStream::connect("irc.chat.twitch.tv:6697")).await {
            Ok(socket) => {
                match socket {
                    Ok(socket2) => {
                        let local_ip = socket2.local_addr().unwrap();
                        let peer_ip = socket2.peer_addr().unwrap();
                        let tls_builder = native_tls::TlsConnector::builder().danger_accept_invalid_certs(false).danger_accept_invalid_hostnames(false).use_sni(false).build().unwrap();
                        let tls_connector = TlsConnector::from(tls_builder);

                        match tls_connector.connect("irc.chat.twitch.tv", socket2).await {
                            Ok(tls_stream) => {
                                let (stream, wstream) = tokio::io::split(tls_stream);
                                let mut stream = BufReader::new(stream);
                                let mut wstream = BufWriter::new(wstream);
                                let mut buff = String::new();

                                for i in [
                                    "CAP REQ :twitch.tv/tags", "CAP REQ :twitch.tv/commands",
                                    "CAP REQ :twitch.tv/membership",
                                    // &format!("PASS {}", config_file.bot_fields.oauth),
                                    &format!("NICK {}", "justinfan77177777"),
                                    &format!("JOIN #{}", "moonmoon")
                                ] {
                                    send_raw_message(&mut wstream, i).await.unwrap();
                                }
                                log::info!("PID: {} | connected to Twitch IRC on host: {} | Local IP: {}\n", process::id(), peer_ip, local_ip);

                                // Send ping to Twitch every 5 seconds
                                bot_threads.push(runtime.spawn(async move {
                                    loop {
                                        if let Err(e) = s3.send("debug PING :tmi.twitch.tv".to_string()) {
                                            log::error!("Failed to send PING to Twitch: {}", e);
                                        }
                                        sleep(Duration::from_secs(5)).await;
                                    }
                                }));

                                // Handle socket input stream
                                if BufReader::read_line(&mut stream, &mut buff).await? == 0 {
                                    log::warn!("Nothing read on the socket!");
                                    if let Err(e) = sock_send.send("forced kys".to_string()) {
                                        log::error!("Failed to send kill command to bots: {}", e);
                                    }
                                    sleep(Duration::from_secs(1)).await;
                                } else {
                                    while BufReader::read_line(&mut stream, &mut buff).await? > 0 {
                                        let mut t = LAST_DATA.lock().await;
                                        *t = Local::now().timestamp();
                                        let buffer = buff.trim().to_string();
                                        // log::debug!("{:?}", buffer);

                                        if buffer == ":tmi.twitch.tv RECONNECT" {
                                            log::warn!("Twitch has requested that a new connection must be made!");
                                            break
                                        }
                                        // if let Err(e) = ParseTwitchData::parse_twitch_data(buffer.clone()).await {
                                        //     log::error!("An error while parsing Twitch data occurred: {} buffer was: [{}]", e, buffer);
                                        // }
                                        if buffer.contains("PRIVMSG") {
                                            let channel: Vec<&str> = buffer.split(' ').collect();
                                            let channel = channel[3].replace('#', "");
                                            let user = User {
                                                user_id: extract_tags(&buffer).get("user-id").unwrap().clone(),
                                                user_type: extract_tags(&buffer).get("user-type").unwrap().clone(),
                                                vip: if extract_tags(&buffer).get("vip").is_some() {
                                                    Some(extract_tags(&buffer).get("vip").unwrap().clone())
                                                } else { None }
                                            };
                                            let raw_user: Vec<&str> = user.user_type.split(|c| c == '!' || c == '@').collect();
                                            let raw_user = if user.vip.is_some() {
                                                let t = user.vip.unwrap();
                                                let t2 = t.split(|c| c == ':' || c == '!').collect::<Vec<&str>>();
                                                t2[1].to_string()
                                            } else {
                                                raw_user[1].to_string()
                                            };
                                            let raw_message = buffer.rsplit(&format!("{} :", &channel)).next().unwrap().trim();
                                            let raw_message = raw_message.replace("  ", " ");


                                            if let Err(e) = sender.send(format!("{} {}: {}", Local::now().format("%T"), raw_user, raw_message)) {
                                                log::error!("Failed to send buffer to app: {:?}", e);
                                            }

                                        }
                                        if buffer.contains("PING :tmi.twitch.tv") {
                                            s4.send("debug PONG :tmi.twitch.tv".to_string())?;
                                        }
                                        buff.clear();
                                    }
                                    if BufReader::read_line(&mut stream, &mut buff).await? == 0 {
                                        log::warn!("Socket disconnected");
                                        sleep(Duration::from_secs(1)).await;
                                    }
                                    log::warn!("Broke out of buffer loop!");
                                    sleep(Duration::from_secs(1)).await;
                                }
                            }
                            Err(e) => log::error!("An error occurred with the SSL connection: {}", e),
                        }
                    }
                    Err(e) => log::error!("An error occurred with the Twitch connection: {:?}", e),
                }
            }
            Err(_) => log::warn!("Timed out while connecting to Twitch"),
        }

        async fn send_raw_message<W: AsyncWrite + Unpin>(w: &mut W, msg: &str) -> Result<(), std::io::Error> {
            let message = format!("{}\r\n", msg);
            match w.write(message.as_bytes()).await {
                Ok(_) => {
                    if let Err(e) = w.flush().await {
                        if e.to_string().to_lowercase().contains("broken pipe") { panic!("{}", e) } else { log::error!("{}", e); }
                    }
                    if message != "PING :tmi.twitch.tv\r\n" { log::debug!("sent: {:?}", message); }
                }
                Err(e) => log::error!("{}", e),
            }
            Ok(())
        }

        bot_threads.iter().for_each(|val| {
            val.abort();
        });
        drop(runtime);
        Ok(())
    }
}
