use teloxide::types::{MediaKind, MediaText, MessageEntityKind, MessageEntity, InputFile};
use teloxide::{prelude::*, types::MessageKind, net};
use std::env;
use std::str;
use std::process::Command;
use std::time::Duration;
use rand::{self, Rng};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let client = net::default_reqwest_settings().timeout(Duration::from_secs(60 * 10))
        .build()
        .expect("Failed to create reqwest client");

    let token = env::var("TELOXIDE_TOKEN").expect("Bot token not found!");
    let bot = Bot::with_client(token, client);

    println!("Up and running!");

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {

        match msg.kind {
            MessageKind::Common(data) => {
                // println!("{:?}", data);

                if let MediaKind::Text(media_text) = data.media_kind {
                    let urls = get_urls(media_text);

                    for url in urls {
                        let path = match download(&url).await {
                            Ok(path) => path,
                            Err(_) => continue,
                        };

                        println!("{:?}", path);
                        let file = InputFile::file(path);
                        bot.send_video(msg.chat.id, file).await?;
                    }
                }
            },
            _ => {
                bot.send_message(msg.chat.id, "Unsupported operation").await?;
            }
        }

        // bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;
}

enum DownloadErr {
    InvalidLink,
}

fn get_urls(media_text: MediaText) -> Vec<String> {
    media_text
        .entities
        .iter()
        .filter(|entity| entity.kind == MessageEntityKind::Url)
        .map(|MessageEntity { kind: _, offset, length }| {
            media_text.text.as_str()[*offset..(*offset + *length)].to_owned()
        }).collect()
}

async fn download(url: &str) -> Result<String, DownloadErr> {
    let filename: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let ext = "mp4";

    let path = format!("./.downloads/{}.{}", filename, ext); 

    Command::new("./binaries/yt-dlp")
        .arg("--cookies")
        .arg("./cookies.txt")
        .arg("--remux-video")
        .arg(ext)
        .arg("-o")
        .arg(&path)
        .arg(format!("{}", url))
        .status()
        .map_err(|e| {
            println!("{:?}", e);
            DownloadErr::InvalidLink
        })?;

    Ok(path)
}
