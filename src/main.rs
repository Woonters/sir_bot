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
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File},
    io::{stdin, Read, Write},
};
use toml::{self, from_str, value::Array, Value};
use urlencoding::encode;

#[group]
#[description = "All of the main commands of the bot, join and leave channels and say stuff"]
#[summary = "Generic commands"]
#[commands(about)]
struct General;

#[group]
#[prefixes("voice", "v")]
#[description = "A group of vc commands"]
#[summary = "Commands for joining and leaving vc"]
#[commands(join, leave)]
struct Voice;

#[group]
#[prefixes("say")]
#[description = "A group of commands for making Sir speak (he should be in vc)"]
#[summary = "Commands for speaking"]
#[default_command(say)]
#[commands(say, sayq)]
struct Say;

struct Handler;

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

// Functions around the channel ID for reading and writing
async fn set_channel_id(ctx: &Context, channel_id: u64) {
    let mut data = ctx.data.write().await;

    let c_id = data.get_mut::<ChannelIdChecker>().unwrap();
    *c_id = channel_id;
}

async fn get_channel_id(ctx: &Context) -> u64 {
    let data = ctx.data.read().await;
    return *data.get::<ChannelIdChecker>().unwrap();
}

async fn set_bot_id(ctx: &Context, user_id: UserId) {
    let mut data = ctx.data.write().await;
    let u_id = data.get_mut::<BotIDChecker>().unwrap();
    *u_id = user_id;
}

async fn get_bot_id(ctx: &Context) -> UserId {
    let data = ctx.data.read().await;
    return *data.get::<BotIDChecker>().unwrap();
}

// TODO: Clean up this function
// TODO: A bot command should also run this so I can update join leave messages
// set up the recorded messages into the database
async fn set_recorded_messages(ctx: &Context) {
    let mut data = ctx.data.write().await;
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
    let to_change = data.get_mut::<JoinLeaveMessageDatabase>().unwrap();
    *to_change = new_accounts;
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
        set_recorded_messages(&ctx).await;
        set_bot_id(&ctx, ready.user.id).await;
    }

    // Any time a discord channel gets an update I handel it with this
    // TODO: Move stuff into match arms and flatten a lot of this
    // TODO: Handel empty lists
    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let new_id = new.member.as_ref().unwrap().user.id.get();
        let bot_id = get_bot_id(&ctx).await;
        if new_id == bot_id.get() {
            return;
        }
        let c_id = get_channel_id(&ctx).await;
        let guild_id = new.guild_id.unwrap();
        let data = ctx.data.read().await;
        let join_leave_message_database = data.get::<JoinLeaveMessageDatabase>().unwrap();
        let personal_messages = join_leave_message_database.get(&new_id.to_string());
        let general_messages = join_leave_message_database.get("0").unwrap();
        match old {
            Some(old) if old.channel_id.unwrap().get() == c_id && new.channel_id.is_none() => {
                // the have left channel
                let file_path_array = &match personal_messages {
                    Some(v) => match rand::random::<bool>() {
                        true => &v.leave,
                        false => &general_messages.leave,
                    },
                    None => &general_messages.leave,
                };
                let file_path = file_path_array.choose(&mut rand::thread_rng());
                let _ = say_saved(&ctx, guild_id, file_path.unwrap()).await;
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
                let _ = say_saved(&ctx, guild_id, file_path.unwrap()).await;
            }
            _ => {
                return;
            }
        }
    }
}

#[help]
#[individual_command_tip = "Hello this is a command tip, please put a command after this to learn about it"]
#[command_not_found_text = "That isn't a command I can do, sorry"]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .bucket("General", BucketBuilder::default().delay(1))
        .await
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&VOICE_GROUP)
        .group(&SAY_GROUP);
    framework.configure(Configuration::new().prefix("~"));
    // that's our prefix, we look for messages with that

    let token = env::var("DISCORD_TOKEN").expect("Token error");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
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
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild_id, channel_id) = {
        let guild = msg.guild(&ctx.cache).unwrap();
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);
        (guild.id, channel_id)
    };
    set_channel_id(ctx, channel_id.unwrap().get()).await;
    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            (msg.reply(ctx, "Not in a voice channel").await.unwrap());

            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    Ok(())
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn unknown_command(ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named {unknown_command_name}");
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

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            msg.channel_id
                .say(&ctx.http, format!("Failed: {:?}", e))
                .await;
        }

        msg.channel_id.say(&ctx.http, "Left voice channel").await;

        set_channel_id(ctx, 0).await;
    } else {
        msg.reply(ctx, "Not in a voice channel").await;
    }

    Ok(())
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, r#"HELLO SIR! I AM A HALE AND HEARTY SIR, YOU CAN FIND MY CODE AT HTTPS://GITHUB.COM/FRIZBOX2000
 I say funny little gnome things, you can use... 
 `~say` to make me say something (a good tip is to use short messages ~250 characters with lots of !'s and ?'s)"#.to_string()).await?;
    Ok(())
}

#[command]
async fn say(ctx: &Context, msg: &Message) -> CommandResult {
    let content = msg.content.clone();
    let guild_id = msg.guild_id.unwrap();
    tokio::task::spawn_blocking(move || {
        let text = fix_input(&content[5..]);
        get_voice_and_save(&text);
    })
    .await
    .expect("Task Panicked");
    // ok now let's get the songbird thingy and play some audio!!!
    let mut f = File::open("audio/temp.mpeg").unwrap();
    let mut input = vec![];
    f.read_to_end(&mut input);
    if let Some(handler_lock) = songbird::get(ctx)
        .await
        .expect("Songbird Voice client not found")
        .clone()
        .get(guild_id)
    {
        let mut handler = handler_lock.lock().await;
        let _ = handler.play_input(input.into());
    }
    Ok(())
}

async fn say_saved(ctx: &Context, guild_id: GuildId, file_path: &String) -> CommandResult {
    let mut f = File::open(format!("audio/{file_path}.mpeg")).unwrap();
    let mut input = vec![];
    f.read_to_end(&mut input);
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

#[command]
async fn sayq(_ctx: &Context, msg: &Message) -> CommandResult {
    let content = msg.content.clone();
    tokio::task::spawn_blocking(move || get_voice_and_save(&content[5..]))
        .await
        .expect("Task Panicked");
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
    use super::*;

    #[test]
    fn test_voice_api() {
        let text = fix_input(&get_input());
        get_voice_and_save(&text);
    }
}
