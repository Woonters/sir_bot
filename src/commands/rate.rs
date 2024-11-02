use std::collections::HashMap;

use serenity::all::{
    ArgumentConvert, CacheHttp, ChannelId, Message, ReactionType, ReactionTypes, UserId,
};
use sqlx;

use crate::{Error, PoiseContext};
use poise::CreateReply;

#[poise::command(prefix_command, slash_command)]
pub async fn rate_me(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let reply =
        CreateReply::default().content(format!("You have a score of {}", get_score(ctx).await));
    ctx.send(reply).await?;
    Ok(())
}

async fn get_score(ctx: PoiseContext<'_>) -> f32 {
    // get the user's messages
    let messages = get_user_messages(ctx).await;
    let mut counter = 0;
    let mut score = 0.0;

    for message in messages {
        let to_add = get_message_score(message, &ctx.data().reactions);
        score += to_add;
        if to_add != 0.0 {
            counter += 1;
        }
    }
    if counter == 0 {
        return 0.0;
    }
    score / (counter as f32)
}

async fn get_user_messages(ctx: PoiseContext<'_>) -> Vec<Message> {
    // this should be a list of all id's of all the messages from the user
    let uid = ctx.author().id.get() as i64;
    let search = sqlx::query!("SELECT * FROM watching WHERE linked_user == (?)", uid)
        .fetch_all(&ctx.data().database)
        .await
        .unwrap();
    // for each message_id get the message related
    // this is actually kinda hard and uses a function I don't realy like
    let mut out: Vec<Message> = Vec::new();
    for message in search {
        let message_id = format!("{}", message.message_id);
        let msg = Message::convert(
            ctx,
            ctx.guild_id(),
            Some(ChannelId::new(message.channel_id as u64)),
            &message_id,
        )
        .await;
        match msg {
            Ok(good_message) => out.push(good_message),
            Err(_) => {
                todo!()
            }
        }
    }
    out
}

fn get_message_score(message: Message, reactions: &[ReactionType]) -> f32 {
    let reacts = get_valid_message_reacts(message, reactions);
    // currently we will just calculate the mean for each message
    // this might change in the future for doing better analytics
    let mut total = 0;
    let mut number = 0;
    for (rate, count) in reacts {
        total += rate;
        number += count;
    }
    if number == 0 {
        return 0.0;
    }
    (total as f32) / (number as f32)
}

// TODO: This function took a bit of time to write trying to get unicode_partial_cmp to work and binary_search_by working as well
// come back and rewrite it / clean it up later
// this uses a binary to find the correct emoji, frankly for such a small list it's a bit silly and with it being based on unicode
// and using that for the ordering and index as a simple conversion for what value it is there are a lot of things that could go wrong
fn get_valid_message_reacts(message: Message, reactions: &[ReactionType]) -> HashMap<usize, u64> {
    let msg_reactions = message.reactions;
    let mut out: HashMap<usize, u64> = HashMap::new();
    for reaction in msg_reactions {
        let index = match reaction.reaction_type {
            ReactionType::Unicode(unicode) => Some(
                reactions.binary_search_by(|probe| probe.unicode_partial_cmp(&unicode).unwrap()),
            ),
            _ => None,
        };
        if let Some(Ok(value)) = index {
            out.insert(value, reaction.count - 1);
        }
    }
    out
}
