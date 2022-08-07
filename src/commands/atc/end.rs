use std::{
    collections::HashMap,
    error::Error as StdError,
    fmt,
    str::FromStr,
    time::Duration,
};

use chrono::Utc;
use serenity::{
    builder::{
        CreateActionRow,
        CreateSelectMenu,
        CreateSelectMenuOption,
    },
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    futures::StreamExt,
    model::{
        application::interaction::InteractionResponseType,
        channel::Message,
    },
};

use humantime::format_duration;

use crate::{
    data::{
        CbStatus,
        ChannelPersona,
        DatabasePool,
        Flight,
        FlightStatus,
    },
    utils::{
        atc::*,
        clan::*,
        macros::*,
        rong::*,
    },
};

#[derive(Debug)]
struct PassengerOptions<'a> {
    ign_map: &'a HashMap<i32, String>,
}

#[derive(Debug)]
struct ParseComponentError(String);

impl fmt::Display for ParseComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse {} as component", self.0)
    }
}

impl StdError for ParseComponentError {}

impl<'a> fmt::Display for PassengerOptions<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Map length: {}", self.ign_map.len())
    }
}

impl<'a> PassengerOptions<'a> {
    pub fn new(ign_map: &'a HashMap<i32, String>) -> Self { Self { ign_map } }

    fn menu_option(&self, flight: &Flight) -> CreateSelectMenuOption {
        let mut opt = CreateSelectMenuOption::default();
        let default_no_ign = "No IGN".to_string();
        // This is what will be shown to the user
        let passenger_text = match &flight.passenger_id {
            Some(p) => format!(
                "Passenger: {}",
                self.ign_map.get(p).unwrap_or(&default_no_ign)
            ),
            None => "Solo Flight".to_string(),
        };
        let humantime_ago = match &flight.end_time {
            Some(t) => format_duration(
                chrono::Duration::seconds(t.timestamp() - flight.start_time.timestamp())
                    .to_std()
                    .unwrap(),
            )
            .to_string(),
            None => format_duration(
                chrono::Duration::seconds(Utc::now().timestamp() - flight.start_time.timestamp())
                    .to_std()
                    .unwrap(),
            )
            .to_string(),
        };
        opt.label(format!(
            "{} - Took off: {} ago",
            passenger_text, humantime_ago
        ));
        // This is used to identify the selected value
        opt.value(flight.id);
        opt
    }

    fn select_menu(&self, flights: &Vec<Flight>) -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id("Passenger_select");
        menu.placeholder("No Passenger selected");
        menu.options(|f| {
            for flight in flights {
                f.add_option(self.menu_option(flight));
            }
            f
        });
        menu
    }

    fn action_row(&self, flights: &Vec<Flight>) -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        // A select menu must be the only thing in an action row!
        ar.add_select_menu(self.select_menu(flights));
        ar
    }
}

#[command("atc_end")]
#[aliases("end")]
#[description("End a flight.")]
#[bucket = "atc"]
async fn flight_end(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    let pilot_user_id =
        result_or_say_why!(get_user_id(ctx, msg, &msg.author.id.to_string()), ctx, msg);

    let pilot_info = result_or_say_why!(
        get_pilot_info_or_create_new(ctx, &pilot_user_id, &clan_id),
        ctx,
        msg
    );

    let pilot_ongoing_flights = result_or_say_why!(
        get_pilot_ongoing_flights(ctx, &pilot_info.id, &clan_id, &cb_info.id),
        ctx,
        msg
    );

    if pilot_ongoing_flights.is_empty() {
        msg.channel_id
            .say(ctx, "You do not have any ongoing flights!")
            .await?;
        return Ok(());
    }

    let all_clanmember_ign_map =
        result_or_say_why!(get_all_clanmember_ign_map(ctx, &clan_id), ctx, msg);

    let all_pilot_ign_map = result_or_say_why!(get_all_pilot_ign_map(ctx, &clan_id), ctx, msg);

    // Get passenger or determine solo flight
    let passenger_clan_member_id: Option<i32>;
    let mut mentioned_flight_id: Option<i32> = None;
    let mut mentioned_passenger = false;
    let mut ign = "No IGN".to_string();
    if !args.is_empty() {
        mentioned_passenger = true;
        ign = args.single::<String>().unwrap();
        if ign.to_ascii_lowercase() == "self" {
            for flight in &pilot_ongoing_flights {
                if flight.passenger_id == None {
                    mentioned_flight_id = Some(flight.id);
                }
            }
        } else {
            let (clan_member_id, _passenger_clan_user_id) =
                result_or_say_why!(get_clan_member_id_by_ign(ctx, &clan_id, &ign), ctx, msg);
            // msg.channel_id
            //    .say(ctx, format!("Passenger_member_id is: {:?}", passenger_member_id))
            //    .await?;
            passenger_clan_member_id = Some(clan_member_id);
            for flight in &pilot_ongoing_flights {
                if flight.passenger_id == passenger_clan_member_id {
                    mentioned_flight_id = Some(flight.id);
                }
            }
        }
    }

    if mentioned_passenger && mentioned_flight_id == None {
        let out_msg = if ign.to_ascii_lowercase() == "self" {
            "You do not have an active self flight!".to_string()
        } else {
            format!("{} does not have an active flight with you!", ign)
        };
        msg.reply(ctx, out_msg).await?;
        return Ok(());
    }

    let end_flight_id: i32;
    let mut m;
    let mut mci = None;
    if mentioned_flight_id != None {
        end_flight_id = mentioned_flight_id.unwrap_or(0);
        m = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.content(format!(
                    "<@{}> Landing your flight with {}. Please hold.",
                    msg.author.id, ign
                ))
            })
            .await
            .unwrap();
    } else {
        // Ensure that the passenger_member_id is within the same guild as the pilot.
        let passenger_options = PassengerOptions::new(&all_clanmember_ign_map);
        m = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.content(format!(
                    "Which flight would you like to land <@{}>?",
                    msg.author.id
                ))
                .components(|c| {
                    c.add_action_row(passenger_options.action_row(&pilot_ongoing_flights))
                })
            })
            .await
            .unwrap();

        // Wait for the user to make a selection
        mci = match m
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(20))
            .await
        {
            Some(ci) => Some(ci),
            None => {
                msg.reply(&ctx, "Timed out.").await.unwrap();
                m.delete(&ctx).await.unwrap();
                return Ok(());
            }
        };

        // data.custom_id contains the id of the component (here "Passenger_select")
        // and should be used to identify if a message has multiple components.
        // data.values contains the selected values from the menu
        end_flight_id = mci
            .clone()
            .unwrap()
            .data
            .values
            .get(0)
            .unwrap()
            .parse::<i32>()?;
    }

    let mut end_flight: &Flight = &pilot_ongoing_flights[0];
    for f in &pilot_ongoing_flights {
        if f.id == end_flight_id {
            end_flight = f;
            break;
        }
    }
    let default_no_ign = "No IGN".to_string();
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

    // msg.channel_id
    // 	.say(ctx, format!("You've selected: {}", end_flight_id))
    // 	.await?;

    // Acknowledge the interaction and edit the message

    if let Some(mci) = mci {
        mci.create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.content(format!(
                        "What happened to your flight? (**{}** - {} ago)",
                        passenger_text, humantime_ago
                    ))
                    .ephemeral(true)
                    .components(|c| c.add_action_row(FlightStatus::action_row()))
                })
        })
        .await
        .unwrap();
    } else {
        m.delete(ctx).await?;
        m = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.content(format!(
                    "<@{}> What happened to your flight? (**{}** - {} ago)",
                    msg.author.id, passenger_text, humantime_ago
                ))
                .components(|c| c.add_action_row(FlightStatus::action_row()))
            })
            .await
            .unwrap();
    }

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

        // Acknowledge the interaction and send a reply
        // mci.create_interaction_response(&ctx, |r| {
        // 	// This time we dont edit the message but reply to it
        // 	r.kind(InteractionResponseType::ChannelMessageWithSource)
        // 		.interaction_response_data(|d| {
        // 			// Make the message hidden for other users by setting `ephemeral(true)`.
        // 			d.ephemeral(true).content(format!(
        // 				"Your flight will end with status {}",
        // 				&end_flight_status
        // 			))
        // 		})
        // })
        // .await
        // .unwrap();
    } else {
        msg.reply(ctx, "Timed out!").await?;
        m.delete(&ctx).await?;
        return Ok(());
    };

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
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
                "Your flight: **{}** (Started {} ago) will end with status: {}",
                &passenger_text, &humantime_ago, &end_flight_status
            );
            if fc_used {
                reply = format!(
                    "Your flight: **{}** (Started {} ago) will end with status: crashed and used FC\n\
                     **PLEASE REMEMBER TO >fc AND TRACK THIS!**",
                    &passenger_text, &humantime_ago
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
                        .get(&pilot_info.id)
                        .unwrap_or(&default_no_ign)
                );

                channel
                    .say(
                        ctx,
                        format!(
                            "<@{}> [**OFF**] Captain {} has ended your flight! Status: {}. Flight duration: {}",
                            &p_mention, &pilot_name, &end_flight_status, &humantime_ago
                        ),
                    )
                    .await?;
            }
        }
    }

    // Delete the orig message or there will be dangling components
    m.delete(&ctx).await?;

    Ok(())
}
