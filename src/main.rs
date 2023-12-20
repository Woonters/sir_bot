use reqwest::get;
use serenity::async_trait;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, Configuration, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::{
    env,
    fs::File,
    io::{stdin, Write},
};
use urlencoding::encode;

#[group]
#[commands(ping, say)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new().prefix("~"));
    // that's our prefix, we look for messages with that

    let token = env::var("DISCORD_TOKEN").expect("Token error");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Failed creating discord client");
    if let Err(why) = client.start().await {
        println!(
            "An Error {} has occurred whilst starting discord client",
            why
        );
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    tokio::task::spawn_blocking(|| {
        let text = fix_input(&get_input());
        get_voice_and_save(&text);
    })
    .await
    .expect("Task Panicked");
    msg.reply(ctx, format!("Pong - {}", msg.content)).await?;
    Ok(())
}

#[command]
async fn say(ctx: &Context, msg: &Message) -> CommandResult {
    let content = msg.content.clone();
    tokio::task::spawn_blocking(move || {
        let text = fix_input(&content[5..]);
        get_voice_and_save(&text);
    })
    .await
    .expect("Task Panicked");
    msg.reply(ctx, "I Have just created the file;").await?;
    Ok(())
}

#[tokio::main]
async fn get_voice_and_save(input: &str) {
    let response = get(format!("https://api.novelai.net/ai/generate-voice?text={}&seed=aHaleAndHeartySir&voice=-1&opus=false&version=v2",input));
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

fn get_input() -> String {
    println!("WHAT WOULD YOU LIKE ME TO SAY SIR?! ");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("I DID NOT UNDERSTAND THAT SIR");
    input
}

fn fix_input(input: &str) -> String {
    let binding = input.to_uppercase();
    let encoded = encode(&binding);
    encoded.into_owned()
}
mod tests {
    // Tests are atm not working, I don't think that's the worst thing in the world atm, but I
    // would like to get it working eventually
    use crate::{fix_input, get_input, get_voice_and_save};

    #[test]
    fn test_voice_api() {
        let text = fix_input(&get_input());
        get_voice_and_save(&text);
    }
}
