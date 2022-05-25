use chrono::Utc;
use serenity::{
    builder::{
        CreateActionRow,
        CreateButton,
        CreateComponents,
    },
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    futures::TryFutureExt,
    model::{
        channel::Message,
        interactions::message_component::ButtonStyle,
    },
};

use core::fmt;
use std::{
    str::FromStr,
    time::Duration,
};

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

#[derive(Debug, strum_macros::EnumString)]
enum StartOptions {
    #[strum(ascii_case_insensitive)]
    Yes,
    #[strum(ascii_case_insensitive)]
    No,
}

impl fmt::Display for StartOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yes => write!(f, "Yes"),
            Self::No => write!(f, "No"),
        }
    }
}

impl StartOptions {
    pub fn emoji(&self) -> char {
        match self {
            Self::Yes => '✅',
            Self::No => '❌',
        }
    }

    fn button(&self) -> CreateButton {
        let mut b = CreateButton::default();
        b.custom_id(self);
        b.emoji(self.emoji());
        b.label(self);
        b.style(ButtonStyle::Primary);
        b
    }

    pub fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        // We can add up to 5 buttons per action row
        ar.add_button(StartOptions::Yes.button());
        ar.add_button(StartOptions::No.button());
        ar
    }
}

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

    let all_pilot_ign_map = result_or_say_why!(get_all_pilot_ign_map(ctx, &clan_id), ctx, msg);

    let cb_start_epoch = cb_info.start_time.unwrap().timestamp();

    let epoch_now = Utc::now().timestamp();
    let cb_day: i32 = ((epoch_now - cb_start_epoch) / 86400 + 1)
        .try_into()
        .unwrap();

    match args.len() {
        0 | 1 => {
            let pool = ctx
                .data
                .read()
                .await
                .get::<DatabasePool>()
                .cloned()
                .unwrap();

            let mut pre_flight_alerts: String = "".to_string();
            // Get passenger or determine solo flight
            let passenger_member_id: Option<i32>;
            let mut ign = "Self Flight".to_string();
            if !args.is_empty() {
                ign = args.single::<String>().unwrap();
                let (member_id, passenger_user_id) =
                    result_or_say_why!(get_clan_member_id_by_ign(ctx, &clan_id, &ign), ctx, msg);
                // msg.channel_id
                //    .say(ctx, format!("Passenger_member_id is: {:?}", passenger_member_id))
                //    .await?;
                passenger_member_id = Some(member_id);
                if passenger_user_id == pilot_user_id {
                    pre_flight_alerts.push_str(
                        "Warning! You mentioned yourself! \
                         This flight will start assuming this is not a solo flight. \
                         This may mess up pilot stats.\n",
                    );
                }
            } else {
                passenger_member_id = None;
                pre_flight_alerts.push_str("No passenger! Starting a solo flight.\n");
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
                            "Warning! You already have an ongoing flight with this passenger! I've canceled this flight.",
                        )
                        .await?;
                    return Ok(());
                }
            }

            let conflicting_flights;
            // Ensure that the self flight or passenger is not already in the air by ANOTHER pilot.
            if let Some(passenger_clanmember_id) = passenger_member_id {
                let passenger_pilot_id = match sqlx::query!(
                    "SELECT p.id FROM rongbot.pilot p
                     JOIN rong_clanmember cm
                        ON p.clan_id = cm.clan_id AND
                           p.user_id = cm.user_id
                     WHERE cm.id = $1;",
                    &passenger_clanmember_id
                )
                .fetch_one(&pool)
                .await
                {
                    Ok(row) => row.id,
                    _ => -1,
                };

                conflicting_flights = match sqlx::query!(
                    "SELECT COUNT(*) as count
                     FROM rongbot.flight
                     WHERE status = 'in flight' AND
                           ((pilot_id = $1 AND
                             passenger_id is NULL) OR
                            (passenger_id = $2));",
                    &passenger_pilot_id,
                    &passenger_clanmember_id
                )
                .fetch_one(&pool)
                .await
                {
                    Ok(row) => row.count.unwrap_or(0),
                    Err(_) => {
                        msg.channel_id
                            .say(
                                ctx,
                                "There is an error in getting your conflicting flights.",
                            )
                            .await?;
                        return Ok(());
                    }
                };
            } else {
                let pilot_cm_id = match sqlx::query!(
                    "SELECT id FROM rong_clanmember
                             WHERE clan_id = $1 AND
                                   user_id = $2;",
                    &clan_id,
                    &pilot_info.user_id
                )
                .fetch_one(&pool)
                .await
                {
                    Ok(row) => row.id,
                    Err(_) => {
                        msg.channel_id
                            .say(ctx, "There was as error matching pilot info.")
                            .await?;
                        return Ok(());
                    }
                };

                conflicting_flights = match sqlx::query!(
                    "SELECT COUNT(*) as count
                     FROM rongbot.flight
                     WHERE status = 'in flight' AND
                           passenger_id = $1;",
                    &pilot_cm_id
                )
                .fetch_one(&pool)
                .await
                {
                    Ok(row) => row.count.unwrap_or(0),
                    Err(_) => {
                        msg.channel_id
                            .say(
                                ctx,
                                "There is an error in getting your conflicting flights.",
                            )
                            .await?;
                        return Ok(());
                    }
                };
            }

            if conflicting_flights > 0 {
                msg.reply_ping(
                    ctx,
                    "WARNING!! THERE IS A CONFLICTING FLIGHT BY ANOTHER PILOT!!\n\
                     I cannot start this flight.",
                )
                .await?;
                return Ok(());
            }

            let m = msg
                .channel_id
                .send_message(&ctx, |m| {
                    m.content(format!(
                        "{}Are you ready to begin your flight?",
                        &pre_flight_alerts
                    ))
                    .components(|c| c.add_action_row(StartOptions::action_row()))
                })
                .await
                .unwrap();

            // Wait for the user to make a selection
            let mci = match m
                .await_component_interaction(&ctx)
                .timeout(Duration::from_secs(20))
                .await
            {
                Some(ci) => ci,
                None => {
                    msg.reply(&ctx, "Timed out.").await.unwrap();
                    m.delete(&ctx).await.unwrap();
                    return Ok(());
                }
            };

            let selected_option: StartOptions = StartOptions::from_str(&mci.data.custom_id)?;
            match selected_option {
                StartOptions::Yes => {
                    let all_flights = result_or_say_why!(
                        get_all_pilot_flights(ctx, &pilot_info.id, &clan_id, &cb_info.id),
                        ctx,
                        msg
                    );

                    let flight_id = match sqlx::query!(
                        "INSERT INTO rongbot.flight
                                (call_sign, pilot_id, clan_id, cb_id,
                                 passenger_id, start_time, status)
                             VALUES ($1, $2, $3, $4, $5, now(), 'in flight') RETURNING id;",
                        (all_flights.len() + 1).to_string(),
                        pilot_info.id,
                        clan_id,
                        cb_info.id,
                        passenger_member_id,
                    )
                    .fetch_one(&pool)
                    .await
                    {
                        Ok(row) => row.id,
                        Err(e) => {
                            msg.channel_id
                                .say(ctx, format!("Something went wrong! {}", e))
                                .await?;
                            return Ok(());
                        }
                    };
                    m.delete(ctx).await?;
                    let mut m = msg.reply(
                        ctx,
                        "Taking off! Have a safe flight captain! <:KyaruAya:966980880939749397>",
                    )
                    .await?;

                    let mut alert_msg = String::default();
                    // Alert the passenger if exists;
                    let mention_ch =
                        result_or_say_why!(get_alert_channel_for_clan(ctx, &clan_id), ctx, msg);

                    if mention_ch == None {
                        alert_msg.push_str("Warning! I cannot notify your passenger as the ATC alert channel is not set!\n");
                    }

                    if let Some(clanmember_id) = passenger_member_id {
                        let used_fc: i64 = match sqlx::query!(
                            "SELECT COUNT (*) as count FROM rongbot.force_quit
                             WHERE clanmember_id = $1 AND
                             cb_id = $2 AND
                             day_used = $3;",
                            &clanmember_id,
                            &cb_info.id,
                            &cb_day
                        )
                        .fetch_one(&pool)
                        .await
                        {
                            Ok(row) => row.count.unwrap_or(0),
                            Err(_) => {
                                msg.channel_id
                                    .say(ctx, "There is a problem getting FCs.")
                                    .await?;
                                return Ok(());
                            }
                        };

                        if used_fc > 0 {
                            alert_msg += "\n**WARNING, YOUR PASSENGER DOES NOT HAVE FC**";
                        }

                        match result_or_say_why!(
                            get_clanmember_mention_from_id(ctx, &clanmember_id),
                            ctx,
                            msg
                        ) {
                            None => return Ok(()),
                            Some(p_mention) => {
                                let out_msg = format!(
                                    "{}\n{}\nTime to ping them... <:KyoukaGiggle:968707085212745819>",
                                    &m.content, &alert_msg
                                );

                                let _ = &m.edit(ctx, |nm| nm.content(out_msg)).await?;

                                let channel = match ctx
                                    .cache
                                    .guild_channel(mention_ch.unwrap_or(msg.channel_id.0))
                                {
                                    Some(channel) => channel,
                                    None => {
                                        let result = msg
                                            .channel_id
                                            .say(ctx, "The ATC alert channel is misconfigured.")
                                            .await;
                                        if let Err(why) = result {
                                            println!("Error sending message: {:?}", why);
                                        }

                                        ctx.cache.guild_channel(msg.channel_id.0).unwrap()
                                    }
                                };
                                let default_no_ign = "No IGN".to_string();
                                let pilot_name = format!(
                                    "{}",
                                    all_pilot_ign_map
                                        .get(&pilot_info.id)
                                        .unwrap_or(&default_no_ign)
                                );

                                channel.say(ctx,
                                                format!("<@{}> [**ON**] Captain {} is taking off with you as a passenger!",
                                                        &p_mention, &pilot_name))
                                           .await?;
                            }
                        }
                    } else {
                        let pilot_cm_id = match sqlx::query!(
                            "SELECT id FROM rong_clanmember
                             WHERE clan_id = $1 AND
                                   user_id = $2;",
                            &clan_id,
                            &pilot_info.user_id
                        )
                        .fetch_one(&pool)
                        .await
                        {
                            Ok(row) => row.id,
                            Err(_) => {
                                msg.channel_id
                                    .say(ctx, "There was as error matching pilot info.")
                                    .await?;
                                return Ok(());
                            }
                        };

                        let used_fc: i64 = match sqlx::query!(
                            "SELECT COUNT (*) as count FROM rongbot.force_quit
                             WHERE clanmember_id = $1 AND
                             cb_id = $2 AND
                             day_used = $3;",
                            &pilot_cm_id,
                            &cb_info.id,
                            &cb_day
                        )
                        .fetch_one(&pool)
                        .await
                        {
                            Ok(row) => row.count.unwrap_or(0),
                            Err(_) => {
                                msg.channel_id
                                    .say(ctx, "There is a problem getting FCs.")
                                    .await?;
                                return Ok(());
                            }
                        };

                        if used_fc > 0 {
                            let out_msg =
                                format!("{}\n**WARNING!! YOU DO NOT HAVE FC TODAY**", &m.content);

                            let _ = &m.edit(ctx, |nm| nm.content(out_msg)).await?;
                        }
                    }

                    let mut alert_times = 0;
                    // Start alerting loop, every half an hour if the flight is still ongoing
                    loop {
                        // std::thread::sleep(Duration::from_secs(2));
                        tokio::time::sleep(Duration::from_secs(60 * 30)).await;
                        alert_times += 1;
                        let res: Option<i32> = match sqlx::query!(
                            "SELECT id FROM rongbot.flight
                                     WHERE id=$1 AND status='in flight';",
                            flight_id
                        )
                        .fetch_one(&pool)
                        .await
                        {
                            Ok(row) => Some(row.id),
                            Err(e) => None,
                        };

                        if res == None {
                            break;
                        } else {
                            msg.reply_mention(
                                ctx,
                                format!(
                                    "WARNING, your flight ({}) has been going on for {} minutes.",
                                    ign,
                                    30 * &alert_times
                                ),
                            )
                            .await?;
                        }
                    }
                }
                StartOptions::No => {
                    m.delete(ctx).await?;
                    msg.reply(ctx, "Canceled, deboarding your plane...").await?;
                    return Ok(());
                }
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
