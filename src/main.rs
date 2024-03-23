mod commands;
mod event_handler;
mod sir_error;

use log::{error, info};
use poise::serenity_prelude as serenity;
use serde::Deserialize;
use serenity::{
    all::{ChannelId, UserId},
    prelude::*,
};
use songbird::{events::TrackEvent, SerenityInit};

use std::time::Duration;
use std::{
    collections::HashMap,
    env,
    fs::{self, DirBuilder},
};
use std::{io::Write, sync::Arc};

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
            log::error!("Error in command '{}' : {:?}", ctx.command().name, error);
        }
        _ => {
            log::error!("Unhandled Error!");
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
// TODO: The error messages are weak in this function, perhaps change to returning a Result?
async fn set_recorded_messages(data: &Data) {
    let f = fs::read_to_string("./prerecordedtable.toml")
        .expect("failed to read prerecordedmessages.toml");
    let table: HashMap<String, Vec<JoinLeaveMessageExt>> =
        from_str(&f).expect("The format of prerecorded table was wrong");

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
    info!("set join and leave messages");
}

async fn download_gnome_images() -> Result<(), Error> {
    for number in 1..9 {
        let response = reqwest::get(format!(
            "https://raw.githubusercontent.com/Fritzbox2000/sir_bot/master/images/gnome_0{}.jpg",
            number
        ));
        match response.await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let bytes = match resp.bytes().await {
                        Ok(b) => b,
                        Err(_) => {
                            log::error!("Getting gnome image {} has failed", number);
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Failed to get gnome image via request",
                            )));
                        }
                    };
                    let mut file = fs::File::create(format!("images/gnome_{}.jpg", number))
                        .expect("File creation failed");
                    file.write_all(&bytes).unwrap();
                    log::info!("Saved gnome image");
                }
            }
            Err(e) => log::error!("Request failed {}", e),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    // let's setup the logger
    env_logger::init();
    // do folder setup
    // check the folders don't exist.
    let images_folder = fs::create_dir("images");
    let audio_folder = fs::create_dir("audio");
    match images_folder {
        Ok(_) => {
            download_gnome_images().await.unwrap();
            log::info!("Images folder created and filled with 9 gnomes");
        }
        Err(_) => {
            log::info!("Images folder already exists so no gnome images were got")
        }
    }
    match audio_folder {
        Ok(_) => {
            log::info!("Audio folder created")
        }
        Err(_) => {
            log::info!("Audio folder already exists carrying on")
        }
    }
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::about::about(),
            commands::join::join(),
            commands::say::say(),
            commands::leave::leave(),
            commands::about::help(),
            commands::reload_messages::reload_join_leave_messages(),
            commands::show_gnome::show_gnome(),
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
            Box::pin(async move { info!("Executing Command {}", ctx.command().qualified_name) })
        },
        post_command: |ctx| {
            Box::pin(async move { info!("Executed Command {}...", ctx.command().qualified_name) })
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

    let token = env::var("DISCORD_TOKEN").expect("Token error");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Failed creating discord client");
    if let Err(why) = client.start().await {
        error!(
            "An Error {} has occurred whilst starting discord client",
            why
        );
    }
    client.start().await.unwrap()
}
