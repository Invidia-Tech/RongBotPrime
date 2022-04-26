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
        CreateButton,
        CreateEmbed,
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
        channel::{
            Embed,
            Message,
            ReactionType,
        },
        interactions::{
            message_component::ButtonStyle,
            InteractionResponseType,
        },
    },
    utils::MessageBuilder,
};

use humantime::format_duration;

use crate::{
    data::{
        CbStatus,
        ChannelPersona,
        DatabasePool,
        Flight,
        FlightStatus,
        RongPilot,
    },
    error::RongError,
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

#[command("end")]
#[description("End a flight.")]
#[bucket = "atc"]
async fn flight_end(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
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

    let passenger_options = PassengerOptions::new(&all_clanmember_ign_map);
    let m = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.content(format!(
                "Which flight would you like to land <@{}>?",
                msg.author.id
            ))
            .components(|c| c.add_action_row(passenger_options.action_row(&pilot_ongoing_flights)))
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

    // data.custom_id contains the id of the component (here "Passenger_select")
    // and should be used to identify if a message has multiple components.
    // data.values contains the selected values from the menu
    let end_flight_id: i32 = mci.data.values.get(0).unwrap().parse::<i32>()?;
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

    // Wait for multiple interactions

    let end_flight_status;
    let mut cib = m
        .await_component_interactions(&ctx)
        .timeout(Duration::from_secs(20))
        .build();

    if let Some(mci) = cib.next().await {
        end_flight_status = FlightStatus::from_str(&mci.data.custom_id)?;
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
            msg.reply_mention(
                ctx,
                format!(
                    "Your flight: **{}** (Started {} ago) will end with status: {}",
                    &passenger_text, &humantime_ago, &end_flight_status,
                ),
            )
            .await?;
        }
        Err(e) => {
            msg.reply_mention(ctx, format!("Something's wrong! {}", e))
                .await?;
        }
    }

    // Delete the orig message or there will be dangling components
    m.delete(&ctx).await?;

    Ok(())
}
