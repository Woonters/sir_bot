use poise::serenity_prelude as serenity;

use rand::seq::SliceRandom;
use serenity::prelude::Context;

use crate::{commands::say::say_saved, set_recorded_messages, Data, Error};

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
