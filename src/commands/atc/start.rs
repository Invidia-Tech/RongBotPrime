use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    utils::parse_username,
};
use serenity::model::prelude::UserId;

use crate::{
    data::{CbStatus, ChannelPersona},
    utils::{atc::*, clan::*, macros::*, rong::*},
};

#[command("atc_start")]
#[aliases("start")]
#[description(
    "Mention a user to take off with them as your passenger.
     Otherwise, mention no one and take off by yourself."
)]
#[bucket = "atc"]
async fn flight_start(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Only allows this command within CB marked channels.
    let (clan_id, clan_name) = result_or_say_why!(
        get_clan_from_channel_context(ctx, msg, ChannelPersona::Cb),
        ctx,
        msg
    );

    let (cb_info, cb_status) =
        result_or_say_why!(get_latest_cb(ctx, &clan_id, &clan_name), ctx, msg);

    match cb_status {
        CbStatus::Past | CbStatus::Future => {
            msg.channel_id
                .say(
                    ctx,
                    format!(
                        "You cannot take off without an active CB!
                        {clan_name} - {name} is already over. \
                        {name} started <t:{start_epoch}:R> and ended <t:{end_epoch}:R>.",
                        clan_name = clan_name,
                        name = cb_info.name,
                        start_epoch = cb_info.start_time.unwrap().timestamp(),
                        end_epoch = cb_info.end_time.unwrap().timestamp()
                    ),
                )
                .await?;
            return Ok(());
        }
        _ => (),
    };

    let user_id = result_or_say_why!(get_user_id(ctx, msg, &msg.author.id.to_string()), ctx, msg);

    let pilot_info = result_or_say_why!(
        get_pilot_info_or_create_new(ctx, &user_id, &clan_id),
        ctx,
        msg
    );

    match args.len() {
        0 | 1 => {
            // Get passenger or determine solo flight
            let passenger: Option<i32>;
            if !args.is_empty() {
                if let ign = args.single::<String>().unwrap() {
                    if false {// UserId(passenger_id) == msg.author.id {
                        passenger = None;
                        msg.channel_id
                           .say(ctx, "No need to mention yourself! Starting a solo flight.")
                           .await?;
                    } else {
                        passenger =
                            Some(
                                result_or_say_why!(get_clan_member_id(ctx, &clan_id, &ign), ctx, msg)
                            );
                        msg.channel_id
                           .say(ctx, format!("Passenger is: {:?}", passenger))
                           .await?;
                    }
                } else {
                    msg.channel_id
                        .say(ctx, "Please mention a valid passenger!")
                        .await?;
                    return Ok(());
                }
            } else {
                passenger = None;
                msg.channel_id
                    .say(ctx, "No passenger! Starting a solo flight.")
                    .await?;
            }

            // Ensure that the passenger is within the same guild as the pilot.
            // if passenger.

            let pilot_ongoing_flights = result_or_say_why!(
                get_ongoing_flights(ctx, &pilot_info.id, &clan_id, &cb_info.id),
                ctx,
                msg
            );
        }
        _ => {
            msg.channel_id
                .say(
                    ctx,
                    "Invalid use, please use the help command to find the right use.",
                )
                .await?;
            return Ok(());
        }
    }

    Ok(())
}
