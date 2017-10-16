#[macro_use]
extern crate serenity;
extern crate dotenv;

#[macro_use]
extern crate log;
extern crate env_logger;
// extern crate rusqlite;

mod commands;

use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::StandardFramework;
use dotenv::dotenv;
use std::env;
// use rusqlite::Connection;

struct Handler;

// struct CustomCommand {
//     name: String,
//     url: String,
//     owner: u64,
//     stat: u32,
//     created: u32
// }

impl EventHandler for Handler {
    fn on_message(&self, _: Context, msg: Message) {
        if msg.content == "hello" {
            if let Err(why) = msg.channel_id.say("Hello!") {
                error!("Error when sending message: {:?}", why);
            }
        }
    }

    fn on_ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

fn main() {
    dotenv().ok();

    env_logger::init();
    info!("Starting...");

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler);

    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .command("ping", |c| c.exec(commands::meta::ping))
        .command("latency", |c| c.exec(commands::meta::latency))
        .command("multiply", |c| c
            .min_args(2)
            .max_args(2)
            .usage("~multiply [number] [number]")
            .exec(commands::math::multiply))
        .command("add", |c| c
            .exec(commands::custom_commands::add)));

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
