#[macro_use]
extern crate serenity;
extern crate dotenv;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rusqlite;
extern crate time;
extern crate typemap;

mod commands;
mod sqlite;

use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::StandardFramework;
use serenity::framework::standard::help_commands;
use dotenv::dotenv;
use std::env;
use std::collections::HashMap;
use typemap::Key;
use sqlite::Database;

const PREFIX: &'static str = "~";

struct Handler;

struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

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

    env_logger::init();
    info!("Starting...");

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler);

    {
        let mut data = client.data.lock();
        data.insert::<CommandCounter>(HashMap::default());

        match sqlite::connect() {
            Ok(db) => data.insert::<Database>(db),
            Err(_) => return error!("Failed to connect to database"),
        };
    }

    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix(PREFIX))

        .before(|ctx, msg, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     msg.author.name);

            // Increment the number of times this command has been run once. If
            // the command's name does not exist in the counter, add a default
            // value of 0.
            let mut data = ctx.data.lock();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.to_string()).or_insert(0);
            *entry += 1;

            true // if `before` returns false, command processing doesn't happen.
        })

        .after(|_, _, command_name, error| {
            match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            }
        })

        .command("help", |c| c.exec_help(help_commands::with_embeds))
        .command("ping", |c| c.exec(commands::meta::ping))
        .command("latency", |c| c.exec(commands::meta::latency))
        .command("multiply", |c| c
            .min_args(2)
            .max_args(2)
            .usage("~multiply [number] [number]")
            .exec(commands::math::multiply))
        .command("cmdcount", |c| c.exec(commands::meta::commands))
        .group("Custom Commands", |g| g
            .command("commands", |c| c
                .usage("~commands")
                .description("Lists all available commands")
                .exec(commands::custom_commands::commands))
            .command("add", |c| c
                .usage("~add [name] [url]")
                .description("Adds a custom command")
                .exec(commands::custom_commands::add))
            .command("delete", |c| c
                .usage("~delete [name]")
                .description("Deletes a custom command.  Limited to the
                    creator of a command or members with MANAGE_GUILD permissions.")
                .exec(commands::custom_commands::delete))
            .command("edit", |c| c
                .usage("~edit [name] [new name] [new url]")
                .description("Edits an existing command.  Limited to the
                    creator of a command or members with MANAGE_GUILD permissions.")
                .exec(commands::custom_commands::edit))
            .command("stat", |c| c
                .usage("~stat [name]")
                .description("Shows informatino about a custom command.")
                .exec(commands::custom_commands::stat))
            .command("search", |c| c
                .usage("~search [name]")
                .description("Searches for a custom command.")
                .exec(commands::custom_commands::search)))
        );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
