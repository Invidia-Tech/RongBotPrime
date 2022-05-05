use std::{
    collections::HashMap,
    error::Error as StdError,
    fmt,
    str::FromStr,
    time::Duration,
};

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
    model::{
        channel::{
            Embed,
            Message,
            ReactionType,
        },
        interactions::message_component::ButtonStyle,
    },
    utils::MessageBuilder,
};

use chrono::Utc;
use humantime::format_duration;
use sqlx::Row;

use crate::{
    checks::rong_admin::*,
    data::{
        CbStatus,
        ChannelPersona,
        DatabasePool,
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
struct CBSelection;

#[derive(Debug)]
struct ParseComponentError(String);

impl fmt::Display for ParseComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse {} as component", self.0)
    }
}

impl StdError for ParseComponentError {}

impl fmt::Display for CBSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "There is nothing here.") }
}

impl CBSelection {
    fn menu_option(cb: &(i32, String)) -> CreateSelectMenuOption {
        let mut opt = CreateSelectMenuOption::default();
        opt.label(&cb.1);
        // This is used to identify the selected value
        opt.value(cb.0);
        opt
    }

    fn select_menu(cbs: &Vec<(i32, String)>) -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id("CB_select");
        menu.placeholder("No CB Selected");
        menu.options(|f| {
            for cb in cbs {
                f.add_option(CBSelection::menu_option(cb));
            }
            f
        });
        menu
    }

    fn action_row(cbs: &Vec<(i32, String)>) -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        // A select menu must be the only thing in an action row!
        ar.add_select_menu(CBSelection::select_menu(cbs));
        ar
    }
}

struct FlightCount {
    pilot_id: i32,
    amb: u32,
    canceled: u32,
    landed: u32,
    crashed: u32,
    inflight: u32,
}

impl Default for FlightCount {
    fn default() -> FlightCount {
        FlightCount {
            pilot_id: 0,
            amb: 0,
            canceled: 0,
            landed: 0,
            crashed: 0,
            inflight: 0,
        }
    }
}

#[command("atc_award")]
#[aliases("award")]
#[checks(RongAdmin)]
#[description("This shows the status of current flights.")]
#[bucket = "atc"]
async fn atc_award(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    let clan_name = match args.single_quoted::<String>() {
        Ok(n) => n,
        Err(_) => {
            msg.channel_id
                .say(ctx, "You must mention a guild by name.")
                .await?;
            return Ok(());
        }
    };

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

    let cbs = match sqlx::query!(
        "SELECT id, name
         FROM rong_clanbattle
         WHERE clan_id = $1
         ORDER BY start_time DESC;",
        clan_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            if rows.is_empty() {
                msg.channel_id
                    .say(ctx, format!("There are no CBs for {}", clan_name))
                    .await?;
                return Ok(());
            }
            rows.iter().map(|x| (x.id, x.name.to_owned())).collect()
        }
        Err(e) => {
            msg.channel_id
                .say(ctx, format!("Ahhh there's an error! {}", e))
                .await?;
            return Ok(());
        }
    };

    let m = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.content(format!(
                "Which CB would you like to award? <@{}>?",
                msg.author.id
            ))
            .components(|c| c.add_action_row(CBSelection::action_row(&cbs)))
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

    let selected_cb_id: i32 = mci.data.values.get(0).unwrap().parse::<i32>()?;
    m.delete(ctx).await?;
    msg.channel_id
        .say(ctx, format!("Selected CB id: {}", selected_cb_id))
        .await?;

    let mut cb_name = "No Name".to_string();
    for n in cbs {
        if selected_cb_id == n.0 {
            cb_name = n.1;
        }
    }

    let all_flights = result_or_say_why!(get_all_flights(ctx, &clan_id, &selected_cb_id), ctx, msg);
    if all_flights.is_empty() {
        msg.channel_id
            .say(ctx, "There are no flights for the selected CB.")
            .await?;
    }

    // ========================= Overall Stats =============================

    // Total flights information
    let mut amb_count: u32 = 0;
    let mut in_flight_count: u32 = 0;
    let mut landed_count: u32 = 0;
    let mut canceled_count: u32 = 0;
    let mut crash_count: u32 = 0;

    let top_nth_pilots = 3;

    let mut pilot_stats: HashMap<i32, FlightCount> = HashMap::new();

    for flight in &all_flights {
        let pilot_stat = pilot_stats.entry(flight.pilot_id).or_insert(FlightCount {
            pilot_id: flight.pilot_id,
            ..Default::default()
        });
        match flight.status {
            FlightStatus::Amb => {
                amb_count += 1;
                pilot_stat.amb += 1;
            }
            FlightStatus::Canceled => {
                canceled_count += 1;
                pilot_stat.canceled += 1;
            }
            FlightStatus::Crashed => {
                crash_count += 1;
                pilot_stat.crashed += 1;
            }
            FlightStatus::InFlight => {
                in_flight_count += 1;
                pilot_stat.inflight += 1;
            }
            FlightStatus::Landed => {
                landed_count += 1;
                pilot_stat.landed += 1;
            }
        }
    }

    // ====================== Output stats =========================
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("Flights summary for {} - {}", clan_name, cb_name))
                    .description(format!("Total flights for this CB: {}", &all_flights.len()))
                    .fields(vec![
                        ("Currently In Flight:", in_flight_count.to_string(), true),
                        ("Successful:", landed_count.to_string(), true),
                        ("Ambulances:", amb_count.to_string(), true),
                        ("Crashes:", crash_count.to_string(), true),
                        ("Cancels:", canceled_count.to_string(), true),
                    ])
                    .timestamp(chrono::Utc::now().to_rfc3339())
            })
        })
        .await?;
    Ok(())
}
