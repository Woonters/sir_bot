use crate::{Error, PoiseContext};
use poise::serenity_prelude as serenity;

/// A simple example command just describes what the bot can do and who the cheeky little gnome is :)
#[poise::command(prefix_command, slash_command)]
pub async fn about(ctx: PoiseContext<'_>) -> Result<(), Error> {
    println!("Test");
    let _ = ctx.reply("I am a little gnome, who wants to help you sir! \n A TTS bot using novel ai's aHaleAndHeartySir \n Idea from the RTVS group (look them up), you can find my code at https://github.com/Fritzbox2000/sir_bot").await;
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
