#[macro_use]
extern crate serenity;
extern crate dotenv;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rusqlite;
extern crate chrono;
extern crate typemap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod commands;
mod sqlite;

use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::StandardFramework;
use serenity::framework::standard::help_commands;
use dotenv::dotenv;
use std::env;
use typemap::Key;
use sqlite::Database;

const PREFIX: &'static str = "~";

struct Handler;

impl Key for Database {
    type Value = Database;
}

impl EventHandler for Handler {
    fn on_message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with(PREFIX) {
            let mut data = ctx.data.lock();
            let db = data.get_mut::<sqlite::Database>().unwrap();

            let command = match db.get(&msg.content[1..].to_string()) {
                Ok(val) => val,
                _ => {
                    // no custom command found
                    return;
                },
            };

            match db.increment(&command) {
                Ok(_) => {},
                Err(why) => error!("Error occurred when incrementing custom command count: {}", why),
            }

            println!("Got custom command '{}' by user '{}'",
                     command.name, msg.author.name);

            if let Err(why) = msg.channel_id.say(command.url) {
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

    let _ = env_logger::init();
    info!("Starting...");

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler);

    {
        let mut data = client.data.lock();

        match sqlite::connect() {
            Ok(db) => data.insert::<Database>(db),
            Err(_) => return error!("Failed to connect to database"),
        };
    }

    let invite_link = env::var("INVITE_LINK")
        .expect("Expected an invite link in the environment");

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .prefix(PREFIX)
            .owners(vec![UserId(150443906511667200)].into_iter().collect()))

        .before(|_ctx, msg, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     msg.author.name);

            true // if `before` returns false, command processing doesn't happen.
        })

        .after(|_, _, command_name, error| {
            match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            }
        })

        .group("Meta", |g| g
            .command("help", |c| c.exec_help(help_commands::with_embeds))
            .command("ping", |c| c.exec_str("Pong!"))
            .command("latency", |c| c
                .desc("Calculates the heartbeat latency between the shard and the gateway.")
                .exec(commands::meta::latency))
            .command("info", |c| c
                .desc("Gives info about the bot.")
                .exec_str(&format!("Hi!  I'm a bot written by tzuwy#7080 with Rust and serenity-rs.\n\
                    If you'd like to add me to another server, here's an invite link: <{}>\n\
                    Commands can be only added in the BLACKPINK server though!", invite_link)))
            .command("shutdown", |c| c
                .desc("Gracefully shuts down the bot.")
                .owners_only(true)
                .exec(commands::meta::shutdown)))
        .group("Custom Commands", |g| g
            .command("commands", |c| c
                .desc("Lists all available commands")
                .exec(commands::custom_commands::commands))
            .command("top", |c| c
                .desc("Lists the top 10 most used commands")
                .exec(commands::custom_commands::top))
            .command("add", |c| c
                .usage("[name] [url]")
                .desc("Adds a custom command")
                .exec(commands::custom_commands::add))
            .command("delete", |c| c
                .usage("[name]")
                .desc("Deletes a custom command.  Limited to the
                    creator of a command or members with MANAGE_GUILD permissions.")
                .exec(commands::custom_commands::delete))
            .command("edit", |c| c
                .usage("[name] [new name] [new url]")
                .desc("Edits an existing command.  Limited to the
                    creator of a command or members with MANAGE_GUILD permissions.")
                .exec(commands::custom_commands::edit))
            .command("stat", |c| c
                .usage("[name]")
                .desc("Shows information about a custom command.")
                .exec(commands::custom_commands::stat))
            .command("search", |c| c
                .usage("[name]")
                .desc("Searches for a custom command.")
                .exec(commands::custom_commands::search))
            .command("import", |c| c
                .usage("[json data]")
                .desc("Imports commands from json.")
                .owners_only(true)
                .exec(commands::custom_commands::import)))
        );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
