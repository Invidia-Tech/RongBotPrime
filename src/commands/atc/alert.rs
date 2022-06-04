use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
    utils::parse_channel,
};

use crate::{
    checks::rong_admin::*,
    data::DatabasePool,
};

#[command("atc_alert_channel")]
#[aliases("alert_ch", "set_alert_ch")]
#[description(
    "Changes the alert channel.
	 Use with >atc set_alert_ch guildname #mention_channel"
)]
#[checks(RongAdmin)]
#[bucket = "atc"]
async fn set_alert_channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() != 2 {
        msg.channel_id
            .say(
                ctx,
                "Invalid command usage. Please use the help command on `atc alert_ch`.",
            )
            .await?;
        return Ok(());
    }

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let clan_name = args.single_quoted::<String>().unwrap();

    let clan_id = match sqlx::query!(
        "SELECT id
             FROM public.rong_clan
             WHERE name ilike $1",
        clan_name
    )
    .fetch_one(&pool)
    .await
    {
        Ok(id) => id.id,
        Err(_) => {
            msg.channel_id
                .say(ctx, format!("I cannot find a clan named {}", clan_name))
                .await?;
            return Ok(());
        }
    };
    let mentioned_ch = match parse_channel(args.single::<String>().unwrap()) {
        Some(ch) => ch,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Expecting a mention of a channel.")
                .await?;
            return Ok(());
        }
    };

    sqlx::query!(
        "INSERT INTO rongbot.atc_config (clan_id, alert_channel)
		 VALUES ($1, $2)
		 ON CONFLICT (clan_id)
         DO UPDATE SET alert_channel = $2;",
        clan_id,
        mentioned_ch.to_string()
    )
    .execute(&pool)
    .await?;

    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "<#{}> has been set to the atc alert channel for {}",
                mentioned_ch, clan_name
            ),
        )
        .await?;

    Ok(())
}
