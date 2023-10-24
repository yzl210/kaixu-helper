use lazy_static::lazy_static;
use serenity::{async_trait, Client};
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::GatewayIntents;
use tokio::sync::RwLock;

use crate::config::CONFIG;

struct Handler;

lazy_static! {
    static ref CONTEXT: RwLock<Option<Context>> = RwLock::new(None);
}
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("{} says: {}", msg.author.name, msg.content);
        if msg.author.bot {
            return;
        }

        for rule in CONFIG.read().await.reply_rules.iter() {
            if rule.check(&msg) {
                if let Err(why) = msg.reply(&ctx.http, rule.reply.as_str()).await {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }


    async fn ready(&self, ctx: Context, ready: Ready) {
        CONTEXT.write().await.replace(ctx);
        println!("{} is connected!", ready.user.name);
    }
}


pub(crate) async fn start() {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let token = CONFIG.read().await.discord_token.clone();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Error creating client: {:?}", why);
    }
}


pub(crate) async fn send_message(title: &str, msg: &str, color: u32) {
    let channel = ChannelId::from(CONFIG.read().await.steam_status_discord_channel_id);
    if let Some(ctx) = CONTEXT.read().await.as_ref() {
        if let Err(why) = channel.send_message(&ctx.http, |builder| {
            builder.embed(|embed| {
                embed.title(title)
                    .description(msg)
                    .color(color)
            })
        }).await {
            println!("Error sending message: {:?}", why);
        }
    }
}

