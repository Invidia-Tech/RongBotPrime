use serenity::{
    client::Context,
    framework::standard::{macros::check, Args, CommandOptions, Reason},
    model::channel::Message,
};

use crate::checks::util::{dabo_ring, rong_admin};
use tokio::join;

#[check]
#[name = "RongAdmin"]
async fn rong_admin_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let (d, r) = join!(dabo_ring(msg), rong_admin(ctx, msg));
    let results = vec![d, r];
    for r in results {
        match r {
            Err(e) => return Err(e),
            _ => continue,
        };
    }

    Ok(())
}
