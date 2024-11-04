use poise::serenity_prelude as serenity;

use ::serenity::all::Message;
use rand::seq::SliceRandom;
use serenity::prelude::Context;
use sqlx::Encode;

use crate::{commands::say::say_saved, set_recorded_messages, Data, Error};
use log::info;

pub async fn event_handler(
    ctx: &Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            // this arm deals with messages (for the rate bot side of operations)
            match new_message.mentions_me(ctx).await {
                Ok(true) => {
                    // the message mentions me, work out if it is a reply
                    match new_message.referenced_message {
                        Some(ref ref_message) => {
                            start_message_rating(ctx, ref_message, data).await;
                            // the referenced message needs to be targeted
                        }
                        None => {
                            start_message_rating(ctx, new_message, data).await;
                            // the message `new_message` needs to be targeted
                        }
                    }
                }
                Err(e) => log::error!("Something went wrong parsing a message '{}'", e),
                _ => {} // the message doesn't mention me so I don't need to care
            }
        }
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            info!("Voice Event Update has triggered ");
            let new_id = new.member.as_ref().unwrap().user.id; // new should always have a memeber as it's argument
            let c_id = {
                let channel_id_getter = data.channel_id.lock().await;
                channel_id_getter.get()
            };
            if new_id != _framework.bot_id {
                let guild_id = new.guild_id.expect(""); // this should always return Some
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
                        if let Some(fp) = file_path {
                            let _ = say_saved(ctx, guild_id, fp).await;
                        }
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
                        if let Some(fp) = file_path {
                            let _ = say_saved(ctx, guild_id, fp).await;
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => (),
    }
    Ok(())
}

async fn start_message_rating(ctx: &Context, msg: &Message, data: &Data) {
    // add the message to the database
    let msg_id = msg.id.get() as i64;
    let channel_id = msg.channel_id.get() as i64;
    let u_id = msg.author.id.get() as i64;
    add_to_database(&data.database, msg_id, channel_id, u_id).await;
    for reaction in &data.reactions {
        // react with all the relevant reactions
        let _ = msg.react(ctx, reaction.clone()).await;
    }
}

async fn add_to_database(database: &sqlx::SqlitePool, msg_id: i64, channel_id: i64, u_id: i64) {
    sqlx::query!(
        "INSERT OR IGNORE INTO users (user_id, cached_score) VALUES (?, NULL)",
        u_id
    )
    .execute(database)
    .await
    .unwrap();
    sqlx::query!("INSERT OR IGNORE INTO watching (message_id, channel_id, cache_score, linked_user) VALUES (?, ?, NULL, ?)", msg_id, channel_id, u_id).execute(database).await.unwrap();
}
