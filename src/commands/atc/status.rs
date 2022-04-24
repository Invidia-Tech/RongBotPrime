use std::{
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

use crate::{
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

// #[derive(Debug)]
// enum Animal {
//     Cat,
//     Dog,
//     Horse,
//     Alpaca,
// }

// #[derive(Debug)]
// struct ParseComponentError(String);

// impl fmt::Display for ParseComponentError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Failed to parse {} as component", self.0)
//     }
// }

// impl StdError for ParseComponentError {}

// impl fmt::Display for Animal {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Cat => write!(f, "Cat"),
//             Self::Dog => write!(f, "Dog"),
//             Self::Horse => write!(f, "Horse"),
//             Self::Alpaca => write!(f, "Alpaca"),
//         }
//     }
// }

// impl FromStr for Animal {
//     type Err = ParseComponentError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "cat" => Ok(Animal::Cat),
//             "dog" => Ok(Animal::Dog),
//             "horse" => Ok(Animal::Horse),
//             "alpaca" => Ok(Animal::Alpaca),
//             _ => Err(ParseComponentError(s.to_string())),
//         }
//     }
// }

// impl Animal {
//     fn emoji(&self) -> char {
//         match self {
//             Self::Cat => '🐈',
//             Self::Dog => '🐕',
//             Self::Horse => '🐎',
//             Self::Alpaca => '🦙',
//         }
//     }

//     fn menu_option(&self) -> CreateSelectMenuOption {
//         let mut opt = CreateSelectMenuOption::default();
//         // This is what will be shown to the user
//         opt.label(format!("{} {}", self.emoji(), self));
//         // This is used to identify the selected value
//         opt.value(self.to_string().to_ascii_lowercase());
//         opt
//     }

//     fn select_menu() -> CreateSelectMenu {
//         let mut menu = CreateSelectMenu::default();
//         menu.custom_id("animal_select");
//         menu.placeholder("No animal selected");
//         menu.options(|f| {
//             f.add_option(Self::Cat.menu_option())
//                 .add_option(Self::Dog.menu_option())
//                 .add_option(Self::Horse.menu_option())
//                 .add_option(Self::Alpaca.menu_option())
//         });
//         menu
//     }

//     fn action_row() -> CreateActionRow {
//         let mut ar = CreateActionRow::default();
//         // A select menu must be the only thing in an action row!
//         ar.add_select_menu(Self::select_menu());
//         ar
//     }
// }

struct MenuPaginator<'a> {
    index: usize,
    ctx: &'a Context,
    msg: &'a Message,
    pages: Vec<CreateEmbed>,
}

impl<'a> MenuPaginator<'a> {
    pub fn new(ctx: &'a Context, msg: &'a Message, pages: Vec<CreateEmbed>) -> Self {
        Self {
            ctx,
            msg,
            pages,
            index: 0,
        }
    }

    fn create_page<'b>(&self, embed: &'b mut CreateEmbed) -> &'b mut CreateEmbed {
        embed.clone_from(&self.pages[self.index]);
        embed
    }

    fn create_action_row<'b>(
        &self,
        builder: &'b mut CreateActionRow,
        disabled: bool,
    ) -> &'b mut CreateActionRow {
        for emoji in ["⏮️", "◀", "⏹️", "▶️", "⏭️"] {
            builder.create_button(|b| {
                b.custom_id(emoji)
                    .style(ButtonStyle::Primary)
                    .emoji(ReactionType::Unicode(String::from(emoji)))
                    .disabled(
                        disabled
                            || (["⏮️", "◀"].contains(&emoji) && self.index == 0)
                            || (["▶️", "⏭️"].contains(&emoji)
                                && self.index == (self.pages.len() - 1)),
                    )
            });
        }
        builder
    }

    async fn create_message(&self) -> Result<Message, RongError> {
        let msg = self
            .msg
            .channel_id
            .send_message(&self.ctx, |b| {
                b.embed(|e| self.create_page(e));
                b.components(|c| c.create_action_row(|r| self.create_action_row(r, false)))
            })
            .await?;

        Ok(msg)
    }

    async fn edit_message(&self, msg: &mut Message, disable: bool) -> Result<(), RongError> {
        msg.edit(&self.ctx, |b| {
            b.embed(|e| self.create_page(e))
                .components(|c| c.create_action_row(|r| self.create_action_row(r, disable)))
        })
        .await?;

        Ok(())
    }

    pub async fn start(mut self) -> Result<(), RongError> {
        let mut message = self.create_message().await?;

        loop {
            let collector = message
                .await_component_interaction(&self.ctx)
                .timeout(Duration::from_secs(60 * 2))
                .author_id(self.msg.author.id)
                .collect_limit(1);

            let interaction = match collector.await {
                Some(interaction) => interaction,
                None => break,
            };

            let data = &interaction.data;
            match &data.custom_id[..] {
                "⏮️" => {
                    self.index = 0;
                    self.edit_message(&mut message, false).await?;
                }
                "◀" => {
                    self.index -= 1;
                    self.edit_message(&mut message, false).await?;
                }
                "⏹️" => {
                    self.edit_message(&mut message, true).await?;
                    interaction.defer(&self.ctx).await?;
                    break;
                }
                "▶️" => {
                    self.index += 1;
                    self.edit_message(&mut message, false).await?;
                }
                "⏭️" => {
                    self.index = self.pages.len() - 1;
                    self.edit_message(&mut message, false).await?;
                }
                _ => unreachable!(),
            };
            interaction.defer(&self.ctx).await?;
        }
        Ok(())
    }
}

#[command("atc_status")]
#[aliases("status")]
#[description("This shows the status of current flights.")]
async fn flight_status(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
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

    let all_flights = result_or_say_why!(get_all_flights(ctx, &clan_id, &cb_info.id), ctx, msg);

    if all_flights.is_empty() {
        msg.channel_id
            .say(ctx, "There are no flights this CB.")
            .await?;
        return Ok(());
    }

    // Time to output flights.
    let all_pilot_info_map = result_or_say_why!(get_all_pilot_info_map(ctx, &clan_id), ctx, msg);

    let all_pilot_ign_map = result_or_say_why!(get_all_pilot_ign_map(ctx, &clan_id), ctx, msg);

    let all_clanmember_ign_map =
        result_or_say_why!(get_all_clanmember_ign_map(ctx, &clan_id), ctx, msg);

    // Total flights information
    let mut amb_count: u8 = 0;
    let mut in_flight_count: u8 = 0;
    let mut landed_count: u8 = 0;
    let mut canceled_count: u8 = 0;
    let mut crash_count: u8 = 0;

    let mut flight_embeds: Vec<CreateEmbed> = Vec::default();
    let flights_per_page = 5;
    let mut flight_output = 0;
    for flight in &all_flights {
        match flight.status {
            FlightStatus::Amb => amb_count += 1,
            FlightStatus::Canceled => canceled_count += 1,
            FlightStatus::Crashed => crash_count += 1,
            FlightStatus::InFlight => in_flight_count += 1,
            FlightStatus::Landed => landed_count += 1,
        }
        if flight_output >= flights_per_page || flight_embeds.is_empty() {
            flight_output = 0;
            let mut new_flight_page = CreateEmbed::default();
            new_flight_page
                .title("Current Flights")
                .description("These are the recent running/landed flights.")
                //.image("attachment://KyoukaSmile.jpg")
                .footer(|f| f.text("Days since last int: 0"))
                .timestamp(chrono::Utc::now().to_rfc3339());
            flight_embeds.push(new_flight_page);
        }

        // Add flight information
        if let Some(f) = flight_embeds.last_mut() {
            let default_code = "NONE".to_string();
            let full_flight_call_sign = format!(
                "__Flight {} {}__",
                match all_pilot_info_map.get(&flight.pilot_id) {
                    Some(p) => p.code.as_ref().unwrap_or(&default_code),
                    None => &default_code,
                },
                flight.call_sign
            );
            let default_no_ign = "No IGN".to_string();
            let pilot_output = format!(
                "Pilot: {}",
                all_pilot_ign_map
                    .get(&flight.pilot_id)
                    .unwrap_or(&default_no_ign)
            );
            let current_status = format!(
                "Current Status: **{}**",
                match flight.status {
                    FlightStatus::InFlight => "In Progress",
                    FlightStatus::Amb => "Ambulanced",
                    FlightStatus::Canceled => "Canceled",
                    FlightStatus::Crashed => "Crashed",
                    FlightStatus::Landed => "Landed Safely",
                }
            );
            let passenger_out = format!(
                "{}",
                match &flight.passenger_id {
                    Some(p) => format!(
                        "Passenger: {}",
                        all_clanmember_ign_map.get(p).unwrap_or(&default_no_ign)
                    ),
                    None => "Solo Flight".to_string(),
                }
            );
            let duration_readable = match &flight.end_time {
                Some(t) => format_duration(
                    chrono::Duration::seconds(t.timestamp() - &flight.start_time.timestamp())
                        .to_std()?,
                )
                .to_string(),
                None => format_duration(
                    chrono::Duration::seconds(
                        Utc::now().timestamp() - &flight.start_time.timestamp(),
                    )
                    .to_std()?,
                )
                .to_string(),
            };
            f.fields(vec![
                (full_flight_call_sign, pilot_output, false),
                (passenger_out, current_status, true),
                ("Duration:".to_string(), duration_readable, true),
            ]);
        }

        flight_output += 1;
    }

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("Flights summary for {}", cb_info.name))
                    .description(format!("Total flights this CB: {}", &all_flights.len()))
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

    MenuPaginator::new(ctx, msg, flight_embeds).start().await?;

    // Ask the user for its favorite animal
    // let m = msg
    //     .channel_id
    //     .send_message(&ctx, |m| {
    //         m.content("Please select your favorite animal")
    //             .components(|c| c.add_action_row(Animal::action_row()))
    //     })
    //     .await
    //     .unwrap();

    // Wait for the user to make a selection
    // let mci = match m
    //     .await_component_interaction(&ctx)
    //     .timeout(Duration::from_secs(60 * 3))
    //     .await
    // {
    //     Some(ci) => ci,
    //     None => {
    //         m.reply(&ctx, "Timed out").await.unwrap();
    //         return Ok(());
    //     }
    // };

    // data.custom_id contains the id of the component (here "animal_select")
    // and should be used to identify if a message has multiple components.
    // data.values contains the selected values from the menu
    // let animal = Animal::from_str(mci.data.values.get(0).unwrap()).unwrap();

    /*
    msg.channel_id
    .say(&ctx.http, "Current flights: None")
    .await?;

    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.content("Rong ATC (Air Traffic Control) Status")
                .embed(|e| {
                    e.title("Current Flights")
                        .description("These are the recent running/landed flights.")
                        //.image("attachment://KyoukaSmile.jpg")
                        .fields(vec![
                            ("🛫 __Flight DB 14002__", "Pilot: Dabomstew", false),
                            ("Current Status:", "**In Progress**", true),
                            ("Duration:", "37 Minutes", true),
                        ])
                        .fields(vec![
                            ("🛬 __Flight BN 14002__", "Pilot: Boon", false),
                            ("Current Status:", "**Landed**", true),
                            ("Duration:", "09 Minutes", true),
                        ])
                        .fields(vec![
                            ("💥 __Flight RG 14001__", "Pilot: Ring", false),
                            ("Current Status:", "**Crashed**", true),
                            ("Duration:", "23 Minutes", true),
                        ])
                        .field("Overall Flight Status", "Flights Today: 2", false)
                        .footer(|f| f.text("Days since last int: 0"))
                        // Add a timestamp for the current time
                        // This also accepts a rfc3339 Timestamp
                        .timestamp(chrono::Utc::now().to_rfc3339())
                })
            //.add_file("./KyoukaSmile.jpg")
        })
        .await;

    if let Err(why) = msg {
        println!("Error sending message: {:?}", why);
    }
    */

    Ok(())
}
