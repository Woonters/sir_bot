use crate::{ChannelId, Error, PoiseContext};

/// Leave the current channel
#[poise::command(slash_command, prefix_command)]
pub async fn leave(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            let _ = ctx.channel_id().say(ctx, format!("Failed: {:?}", e)).await;
            log::error!("Failed to leave vc: {:?}", e);
        }

        let _ = ctx.reply("Bye bye :wave:").await;
        let mut c_id = ctx.data().channel_id.lock().await;
        *c_id = ChannelId::new(1)
    } else {
        let _ = ctx.reply("Not in a voice channel").await;
        log::info!("Tried to call leave when no bot in any vc");
    }

    Ok(())
}
