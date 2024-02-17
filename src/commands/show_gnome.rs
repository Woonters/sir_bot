use rand::seq::IteratorRandom;
use std::fs;

use poise::CreateReply;
use serenity::builder::CreateAttachment;

use crate::{Error, PoiseContext};

#[poise::command(prefix_command, slash_command)]
pub async fn show_gnome(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let reply = CreateReply::default()
        .content("A little gnome for me and you <3")
        .attachment(get_gnome_photo())
        .reply(true);
    ctx.send(reply).await?;
    Ok(())
}

pub fn get_gnome_photo() -> CreateAttachment {
    let mut rng = rand::thread_rng();
    // let's select a random gnome image
    let gnome_pic = fs::read(
        fs::read_dir("images/")
            .unwrap()
            .choose(&mut rng)
            .unwrap()
            .unwrap()
            .path()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    CreateAttachment::bytes(gnome_pic, "GNOME.jpg")
}
