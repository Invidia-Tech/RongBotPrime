use std::{str::FromStr, time::Duration};

use chrono::Utc;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    futures::StreamExt,
    model::{application::interaction::InteractionResponseType, channel::Message},
};

use humantime::format_duration;

use crate::{
    data::{CbStatus, ChannelPersona, DatabasePool, Flight, FlightStatus, PassengerOptions},
    utils::{atc::*, clan::*, macros::*},
};

#[command("atc_crash")]
#[aliases("crash")]
#[description(
    "Crash a flight. Please mention the clan name to crash.\n\n\
     >atc crash Ethereal"
)]
#[bucket = "atc"]
async fn flight_crash(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id
            .say(ctx, "Which clan's flight should I crash?")
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

    // Only allows this command within CB marked channels.
    result_or_say_why!(is_channel_persona(ctx, msg, ChannelPersona::Cb), ctx, msg);

    // Get the clan of which to crash
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

    let (cb_info, cb_status) =
        result_or_say_why!(get_latest_cb(ctx, &clan_id, &clan_name), ctx, msg);

    match cb_status {
        CbStatus::Past | CbStatus::Future => {
            msg.channel_id
                .say(
                    ctx,
                    format!(
                        "Warning, there is no currently active CB!\n\
                        {clan_name} - {name} is already over. \
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

    let is_superadmin = match sqlx::query!(
        "SELECT is_superadmin
         FROM public.rong_user
         WHERE platform_id = $1;",
        msg.author.id.to_string()
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row.is_superadmin,
        Err(_) => false,
    };

    let is_guild_lead = match sqlx::query!(
        "SELECT is_lead
         FROM rong_clanmember cm
         JOIN rong_user u ON cm.user_id = u.id
         WHERE cm.active = true
           AND clan_id = $1
           AND platform_id = $2",
        clan_id,
        msg.author.id.to_string()
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row.is_lead,
        Err(_) => false,
    };

    if !(is_guild_lead || is_superadmin) {
        msg.reply_ping(
            ctx,
            format!("You are neither a lead of {} nor a Superadmin.", clan_name),
        )
        .await?;
        return Ok(());
    }

    let all_in_air_flights =
        result_or_say_why!(get_all_in_air_flights(ctx, &clan_id, &cb_info.id), ctx, msg);

    let all_clanmember_ign_map =
        result_or_say_why!(get_all_clanmember_ign_map(ctx, &clan_id), ctx, msg);

    let all_pilot_ign_map = result_or_say_why!(get_all_pilot_ign_map(ctx, &clan_id), ctx, msg);
    let all_pilot_discord_map =
        result_or_say_why!(get_all_pilot_discord_map(ctx, &clan_id), ctx, msg);

    let crasher_nick = match msg.author_nick(ctx).await {
        Some(nick) => nick,
        None => (*msg.author.name).to_string(),
    };

    // Ensure that the passenger_member_id is within the same guild as the pilot.
    let passenger_options =
        PassengerOptions::new(&all_clanmember_ign_map, &all_pilot_ign_map, true);
    let m = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.content(format!(
                "Which flight would you like to **CRASH** <@{}>?",
                msg.author.id
            ))
            .components(|c| c.add_action_row(passenger_options.action_row(&all_in_air_flights)))
        })
        .await?;

    // Wait for the user to make a selection
    let mci = match m
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(20))
        .await
    {
        Some(ci) => ci,
        None => {
            msg.reply(&ctx, "Timed out.").await?;
            m.delete(&ctx).await?;
            return Ok(());
        }
    };

    // data.custom_id contains the id of the component (here "Passenger_select")
    // and should be used to identify if a message has multiple components.
    // data.values contains the selected values from the menu
    let end_flight_id = mci.clone().data.values.get(0).unwrap().parse::<i32>()?;

    let mut end_flight: &Flight = &all_in_air_flights[0];
    for f in &all_in_air_flights {
        if f.id == end_flight_id {
            end_flight = f;
            break;
        }
    }
    let default_no_ign = "No IGN".to_string();

    let pilot_text = format!(
        "Pilot: {}",
        &all_pilot_ign_map
            .get(&end_flight.pilot_id)
            .unwrap_or(&default_no_ign)
    );
    let passenger_text = match &end_flight.passenger_id {
        Some(p) => format!(
            "Passenger: {}",
            &all_clanmember_ign_map.get(p).unwrap_or(&default_no_ign)
        ),
        None => "Solo Flight".to_string(),
    };
    let humantime_ago = match &end_flight.end_time {
        Some(t) => format_duration(
            chrono::Duration::seconds(t.timestamp() - end_flight.start_time.timestamp())
                .to_std()
                .unwrap(),
        )
        .to_string(),
        None => format_duration(
            chrono::Duration::seconds(Utc::now().timestamp() - end_flight.start_time.timestamp())
                .to_std()
                .unwrap(),
        )
        .to_string(),
    };

    // Acknowledge the interaction and edit the message

    mci.create_interaction_response(&ctx, |r| {
        r.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|d| {
                d.content(format!(
                    "What happened to this flight? (**{} {}** - {} ago)",
                    pilot_text, passenger_text, humantime_ago
                ))
                .ephemeral(true)
                .components(|c| c.add_action_row(FlightStatus::action_row()))
            })
    })
    .await?;
    // Wait for multiple interactions

    let end_flight_status;
    let mut cib = m
        .await_component_interactions(&ctx)
        .timeout(Duration::from_secs(20))
        .build();
    let mut fc_used = false;

    if let Some(mci) = cib.next().await {
        let custom_id = &mci.data.custom_id;
        if custom_id == "used fc" {
            end_flight_status = FlightStatus::Crashed;
            fc_used = true;
        } else {
            end_flight_status = FlightStatus::from_str(custom_id)?;
        }
    } else {
        msg.reply(ctx, "Timed out!").await?;
        m.delete(&ctx).await?;
        return Ok(());
    };

    match sqlx::query_unchecked!(
        "UPDATE rongbot.flight SET (end_time, status) =
		 (now(), $1) WHERE id = $2",
        &end_flight_status,
        &end_flight_id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => {
            let mut reply = format!(
                "You have FORCED DOWN: **{} {}** (Started {} ago) will end with status: {}",
                &pilot_text, &passenger_text, &humantime_ago, &end_flight_status
            );
            if fc_used {
                reply = format!(
                    "You have FORCED DOWN: **{} {}** (Started {} ago) will end with status: crashed and used FC\n\
                     **PLEASE REMEMBER TO >fc AND TRACK THIS!**",
                    &pilot_text, &passenger_text, &humantime_ago
                );
            }
            msg.reply_mention(ctx, reply).await?
        }
        Err(e) => {
            msg.reply_mention(ctx, format!("Something's wrong! {}", e))
                .await?;
            // Delete the orig message or there will be dangling components
            m.delete(&ctx).await?;
            return Ok(());
        }
    };

    // Delete the orig message or there will be dangling components
    m.delete(&ctx).await?;

    let mut alert_msg = String::default();
    // Alert the passenger if exists;
    let mention_ch = match result_or_say_why!(get_alert_channel_for_clan(ctx, &clan_id), ctx, msg) {
        None => {
            alert_msg.push_str(
                "Warning! I cannot notify your passenger as the ATC alert channel is not set!",
            );
            None
        }
        Some(alert_ch) => Some(alert_ch),
    };
    if let Some(clanmember_id) = end_flight.passenger_id {
        let mention_ch = mention_ch.unwrap_or(msg.channel_id.0);
        let default_no_discord = "none".to_string();
        println!("End flight pilot_id: {}", &end_flight.pilot_id);
        println!(
            "Pilot discord: {:?}",
            all_pilot_discord_map.get(&end_flight.pilot_id)
        );
        let pilot_discord = (all_pilot_discord_map
            .get(&end_flight.pilot_id)
            .unwrap_or(&default_no_discord))
        .to_string();
        if pilot_discord == "none" {
            msg.channel_id
                .say(ctx, "I couldn't find the pilot to ping!")
                .await?;
            return Ok(());
        }
        match result_or_say_why!(
            get_clanmember_mention_from_id(ctx, &clanmember_id),
            ctx,
            msg
        ) {
            None => return Ok(()),
            Some(p_mention) => {
                let channel = match ctx.cache.guild_channel(mention_ch) {
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
                        .get(&end_flight.pilot_id)
                        .unwrap_or(&default_no_ign)
                );

                channel
                    .say(
                        ctx,
                        format!(
                            "<@{}> <@{}> [**FORCE CRASHED**] Captain {}, {} has \
                             force ended your flight! Status: {}. Flight duration: {}",
                            &p_mention,
                            &pilot_discord,
                            &pilot_name,
                            &crasher_nick,
                            &end_flight_status,
                            &humantime_ago
                        ),
                    )
                    .await?;
            }
        }
    } else {
        let mention_ch = mention_ch.unwrap_or(msg.channel_id.0);
        let default_no_discord = "none".to_string();
        let pilot_discord = (all_pilot_discord_map
            .get(&end_flight.pilot_id)
            .unwrap_or(&default_no_discord))
        .to_string();
        if pilot_discord == "none" {
            msg.channel_id
                .say(ctx, "I couldn't find the pilot to ping!")
                .await?;
            return Ok(());
        }
        let channel = match ctx.cache.guild_channel(mention_ch) {
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
                .get(&end_flight.pilot_id)
                .unwrap_or(&default_no_ign)
        );

        channel
            .say(
                ctx,
                format!(
                    "<@{}> [**FORCE CRASHED**] Captain {}, your self flight has been \
                     force ended by {} Status: {}. Flight duration: {}",
                    &pilot_discord, &pilot_name, &crasher_nick, &end_flight_status, &humantime_ago
                ),
            )
            .await?;
    }

    Ok(())
}
