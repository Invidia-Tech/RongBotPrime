use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

use std::time::Duration;

use crate::{
    data::{
        CbStatus,
        ChannelPersona,
        DatabasePool,
    },
    utils::{
        atc::*,
        clan::*,
        macros::*,
        rong::*,
    },
};

#[command("atc_call_sign")]
#[aliases("callsign", "call_sign")]
#[description("Sets your pilot call sign.")]
#[bucket = "atc"]
async fn set_call_sign(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let MAX_CALL_SIGN_SIZE = 4;

    if args.len() != 1 {
        msg.channel_id
            .say(
                ctx,
                "Invalid command usage. Please use the help command on `atc alert_ch`.",
            )
            .await?;
        return Ok(());
    }

    let (clan_id, clan_name) = result_or_say_why!(
        get_clan_from_channel_context(ctx, msg, ChannelPersona::Cb),
        ctx,
        msg
    );

    let pilot_user_id =
        result_or_say_why!(get_user_id(ctx, msg, &msg.author.id.to_string()), ctx, msg);

    let mut pilot_info = result_or_say_why!(
        get_pilot_info_or_create_new(ctx, &pilot_user_id, &clan_id),
        ctx,
        msg
    );

    let mut new_call_sign = args.single_quoted::<String>().unwrap();
    new_call_sign.make_ascii_uppercase();

    if new_call_sign.chars().count() > MAX_CALL_SIGN_SIZE {
        msg.reply(
            ctx,
            format!(
                "Sorry, the max allowed call sign length is {}.",
                &MAX_CALL_SIGN_SIZE
            ),
        )
        .await?;
        return Ok(());
    }

    pilot_info.code = Some(new_call_sign);

    let _ = result_or_say_why!(update_pilot_info(ctx, &pilot_info), ctx, msg);

    msg.reply(
        ctx,
        format!(
            "We've updated your call sign to {}",
            &pilot_info.code.unwrap()
        ),
    )
    .await?;

    Ok(())
}
