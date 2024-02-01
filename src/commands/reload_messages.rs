use crate::{Error, PoiseContext};
#[poise::command(slash_command)]
pub async fn reload_join_leave_messages(ctx: PoiseContext<'_>) -> Result<(), Error> {
    crate::set_recorded_messages(&ctx.data()).await;
    Ok(())
}
