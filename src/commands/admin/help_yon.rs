use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{checks::rong_admin::*, data::DatabasePool};

#[command("give_yon_access")]
// Limit command usage to guilds.
#[only_in(guilds)]
#[checks(RongAdmin)]
async fn help_yon(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    sqlx::query!(
        "GRANT SELECT
         ON ALL TABLES IN SCHEMA redive_cn
         TO yon;",
    )
    .execute(&pool)
    .await?;

    sqlx::query!(
        "GRANT SELECT
         ON ALL TABLES IN SCHEMA redive_jp
         TO yon;",
    )
    .execute(&pool)
    .await?;

    sqlx::query!(
        "GRANT SELECT
         ON ALL TABLES IN SCHEMA redive_en
         TO yon;",
    )
    .execute(&pool)
    .await?;

    msg.reply(ctx, "Yon has been helped.").await?;

    Ok(())
}
