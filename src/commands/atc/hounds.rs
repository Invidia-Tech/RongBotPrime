use std::collections::HashSet;

use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

use crate::{
    data::ChannelPersona,
    utils::{
        atc::*,
        clan::*,
        macros::*,
    },
};

#[command("atc_hounds")]
#[aliases("hounds")]
#[description("Calls for all active pilots to release their flights.")]
async fn atc_hounds(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Only allow this command to be called within cb clan contexts.
    let (clan_id, clan_name) = result_or_say_why!(
        get_clan_from_channel_context(ctx, msg, ChannelPersona::Cb),
        ctx,
        msg
    );

    let (cb_info, cb_status) =
        result_or_say_why!(get_latest_cb(ctx, &clan_id, &clan_name), ctx, msg);

    let all_in_air_flights =
        result_or_say_why!(get_all_in_air_flights(ctx, &clan_id, &cb_info.id), ctx, msg);

    if all_in_air_flights.is_empty() {
        msg.channel_id
            .say(ctx, "There are no active flights currently.")
            .await?;
        return Ok(());
    }

    let all_pilot_discord_map =
        result_or_say_why!(get_all_pilot_discord_map(ctx, &clan_id), ctx, msg);

    let mut pilot_pings: HashSet<String> = HashSet::new();

    for flight in all_in_air_flights {
        let default_no_discord = "none".to_string();
        let pilot_discord = (all_pilot_discord_map
            .get(&flight.pilot_id)
            .unwrap_or(&default_no_discord))
        .to_string();
        if pilot_discord != "none" {
            pilot_pings.insert(pilot_discord);
        }
    }

    if pilot_pings.is_empty() {
        msg.channel_id
            .say(ctx, "Could not find pilots' discord info.")
            .await?;
        return Ok(());
    }

    let mut out_msg = "**RELEASE THE HOUNDS**\n".to_string();
    for ping in pilot_pings {
        out_msg.push_str(&format!("<@{}> ", ping));
    }

    msg.channel_id
        .send_message(&ctx, |m| {
            m.content(out_msg).add_embed(|e| {
                e.image("https://c.tenor.com/t5JELdbvC3cAAAAd/release-the-hounds-the-simpsons.gif")
            })
        })
        .await?;

    Ok(())
}
