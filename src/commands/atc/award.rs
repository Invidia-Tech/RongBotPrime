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
    solo: u32,
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
            solo: 0,
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
    // msg.channel_id
    //     .say(ctx, format!("Selected CB id: {}", selected_cb_id))
    //     .await?;

    let mut cb_name = "No Name".to_string();
    for n in cbs {
        if selected_cb_id == n.0 {
            cb_name = n.1;
        }
    }

    let mut all_flights =
        result_or_say_why!(get_all_flights(ctx, &clan_id, &selected_cb_id), ctx, msg);
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

    let top_nth_pilots = 6;

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
        match flight.passenger_id {
            None => {
                if flight.status != FlightStatus::Canceled {
                    pilot_stat.solo += 1;
                }
            }
            _ => (),
        };
    }

    let mut pilot_stats: Vec<FlightCount> = pilot_stats.into_values().collect();

    // ====================== Output stats =========================
    let all_pilot_ign_map = result_or_say_why!(get_all_pilot_ign_map(ctx, &clan_id), ctx, msg);

    // OVERALL
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

    // ========== TOP PER CATEGORY =========
    // Top total not including cancelled
    pilot_stats.sort_unstable_by(|a, b| {
        (b.landed + b.amb + b.crashed).cmp(&(a.landed + a.amb + a.crashed))
    });
    let mut top_total_page = CreateEmbed::default();
    top_total_page
        .title("Total Flights by Pilot")
        .description("OmegaChad pilots (excluding cancelled & in flight)")
        //.image("attachment://KyoukaSmile.jpg")
        .footer(|f| f.text("Days since last int: 0"))
        .timestamp(chrono::Utc::now().to_rfc3339());
    let default_no_ign = "No IGN".to_string();
    for stat in &pilot_stats[..top_nth_pilots] {
        let pilot_output = format!(
            "Pilot: {}",
            all_pilot_ign_map
                .get(&stat.pilot_id)
                .unwrap_or(&default_no_ign)
        );
        top_total_page.field(pilot_output, stat.landed + stat.amb + stat.crashed, true);
    }
    msg.channel_id
        .send_message(ctx, |e| {
            e.embed(|m| {
                *m = top_total_page;
                m
            })
        })
        .await?;

    // Percentage landed/amb
    pilot_stats.sort_unstable_by(|a, b| {
        let bl = b.landed as f64;
        let btotal = (b.landed
            + if b.amb == 0 { 1 } else { b.amb }
            + if b.canceled == 0 { 1 } else { b.canceled }
            + if b.crashed == 0 { 1 } else { b.crashed }) as f64;
        let al = a.landed as f64;
        let atotal = (a.landed
            + if a.amb == 0 { 1 } else { a.amb }
            + if a.canceled == 0 { 1 } else { a.canceled }
            + if a.crashed == 0 { 1 } else { a.crashed }) as f64;
        // println!("Comparing {} / {} against {} / {}", bl, btotal, al, atotal);
        (bl / btotal).partial_cmp(&(al / atotal)).unwrap()
    });
    let mut landed_percentage_page = CreateEmbed::default();
    landed_percentage_page
        .title("Top Percentage No ints")
        .description("Omega percentage chads")
        //.image("attachment://KyoukaSmile.jpg")
        .footer(|f| f.text("Days since last int: 0"))
        .timestamp(chrono::Utc::now().to_rfc3339());
    let default_no_ign = "No IGN".to_string();
    for stat in &pilot_stats[..] {
        let pilot_output = format!(
            "Pilot: {}",
            all_pilot_ign_map
                .get(&stat.pilot_id)
                .unwrap_or(&default_no_ign)
        );
        landed_percentage_page.field(
            pilot_output,
            format!(
                "{:.2}% ({}/{})",
                (stat.landed as f64
                    / (stat.landed
                        + if stat.amb == 0 { 1 } else { stat.amb }
                        + if stat.canceled == 0 { 1 } else { stat.canceled }
                        + if stat.crashed == 0 { 1 } else { stat.crashed })
                        as f64)
                    * 100.0,
                stat.landed,
                (stat.landed
                    + if stat.amb == 0 { 1 } else { stat.amb }
                    + if stat.canceled == 0 { 1 } else { stat.canceled }
                    + if stat.crashed == 0 { 1 } else { stat.crashed })
            ),
            true,
        );
    }
    msg.channel_id
        .send_message(ctx, |e| {
            e.embed(|m| {
                *m = landed_percentage_page;
                m
            })
        })
        .await?;

    // Top Landed
    pilot_stats.sort_unstable_by(|a, b| b.landed.cmp(&a.landed));
    let mut top_landed_page = CreateEmbed::default();
    top_landed_page
        .title("Top Successful Flights")
        .description("Here are your Gigachad pilots.")
        //.image("attachment://KyoukaSmile.jpg")
        .footer(|f| f.text("Days since last int: 0"))
        .timestamp(chrono::Utc::now().to_rfc3339());
    let default_no_ign = "No IGN".to_string();
    for stat in &pilot_stats[..top_nth_pilots] {
        let pilot_output = format!(
            "Pilot: {}",
            all_pilot_ign_map
                .get(&stat.pilot_id)
                .unwrap_or(&default_no_ign)
        );
        top_landed_page.field(pilot_output, stat.landed, true);
    }

    msg.channel_id
        .send_message(ctx, |e| {
            e.embed(|m| {
                *m = top_landed_page;
                m
            })
        })
        .await?;

    // Top Ambulanced
    pilot_stats.sort_unstable_by(|a, b| b.amb.cmp(&a.amb));
    let mut top_amb_page = CreateEmbed::default();
    top_amb_page
        .title("Most Ambulanced Pilots")
        .description("Top inting pleasseeeeee...")
        //.image("attachment://KyoukaSmile.jpg")
        .footer(|f| f.text("Days since last int: 0"))
        .timestamp(chrono::Utc::now().to_rfc3339());
    let default_no_ign = "No IGN".to_string();
    for stat in &pilot_stats[..top_nth_pilots] {
        let pilot_output = format!(
            "Pilot: {}",
            all_pilot_ign_map
                .get(&stat.pilot_id)
                .unwrap_or(&default_no_ign)
        );
        top_amb_page.field(pilot_output, stat.amb, true);
    }

    msg.channel_id
        .send_message(ctx, |e| {
            e.embed(|m| {
                *m = top_amb_page;
                m
            })
        })
        .await?;

    // Top Crashes
    pilot_stats.sort_unstable_by(|a, b| b.crashed.cmp(&a.crashed));
    let mut top_crashed_page = CreateEmbed::default();
    top_crashed_page
        .title("Too many crashes")
        .description("If you BB, you're kicked instantly.")
        //.image("attachment://KyoukaSmile.jpg")
        .footer(|f| f.text("Days since last int: 0"))
        .timestamp(chrono::Utc::now().to_rfc3339());
    let default_no_ign = "No IGN".to_string();
    for stat in &pilot_stats[..top_nth_pilots] {
        let pilot_output = format!(
            "Pilot: {}",
            all_pilot_ign_map
                .get(&stat.pilot_id)
                .unwrap_or(&default_no_ign)
        );
        top_crashed_page.field(pilot_output, stat.crashed, true);
    }

    msg.channel_id
        .send_message(ctx, |e| {
            e.embed(|m| {
                *m = top_crashed_page;
                m
            })
        })
        .await?;

    // Top Canceled
    pilot_stats.sort_unstable_by(|a, b| b.canceled.cmp(&a.canceled));
    let mut top_canceled_page = CreateEmbed::default();
    top_canceled_page
        .title("Most cancels")
        .description("You sure are unsure of yourself")
        //.image("attachment://KyoukaSmile.jpg")
        .footer(|f| f.text("Days since last int: 0"))
        .timestamp(chrono::Utc::now().to_rfc3339());
    let default_no_ign = "No IGN".to_string();
    for stat in &pilot_stats[..top_nth_pilots] {
        let pilot_output = format!(
            "Pilot: {}",
            all_pilot_ign_map
                .get(&stat.pilot_id)
                .unwrap_or(&default_no_ign)
        );
        top_canceled_page.field(pilot_output, stat.canceled, true);
    }

    msg.channel_id
        .send_message(ctx, |e| {
            e.embed(|m| {
                *m = top_canceled_page;
                m
            })
        })
        .await?;

    // Good solo flyers.
    let min_solo = 10;
    pilot_stats.sort_unstable_by(|a, b| b.solo.cmp(&a.solo));
    let mut top_canceled_page = CreateEmbed::default();
    top_canceled_page
        .title("Good solo flights")
        .description(format!("Thanks so much! Solo minimum: {}", min_solo))
        //.image("attachment://KyoukaSmile.jpg")
        .footer(|f| f.text("Days since last int: 0"))
        .timestamp(chrono::Utc::now().to_rfc3339());
    let default_no_ign = "No IGN".to_string();
    for stat in &pilot_stats {
        if stat.solo >= min_solo {
            let pilot_output = format!(
                "Pilot: {}",
                all_pilot_ign_map
                    .get(&stat.pilot_id)
                    .unwrap_or(&default_no_ign)
            );
            top_canceled_page.field(pilot_output, stat.solo, true);
        }
    }
    msg.channel_id
        .send_message(ctx, |e| {
            e.embed(|m| {
                *m = top_canceled_page;
                m
            })
        })
        .await?;

    // Longest Flight
    all_flights.sort_by(|a, b| {
        (b.end_time.unwrap().timestamp() - b.start_time.timestamp())
            .cmp(&(a.end_time.unwrap().timestamp() - a.start_time.timestamp()))
    });
    let mut longest_flights_page = CreateEmbed::default();
    longest_flights_page
        .title("Omega Long Flights")
        .description("Omg you fell asleep.")
        //.image("attachment://KyoukaSmile.jpg")
        .footer(|f| f.text("Days since last int: 0"))
        .timestamp(chrono::Utc::now().to_rfc3339());
    for flight in &all_flights[..top_nth_pilots] {
        let duration_readable = match &flight.end_time {
            Some(t) => format_duration(
                chrono::Duration::seconds(t.timestamp() - flight.start_time.timestamp())
                    .to_std()?,
            )
            .to_string(),
            None => format_duration(
                chrono::Duration::seconds(Utc::now().timestamp() - flight.start_time.timestamp())
                    .to_std()?,
            )
            .to_string(),
        };
        let pilot_output = format!(
            "Pilot: {}",
            all_pilot_ign_map
                .get(&flight.pilot_id)
                .unwrap_or(&default_no_ign)
        );
        longest_flights_page.field(pilot_output, duration_readable, true);
    }
    msg.channel_id
        .send_message(ctx, |e| {
            e.embed(|m| {
                *m = longest_flights_page;
                m
            })
        })
        .await?;

    Ok(())
}
