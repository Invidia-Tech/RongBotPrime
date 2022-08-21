use crate::data::DatabasePool;

use serenity::{
    client::Context,
    framework::standard::{macros::check, Args, CommandOptions, Reason},
    model::channel::Message,
};

#[check]
#[name = "RongAdmin"]
async fn rong_admin_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    // Ring/Dabo admin aboose
    if msg.author.id == 162034086066520064 || msg.author.id == 79515100536385536 {
        return Ok(());
    }

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let user_admin_status = match sqlx::query!(
        "SELECT is_superadmin
         FROM public.rong_user
         WHERE platform_id = $1;",
        msg.author.id.to_string()
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row.is_superadmin,
        Err(_) => {
            return Err(Reason::User(
                "You're definitely not a Rong superadmin! <:Suzunaaaaaaaaa:914426187378466836>"
                    .to_string(),
            ));
        }
    };

    if !user_admin_status {
        return Err(Reason::User(
            "Nice try, but you are not a rong superadmin! <:YuiBat:964435494509346897>".to_string(),
        ));
    }

    Ok(())
}
