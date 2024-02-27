use reqwest::get;
use serenity::all::GuildId;
use serenity::framework::standard::CommandResult;
use serenity::prelude::Context;
use urlencoding::encode;

use crate::{Error, PoiseContext};
use std::fs::File;
use std::io::{Read, Write};
use std::num::NonZeroU64;
/// Say Something with the TTS
#[poise::command(prefix_command, slash_command, aliases("s"))]
pub async fn say(
    ctx: PoiseContext<'_>,
    #[description = "What you would like the bot to say"] msg: String,
    #[description = "Use another voice, any string will work"] voice: Option<String>,
) -> Result<(), Error> {
    match voice {
        // This means the bot will default to "aHaleandHeartySir" (blue gnome voice)
        Some(v) => _say(ctx, msg, v, None).await,
        None => _say(ctx, msg, "aHaleAndHeartySir".to_string(), None).await,
    }
}

async fn _say(
    ctx: PoiseContext<'_>,
    msg: String,
    voice: String,
    save: Option<&str>,
) -> Result<(), Error> {
    let content = msg.clone();
    let guild_id = ctx.guild_id();
    let seed = voice.clone();
    let text = fix_input(&content);
    let _input = get_voice(&text, &seed, save).await;
    let input: bytes::Bytes = match _input {
        Ok(i) => i,
        Err(e) => {
            log::error!("{:?}", e);
            return Err(Box::new(e));
        }
    };
    if let Some(handler_lock) = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client not found")
        .clone()
        .get(songbird::id::GuildId(
            NonZeroU64::new(guild_id.unwrap().get()).unwrap(),
        ))
    {
        let mut handler = handler_lock.lock().await;
        let _ = handler.play_input(input.into());
    }
    let _ = ctx.reply(":)").await; // TODO: Better text response, maybe just say some gnome stuff
    Ok(())
}

pub async fn say_saved(ctx: &Context, guild_id: GuildId, file_path: &String) -> CommandResult {
    let mut f = match File::open(format!("audio/{file_path}.mpeg")) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to open file {:?} | {:?}", file_path, e);
            return Err(Box::new(e));
        }
    };
    let mut input = vec![];
    let _ = f.read_to_end(&mut input);
    if let Some(handler_lock) = songbird::get(ctx)
        .await
        .expect("Songbird Client not working")
        .clone()
        .get(guild_id)
    {
        let mut handler = handler_lock.lock().await;
        let _ = handler.play_input(input.into());
    }
    Ok(())
}

async fn get_voice(
    input: &str,
    voice: &str,
    save: Option<&str>,
) -> Result<bytes::Bytes, std::io::Error> {
    let response = get(format!(
        "https://api.novelai.net/ai/generate-voice?text={}&seed={}&voice=-1&opus=false&version=v2",
        input, voice
    ));
    match response.await {
        Ok(resp) => {
            if resp.status().is_success() {
                let bytes = match resp.bytes().await {
                    Ok(b) => b,
                    Err(_) => {
                        log::error!(
                        "Voice request responded sucess but failed to be parsed, contact novel ai"
                    );
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "failed to read positive response",
                        ));
                    }
                };
                if let Some(filename) = save {
                    let mut file = File::create(format!("audio/{:?}.mpeg", filename))
                        .expect("File creation failed");
                    file.write_all(&bytes).unwrap();
                    log::info!("Message received, file saved, all success");
                }
                log::info!("Message received, bytes sent forward");
                return Ok(bytes);
            } else {
                log::error!("Bad Response: {}", resp.status());
            }
        }
        Err(e) => log::error!("Request failed {}", e),
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Couldn't generate voice",
    ))
}

fn fix_input(input: &str) -> String {
    let binding = input.to_uppercase();
    let encoded = encode(&binding);
    encoded.into_owned()
}
