mod commands;
mod event_handler;


use poise::serenity_prelude as serenity;


use serde::Deserialize;
use serenity::{
    all::{ChannelId, UserId},
    prelude::*,
};
use songbird::{
    events::{TrackEvent},
    SerenityInit,
};

use std::sync::Arc;
use std::time::Duration;
use std::{
    collections::HashMap,
    env,
    fs::{self},
};

use toml::{self, from_str};


type Error = Box<dyn std::error::Error + Send + Sync>;
type PoiseContext<'a> = poise::Context<'a, Data, Error>;
pub struct Data {
    channel_id: Mutex<ChannelId>,
    bot_id: Mutex<UserId>,
    join_leave_message_database: Mutex<HashMap<String, JoinLeaveMessages>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // Custom Error handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command '{}' : {:?}", ctx.command().name, error,);
        }
        _ => {
            println!("Unhandled Error!");
        }
    }
}

// Structs for reading the toml of join and leave messages
#[derive(Deserialize, Debug)]
struct JoinLeaveMessageExt {
    id: String,
    #[serde(flatten)]
    inner: JoinLeaveMessages,
}

// Struct for internal use of join leave messages, the ID is the key allowing for quick searching
#[derive(Deserialize, Debug)]
struct JoinLeaveMessages {
    name: String,
    join: Vec<String>,
    leave: Vec<String>,
}

// To know the current ChannelID that the bot is in
// TODO: This does mean the bot only knows 1 channel it is in (noteably the last one it joined) meaning if it is in two channels it will only log people entering / leaving in it's most recent one. I should upgrade this to work
//       with the bot in more than 1 channel though that might take a bit of thinking

// Moving the database of join leave messages across threads for the handler
struct JoinLeaveMessageDatabase;
impl TypeMapKey for JoinLeaveMessageDatabase {
    type Value = HashMap<String, JoinLeaveMessages>;
}
// TODO: Clean up this function
// TODO: A bot command should also run this so I can update join leave messages
// set up the recorded messages into the database
async fn set_recorded_messages(data: &Data) {
    let f = fs::read_to_string("./prerecordedtable.toml").expect("No prerecorded info, please add");
    let table: HashMap<String, Vec<JoinLeaveMessageExt>> = from_str(&f).unwrap();
    let accounts: &[JoinLeaveMessageExt] = &table["User"];
    let mut new_accounts: HashMap<String, JoinLeaveMessages> = HashMap::new();
    accounts.iter().for_each(|value| {
        new_accounts.insert(
            value.id.to_string(),
            JoinLeaveMessages {
                name: value.inner.name.clone(),
                join: value.inner.join.clone(),
                leave: value.inner.leave.clone(),
            },
        );
    });
    let mut write_database = data.join_leave_message_database.lock().await;
    *write_database = new_accounts;
}
#[tokio::main]
async fn main() {
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::about::about(),
            commands::join::join(),
            commands::say::say(),
            commands::leave::leave(),
            commands::about::help(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(100),
            ))),
            additional_prefixes: vec![poise::Prefix::Literal("Sir")],
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),

        pre_command: |ctx| {
            Box::pin(
                async move { println!("Executing Command {}...", ctx.command().qualified_name) },
            )
        },
        post_command: |ctx| {
            Box::pin(
                async move { println!("Executed Command {}...", ctx.command().qualified_name) },
            )
        },
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler::event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };
    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    channel_id: Mutex::new(ChannelId::new(1)),
                    bot_id: Mutex::new(UserId::new(1)),
                    join_leave_message_database: Mutex::new(HashMap::new()),
                })
            })
        })
        .options(options)
        .build();
    // that's our prefix, we look for messages with that

    let token = env::var("DISCORD_TOKEN").expect("Token error");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Failed creating discord client");
    if let Err(why) = client.start().await {
        println!(
            "An Error {} has occurred whilst starting discord client",
            why
        );
    }
    client.start().await.unwrap()
}
