use std::collections::HashSet;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{
    data::{CbStatus, ChannelPersona, DatabasePool, Flight},
    utils::{atc::*, clan::*, macros::*, rong::*},
};

#[command("atc_note")]
#[aliases("note")]
#[description(
    "Set a note for an active flight. \
     Lim: 50 characters.\n\
     Use \"self\" for solo flight.\n\
     **This only works on active flights**.\n\
     `\t>atc note Dabo C31 - 5.6M`\n\
     `\t>atc note Self I died to crit bruh`\n\
     `\t>atc note Ring C50 - Need amb`\n"
)]
#[bucket = "atc"]
async fn set_note(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() < 2 {
        msg.channel_id
            .say(ctx, "Invalid command usage. Please check `>help atc note`.")
            .await?;
        return Ok(());
    }

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
                        "Warning, there is no currently active CB!\n\
                        {clan_name} - {name}. \
                        {name} started <t:{start_epoch}:R> and ended <t:{end_epoch}:R>.",
                        clan_name = clan_name,
                        name = cb_info.name,
                        start_epoch = cb_info.start_time.unwrap().timestamp(),
                        end_epoch = cb_info.end_time.unwrap().timestamp()
                    ),
                )
                .await?;
            // return Ok(());
        }
        _ => (),
    };

    let pilot_user_id =
        result_or_say_why!(get_user_id(ctx, msg, &msg.author.id.to_string()), ctx, msg);

    let pilot_info = result_or_say_why!(
        get_pilot_info_or_create_new(ctx, &pilot_user_id, &clan_id),
        ctx,
        msg
    );

    let ign = args.single::<String>().unwrap().to_lowercase();
    let note: &str = args.rest();
    // Max 50 length for db.
    if note.len() > 50 {
        msg.reply(ctx, "This note is too long! Lim: 50 characters.")
            .await?;
        return Ok(());
    }
    // Double checking note len max.
    let char_blacklist: HashSet<char> = HashSet::from(['<', '>', '@', '/', '\\']);
    for c in note.chars() {
        if char_blacklist.contains(&c) {
            msg.reply(ctx, "This note contains illegal characters!")
                .await?;
            return Ok(());
        }
    }

    let passenger_member_id;
    if ign != "self" {
        let (member_id, p_user_id) =
            result_or_say_why!(get_clan_member_id_by_ign(ctx, &clan_id, &ign), ctx, msg);
        if p_user_id == pilot_user_id {
            // User somehow mentioned themselves
            passenger_member_id = None;
        } else {
            passenger_member_id = Some(member_id);
        }
    } else {
        passenger_member_id = None;
    }

    // Ensure that the passenger_member_id is within the same guild as the pilot.
    let pilot_ongoing_flights = result_or_say_why!(
        get_pilot_ongoing_flights(ctx, &pilot_info.id, &clan_id, &cb_info.id),
        ctx,
        msg
    );

    let mut active_flight_to_update: Option<Flight> = None;
    // Now we much make sure that the flight in question in an ongoing flight by the pilot.
    for flight in pilot_ongoing_flights {
        if flight.passenger_id == passenger_member_id {
            active_flight_to_update = Some(flight);
            break;
        }
    }

    match active_flight_to_update {
        None => {
            msg.reply(ctx, "You do not have an active flight with this passenger.")
                .await?;
            return Ok(());
        }
        Some(flight) => {
            let pool = ctx
                .data
                .read()
                .await
                .get::<DatabasePool>()
                .cloned()
                .unwrap();
            match sqlx::query_unchecked!(
                "UPDATE rongbot.flight SET note =
	        	    $1 WHERE id = $2",
                note,
                flight.id
            )
            .execute(&pool)
            .await
            {
                Ok(_) => {
                    msg.react(ctx, '✅').await?;
                }
                Err(e) => {
                    msg.reply(ctx, format!("Something's wrong! {}", e)).await?;
                    return Ok(());
                }
            };
        }
    }

    Ok(())
}
