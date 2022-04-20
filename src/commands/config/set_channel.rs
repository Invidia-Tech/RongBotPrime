use crate::{
    checks::rong_admin::*,
    data::{ChannelPersona, ChannelType, DatabasePool},
};

use std::str::FromStr;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    utils::parse_channel,
};

#[command("channel")]
#[description(
    "Allows you to set the permissions of a channel within Rong.\n\n\
               Usage: >config channel <#channel> \"<clan name>\" <channel type>\n\n\
               Please use quotes around the clan name if it contains spaces. \
               channel type can be one of: cb, public, clan, pvp\n\n\
               Example: >config channel #cb-reports \"Invidia\" cb"
)]
#[checks(RongAdmin)]
async fn set_channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() != 3 {
        msg.channel_id
            .say(
                ctx,
                "Invalid command usage. Please use the help command on `config channel`.",
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
    let mentioned_ch = match parse_channel(args.single::<String>().unwrap()) {
        Some(ch) => ch,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Expecting a mention of a channel.")
                .await?;
            return Ok(());
        }
    };

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
    let p = args.single::<String>().unwrap().to_lowercase();
    let persona = p.as_str();
    match ChannelPersona::from_str(persona) {
        Ok(persona_enum) => {
            // println!("Channel: {}, clan_id: {}, persona: {:?}", mentioned_ch, clan_id, persona_enum);
            sqlx::query_as_unchecked!(
                ChannelType,
                "INSERT INTO rongbot.channel_type (channel_id, clan_id, persona)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (channel_id, clan_id)
                 DO UPDATE SET persona = $3;",
                mentioned_ch.to_string(),
                clan_id,
                persona_enum
            )
            .execute(&pool)
            .await?;
            msg.channel_id
                .say(
                    ctx,
                    &format!("<#{}> has been updated to {}.", mentioned_ch, persona),
                )
                .await?;
            // println!("Channel ID: {} has been updated to {:?}.",
            //          ch_type.channel_id,
            //          ch_type.persona);
        }
        Err(_) => {
            msg.channel_id.say(ctx, "Invalid channel type.").await?;
        }
    }

    Ok(())
}
