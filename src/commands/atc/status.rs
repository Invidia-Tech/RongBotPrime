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
        self.edit_message(&mut message, true).await?;
        Ok(())
    }
}

#[command("atc_status")]
#[aliases("status", "s")]
#[description("This shows the status of current most important flights.")]
async fn flight_status(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let (clan_id, clan_name) = result_or_say_why!(
        get_clan_from_channel_context(ctx, msg, ChannelPersona::Cb),
        ctx,
        msg
    );

    let (cb_info, _) = result_or_say_why!(get_latest_cb(ctx, &clan_id, &clan_name), ctx, msg);

    // let pilot_user_id =
    //     result_or_say_why!(get_user_id(ctx, msg, &msg.author.id.to_string()), ctx, msg);

    let all_in_air_flights =
        result_or_say_why!(get_all_in_air_flights(ctx, &clan_id, &cb_info.id), ctx, msg);

    if all_in_air_flights.is_empty() {
        msg.channel_id
            .say(ctx, "There are no active flights currently.")
            .await?;
        return Ok(());
    }

    // Time to output flights.
    let all_pilot_info_map = result_or_say_why!(get_all_pilot_info_map(ctx, &clan_id), ctx, msg);

    let all_pilot_ign_map = result_or_say_why!(get_all_pilot_ign_map(ctx, &clan_id), ctx, msg);

    let all_clanmember_ign_map =
        result_or_say_why!(get_all_clanmember_ign_map(ctx, &clan_id), ctx, msg);

    // Total flights information
    let mut in_flight_count: u32 = 0;

    let mut flight_embeds: Vec<CreateEmbed> = Vec::default();
    let flights_per_page = 3;
    let mut flight_output = 0;
    for flight in &all_in_air_flights {
        if flight.status == FlightStatus::InFlight {
            in_flight_count += 1;
        }
        if flight_output >= flights_per_page || flight_embeds.is_empty() {
            flight_output = 0;
            let mut new_flight_page = CreateEmbed::default();
            new_flight_page
                .title(format!(
                    "Currently in flight: {}",
                    &all_in_air_flights.len()
                ))
                .footer(|f| f.text("Days since last int: 0"))
                .timestamp(chrono::Utc::now().to_rfc3339());
            flight_embeds.push(new_flight_page);
        }

        // Add flight information
        if let Some(f) = flight_embeds.last_mut() {
            let default_no_ign = "No IGN".to_string();
            let pilot_output = format!(
                "Pilot: {}",
                all_pilot_ign_map
                    .get(&flight.pilot_id)
                    .unwrap_or(&default_no_ign)
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
                    chrono::Duration::seconds(t.timestamp() - flight.start_time.timestamp())
                        .to_std()?,
                )
                .to_string(),
                None => format_duration(
                    chrono::Duration::seconds(
                        Utc::now().timestamp() - flight.start_time.timestamp(),
                    )
                    .to_std()?,
                )
                .to_string(),
            };
            f.fields(vec![(
                format!("{} - {}", pilot_output, passenger_out),
                format!("Duration: {}", duration_readable),
                false,
            )]);
        }

        flight_output += 1;
    }

    // Start paginated flights
    MenuPaginator::new(ctx, msg, flight_embeds).start().await?;

    Ok(())
}
