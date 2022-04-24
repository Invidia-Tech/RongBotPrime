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

    let pilot_user_id =
        result_or_say_why!(get_user_id(ctx, msg, &msg.author.id.to_string()), ctx, msg);

    let pilot_info = result_or_say_why!(
        get_pilot_info_or_create_new(ctx, &pilot_user_id, &clan_id),
        ctx,
        msg
    );

    match args.len() {
        0 | 1 => {
            // Get passenger or determine solo flight
            let passenger_member_id: Option<i32>;
            if !args.is_empty() {
                let ign = args.single::<String>().unwrap();
                let (member_id, passenger_user_id) =
                    result_or_say_why!(get_clan_member_id_by_ign(ctx, &clan_id, &ign), ctx, msg);
                // msg.channel_id
                //    .say(ctx, format!("Passenger_member_id is: {:?}", passenger_member_id))
                //    .await?;
                passenger_member_id = Some(member_id);
                if passenger_user_id == pilot_user_id {
                    msg.channel_id
                       .say(ctx, "Warning! You mentioned yourself! This flight will start assuming this is not a solo flight. This may mess up pilot stats.")
                       .await?;
                }
            } else {
                passenger_member_id = None;
                msg.channel_id
                    .say(ctx, "No passenger! Starting a solo flight.")
                    .await?;
            }

            // Ensure that the passenger_member_id is within the same guild as the pilot.

            let pilot_ongoing_flights = result_or_say_why!(
                get_pilot_ongoing_flights(ctx, &pilot_info.id, &clan_id, &cb_info.id),
                ctx,
                msg
            );

            for flight in pilot_ongoing_flights {
                if flight.passenger_id == passenger_member_id {
                    msg.channel_id
                        .say(
                            ctx,
                            "Warning! You already have an ongoing flight with this passenger!",
                        )
                        .await?;
                    return Ok(());
                }
            }

            // Ask the user if they're ready to start.
            let react_msg = msg
                .reply(ctx, "Are you ready to start your flight?")
                .await
                .unwrap();
            react_msg.react(ctx, '✅').await?;
            react_msg.react(ctx, '❌').await?;
            // The message model has a way to collect reactions on it.
            // Other methods are `await_n_reactions` and `await_all_reactions`.
            // Same goes for messages!
            if let Some(reaction) = &react_msg
                .await_reaction(&ctx)
                .timeout(Duration::from_secs(10))
                .author_id(msg.author.id)
                .await
            {
                // By default, the collector will collect only added reactions.
                // We could also pattern-match the reaction in case we want
                // to handle added or removed reactions.
                // In this case we will just get the inner reaction.
                let emoji = &reaction.as_inner_ref().emoji;

                match emoji.as_data().as_str() {
                    "✅" => {
                        let pool = ctx
                            .data
                            .read()
                            .await
                            .get::<DatabasePool>()
                            .cloned()
                            .unwrap();

                        let all_flights = result_or_say_why!(
                            get_all_pilot_flights(ctx, &pilot_info.id, &clan_id, &cb_info.id),
                            ctx,
                            msg
                        );

                        sqlx::query!(
                            "INSERT INTO rongbot.flight
                                (call_sign, pilot_id, clan_id, cb_id,
                                 passenger_id, start_time, status)
                             VALUES ($1, $2, $3, $4, $5, now(), 'in flight');",
                            all_flights.len().to_string(),
                            pilot_info.id,
                            clan_id,
                            cb_info.id,
                            passenger_member_id,
                        )
                        .execute(&pool)
                        .await?;
                        msg.reply(ctx, "Taking off! Have a safe flight captain! <:KyaruAya:966980880939749397>")
                            .await?
                    }
                    _ => msg.reply(ctx, "Deboarding your plane...").await?,
                };
            } else {
                msg.reply(ctx, "Timed out. We've deboarded your plane.")
                    .await?;
            };
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
