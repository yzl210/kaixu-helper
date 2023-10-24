use poise::{builtins, send_reply};
use serenity::model::id::GuildId;

use crate::config;
use crate::config::CONFIG;
use crate::steam::STATUSES;

struct Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub(crate) async fn start() {
    let token = CONFIG.read().await.discord_token.clone();
    let guild_id = CONFIG.read().await.discord_guild_id;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![game_playing(), add_account()],
            ..Default::default()
        })
        .token(token)
        .intents(serenity::prelude::GatewayIntents::non_privileged())
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                builtins::register_in_guild(ctx, &framework.options().commands, GuildId(guild_id)).await?;
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}


#[poise::command(slash_command)]
async fn game_playing(
    ctx: Context<'_>
) -> Result<(), Error> {
    let lines: Vec<String> = STATUSES.lock().await.values().map(|player| {
        let game_info = player.game_info.clone().unwrap_or_else(|| "None".to_string());
        format!("{}: {}", player.profile_name, game_info)
    }).collect();
    send_reply(ctx, |reply| {
        reply.embed(|embed| {
            embed.title("Game Playing")
                .description(lines.join("\n"))
                .color(0x32cd32)
        })
    }).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn add_account(
    ctx: Context<'_>,
    #[description = "Steam ID"] steam_id: String,
) -> Result<(), Error> {
    {
        let mut config = CONFIG.write().await;
        config.steam_ids.push(steam_id.parse::<u64>().unwrap());
    }
    config::save().await;

    ctx.reply(format!("Added {} to the list", steam_id)).await?;
    Ok(())
}
