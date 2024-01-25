use poise::serenity_prelude as serenity;
use poise::Command;
use rand::seq::SliceRandom;
use reqwest::get;
use serde::Deserialize;
use serenity::{
    all::{ChannelId, GuildId, Ready, UserId, VoiceState},
    async_trait,
    client::Context,
    framework::standard::{
        buckets::{LimitedFor, RevertBucket},
        help_commands,
        macros::{command, group, help, hook},
        Args, BucketBuilder, CommandGroup, CommandResult, Configuration, HelpOptions,
        StandardFramework,
    },
    model::channel::Message,
    prelude::*,
};
use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent},
    SerenityInit,
};
use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::Duration;
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File},
    io::{stdin, Read, Write},
};
use toml::{self, from_str, value::Array, Value};
use urlencoding::encode;

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
struct ChannelIdChecker;
impl TypeMapKey for ChannelIdChecker {
    type Value = u64;
}

struct BotIDChecker;
impl TypeMapKey for BotIDChecker {
    type Value = UserId;
}

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

async fn event_handler(
    ctx: &Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            set_recorded_messages(data).await;
            let mut write_bot_id = data.bot_id.lock().await;
            *write_bot_id = data_about_bot.user.id;
        }

        // Little note I seem to be getting a message:
        //Track b180e8b2-b52d-45ca-b044-34554bd854bd encountered an error: Errored(Parse(IoError(Custom { kind: UnexpectedEof, error: "end of stream" })))
        // which might mean that `say_saved` is broken in some way, it should be a carbon copy of say, but idk
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            println!("Voice Event Update has triggered ");
            let new_id = new.member.as_ref().unwrap().user.id.get();
            let (bot_id, c_id) = {
                let bot_id_getter = data.bot_id.lock().await;
                let channel_id_getter = data.channel_id.lock().await;
                (bot_id_getter.get(), channel_id_getter.get())
            };
            if new_id != bot_id {
                let guild_id = new.guild_id.unwrap();
                let join_leave_message_database = data.join_leave_message_database.lock().await;
                let personal_messages = join_leave_message_database.get(&new_id.to_string());
                let general_messages = join_leave_message_database.get("0").unwrap();
                match old {
                    Some(old)
                        if old.channel_id.unwrap().get() == c_id && new.channel_id.is_none() =>
                    {
                        // the have left channel
                        let file_path_array = &match personal_messages {
                            Some(v) => match rand::random::<bool>() {
                                true => &v.leave,
                                false => &general_messages.leave,
                            },
                            None => &general_messages.leave,
                        };
                        let file_path = file_path_array.choose(&mut rand::thread_rng());
                        println!("I should say that they have left!");
                        let _ = say_saved(ctx, guild_id, file_path.unwrap()).await;
                    }
                    None if c_id == new.channel_id.unwrap().get() => {
                        // account has joined a channel
                        let file_path_array = &match personal_messages {
                            Some(v) => match rand::random::<bool>() {
                                true => &v.join,
                                false => &general_messages.join,
                            },
                            None => &general_messages.join,
                        };
                        let file_path = file_path_array.choose(&mut rand::thread_rng());
                        println!("I should say they have joined");
                        let _ = say_saved(ctx, guild_id, file_path.unwrap()).await;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: PoiseContext<'_>,
    #[description = "Specific Command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    let _ = poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "I am a silly little bot, please don't abuse me :)",
            ..Default::default()
        },
    )
    .await;
    Ok(())
}

#[tokio::main]
async fn main() {
    let options = poise::FrameworkOptions {
        commands: vec![about(), join(), say(), leave(), help()],
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
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };
    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(667311502650376192),
                )
                .await?;
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
    {
        let mut data = client.data.write().await;
        data.insert::<ChannelIdChecker>(0);
        data.insert::<JoinLeaveMessageDatabase>(HashMap::new());
    }
    if let Err(why) = client.start().await {
        println!(
            "An Error {} has occurred whilst starting discord client",
            why
        );
    }
    client.start().await.unwrap()
}

/// Join the Users current Voice chat
#[poise::command(slash_command, prefix_command)]
async fn join(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();
        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);
        (guild.id, channel_id)
    };
    let mut c_id = ctx.data().channel_id.lock().await;
    *c_id = channel_id.unwrap();

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            (ctx.reply("Not in a voice channel").await.unwrap());
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }
    let channel_name = channel_id.unwrap().name(ctx).await.unwrap();
    let _ = ctx
        .reply(format!("Joining the Channel {}", channel_name))
        .await;
    Ok(())
}
struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}
/// Leave the current channel
#[poise::command(slash_command, prefix_command)]
async fn leave(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            let _ = ctx.channel_id().say(ctx, format!("Failed: {:?}", e)).await;
        }

        let _ = ctx.channel_id().say(ctx, "Left voice channel").await;
        let mut c_id = ctx.data().channel_id.lock().await;
        *c_id = ChannelId::new(1)
    } else {
        let _ = ctx.reply("Not in a voice channel").await;
    }

    Ok(())
}

/// A simple example command just describes what the bot can do and who the cheeky little gnome is :)
#[poise::command(prefix_command, slash_command)]
pub async fn about(ctx: PoiseContext<'_>) -> Result<(), Error> {
    println!("Test");
    let _ = ctx.reply("I am a little gnome, who wants to help you sir! \n A TTS bot using novel ai's aHaleAndHeartySir \n Idea from the RTVS group (look them up), you can find my code at https://github.com/Fritzbox2000/sir_bot").await;
    Ok(())
}

/// Say Something with the TTS
#[poise::command(prefix_command, slash_command, aliases("s"))]
async fn say(
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
    let _ = f.read_to_end(&mut input);
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

async fn say_saved(ctx: &Context, guild_id: GuildId, file_path: &String) -> CommandResult {
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
