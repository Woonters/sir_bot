use crate::{Error, PoiseContext, TrackEvent};
use poise::serenity_prelude as serenity;
use songbird::{events::EventHandler as VoiceEventHandler, Event, EventContext};
/// Join the Users current Voice chat
#[poise::command(slash_command, prefix_command)]
pub async fn join(ctx: PoiseContext<'_>) -> Result<(), Error> {
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

#[::serenity::async_trait]
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
