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
        Some(v) => _say(ctx, msg, v).await,
        None => _say(ctx, msg, "aHaleAndHeartySir".to_string()).await,
    }
}

async fn _say(ctx: PoiseContext<'_>, msg: String, voice: String) -> Result<(), Error> {
    let content = msg.clone();
    let guild_id = ctx.guild_id();
    let seed = voice.clone();
    tokio::task::spawn_blocking(move || {
        let text = fix_input(&content);
        get_voice_and_save(&text, &seed);
    })
    .await
    .expect("Task Panicked");
    // ok now let's get the songbird thingy and play some audio!!!
    let mut f = File::open("audio/temp.mpeg").unwrap();
    let mut input = vec![];
    let _ = Read::read_to_end(&mut f, &mut input);
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
    let _ = ctx.reply(":)").await;
    Ok(())
}

pub async fn say_saved(ctx: &Context, guild_id: GuildId, file_path: &String) -> CommandResult {
    let mut f = File::open(format!("audio/{file_path}.mpeg")).unwrap();
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

#[tokio::main]
async fn get_voice_and_save(input: &str, voice: &str) {
    let response = get(format!(
        "https://api.novelai.net/ai/generate-voice?text={}&seed={}&voice=-1&opus=false&version=v2",
        input, voice
    ));
    match response.await {
        Ok(resp) => {
            if resp.status().is_success() {
                let bytes = resp
                    .bytes()
                    .await
                    .expect("SOMETHING VERY WRONG HAS HAPPENED SIR");
                let mut file = File::create("audio/temp.mpeg").expect("File creation failed");
                file.write_all(&bytes).unwrap();
                println!("Message received, file saved, all success")
            } else {
                println!("Bad Response: {}", resp.status());
            }
        }
        Err(e) => println!("Request failed {}", e),
    }
}

fn fix_input(input: &str) -> String {
    let binding = input.to_uppercase();
    let encoded = encode(&binding);
    encoded.into_owned()
}
