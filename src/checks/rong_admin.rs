use crate::data::DatabasePool;

use serenity::{
    client::Context,
    framework::standard::{
        Args,
        CommandOptions,
        macros::check,
        Reason,
    },
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
    let pool = ctx.data.read().await.get::<DatabasePool>().cloned().unwrap();

    let user_admin_status =
        match sqlx::query!(
            "SELECT is_superadmin
             FROM public.rong_user
             WHERE platform_id = $1;",
            msg.author.id.to_string())
        .fetch_one(&pool)
        .await {
            Ok(row) => row.is_superadmin,
            Err(_) => {
                return Err(Reason::User("You do not exist within rong's database.".to_string()));
            }
        };

    if !user_admin_status {
        return Err(Reason::User("You are not a rong superadmin.".to_string()));
    }
    Ok(())
}
