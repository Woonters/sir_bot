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
    .unwrap(); // this is somehow genuinely the way to do this, or atleast one way
               // how many unwraps does one function need!
               // I'm going to just hope it works every time and not write my own logging stuff or
               // deal with Errs for this function as I really don't think anyone should need to. Effectively
               // only way I can see this going is if some file can't be read, or a file is edited whilst this
               // function is executing. The former is much more likely but still unlikely
    CreateAttachment::bytes(gnome_pic, "GNOME.jpg")
}
