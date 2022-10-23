use std::collections::HashSet;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{
    data::ChannelPersona,
    utils::{atc::*, clan::*, macros::*, rong::*},
};

#[command("atc_motto")]
#[aliases("motto", "notice")]
#[description(
    "Set your motto for other pilots to know!\n\
     >atc motto \"Please don't int my account!\" \
     to set a motto for you. Other pilots will know \
     about when they pilot you next."
)]
#[bucket = "atc"]
async fn set_motto(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() != 1 {
        msg.channel_id
            .say(
                ctx,
                "Invalid command usage. Please use the help command on `atc motto`.",
            )
            .await?;
        return Ok(());
    }

    let (clan_id, _clan_name) = result_or_say_why!(
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

    let new_motto = args.single_quoted::<String>()?;
    let char_blacklist: HashSet<char> = HashSet::from(['<', '>', '@', '/', '\\']);
    for c in new_motto.as_str().chars() {
        if char_blacklist.contains(&c) {
            msg.reply(ctx, "I can't set that as your motto...").await?;
            return Ok(());
        }
    }
    println!("Check assed, setting motto as: {}", new_motto);

    pilot_info.motto = Some(new_motto);

    let _ = result_or_say_why!(update_pilot_info(ctx, &pilot_info), ctx, msg);

    msg.reply(
        ctx,
        format!(
            "We've updated your motto to \"{}\"",
            &pilot_info.motto.unwrap()
        ),
    )
    .await?;

    Ok(())
}
