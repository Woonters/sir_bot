use poise::serenity_prelude as serenity;

use rand::seq::SliceRandom;
use serenity::prelude::Context;

use crate::{commands::say::say_saved, set_recorded_messages, Data, Error};
use log::{debug, error, info, log_enabled, Level};

pub async fn event_handler(
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
            info!("Bot has started and Bot's ID has been saved");
        }

        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            info!("Voice Event Update has triggered ");
            let new_id = new.member.as_ref().unwrap().user.id.get(); // new should always have a memeber as it's argument
            let (bot_id, c_id) = {
                let bot_id_getter = data.bot_id.lock().await;
                let channel_id_getter = data.channel_id.lock().await;
                (bot_id_getter.get(), channel_id_getter.get())
            };
            if new_id != bot_id {
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
        _ => {}
    }
    Ok(())
}
