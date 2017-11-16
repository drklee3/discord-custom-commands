use sqlite;
use std::fmt::Write;
use serenity::model::Message;
use chrono::prelude::*;
use std::env;
use serde_json;
use serde_json::Value;
use serde_json::Map;
use helpers;


fn has_permission(msg: &Message) -> bool {
    let guild = match msg.guild() {
        Some(guild) => guild,
        None => {
            warn!("Couldn't get message guild!");

            return false;
        }
    };
    let guild = guild.read().unwrap();

    // fetch member
    let member = match guild.members.get(&msg.author.id) {
        Some(member) => member,
        None => return false
    };
    // check if has perm
    if let Ok(permissions) = member.permissions() {
        return permissions.manage_guild();
    } else {
        return false;
    }
}

// splits a string that might be too long
fn split_message(msg: &str, prepend: Option<&str>, with_code_block: bool) -> Vec<String> {
    let split = msg.split("\n");
    let mut vec = Vec::new();
    let mut single_msg = String::new();

    // add text in beginning before code blocks
    match prepend {
        Some(val) => {
            single_msg.push_str(&val);
        },
        None => {},
    };

    if with_code_block {
        single_msg.push_str("\n```"); // add starting code block
    }

    for s in split {
        single_msg.push_str("\n"); // re-add new line at end

        // will overflow, move to next msg (in bytes) + 6 just in case?
        if single_msg.len() + s.len() + 6 > 4000 {
            // add closing code block
            if with_code_block {
                single_msg.push_str("```");
            }

            vec.push(single_msg.clone());   // push the full string to vec
            single_msg.clear();     // reset single message

            // start new code block 
            if with_code_block {
                single_msg.push_str("```\n");
            }
        }

        // append the next line
        single_msg.push_str(s);
    }

    // push the remaining string
    if !single_msg.is_empty() {
        if with_code_block {
            single_msg.push_str("```"); // add closing code block
        }

        vec.push(single_msg);
    }

    vec
}

command!(commands(ctx, msg, _args) {
    let mut data = ctx.data.lock();
    let db = data.get_mut::<sqlite::Database>().unwrap();

    let commands = try!(db.all());

    let mut contents = String::new();
    for cmd in commands {
        let _ = write!(contents, "{}\n", cmd.name);
    }

    let dm = match msg.author.create_dm_channel() {
        Ok(val) => val,
        Err(_) => {
            let _ = msg.channel_id.say("Failed to send DM, maybe you don't have them enabled?");
            return Ok(());
        }
    };

    let messages = split_message(&contents, Some("Available Commands:"), true);

    for msg in messages {
        let _ = dm.say(&msg);
    }

    if !msg.is_private() {
        let _ = msg.channel_id.say(":mailbox_with_mail: Sent you a DM with the commands list.");
    }
});

command!(top(ctx, msg, _args) {
    let mut data = ctx.data.lock();
    let db = data.get_mut::<sqlite::Database>().unwrap();

    let commands = try!(db.top());

    let mut contents = "```Top 10 Most Used Commands:\n".to_string();

    let commands = commands.iter().take(10);

    for cmd in commands {
        let _ = write!(contents, "{} - {}\n", cmd.stat, cmd.name);
    }

    let _ = write!(contents, "```");

    let _ = msg.channel_id.say(&contents);
});

command!(add(ctx, msg, args) {
    let home_guild_id = env::var("HOME_GUILD_ID")
        .unwrap_or("0".to_string()).parse::<u64>().unwrap();

    // limit command adding to the main guild
    if let Some(guild_id) = msg.guild_id() {
        if guild_id.0 != home_guild_id || home_guild_id == 0 {
            let _ = msg.channel_id.say(helpers::get_error("home_guild"));
            return Ok(());
        }
    } else { // return if no guild found (maybe in dms?)
        let _ = msg.channel_id.say(helpers::get_error("home_guild"));
        return Ok(()); 
    }

    let name = match args.single::<String>() {
        Ok(val) => val,
        Err(why) => {
            let _ = msg.channel_id.say(&format!("Error: {}", why));
            return Ok(());
        },
    };

    let url = match args.single::<String>() {
        Ok(val) => val,
        Err(why) => {
            let _ = msg.channel_id.say(&format!("Error: {}", why));
            return Ok(());
        },
    };

    let mut data = ctx.data.lock();
    let db = data.get_mut::<sqlite::Database>().unwrap();

    if !try!(db.is_command(&name)) {
        try!(db.add(&name, &url, msg.author.id.0));
        let _ = msg.channel_id.say(helpers::get_info_f("command_added", &[&name, &url]));
    } else {
        let _ = msg.channel_id.say(helpers::get_error_f("command_exists", &[&name]));
    }
});


command!(delete(ctx, msg, args) {
    let name = match args.single::<String>() {
        Ok(val) => val,
        Err(why) => {
            let _ = msg.channel_id.say(&format!("Error: {}", why));
            return Ok(());
        },
    };

    let mut data = ctx.data.lock();
    let db = data.get_mut::<sqlite::Database>().unwrap();

    if try!(db.is_command(&name)) {
        let cmd = try!(db.get(&name));

        if cmd.is_owner(msg.author.id.0) || has_permission(msg) {
            try!(db.delete(&name));
            let _ = msg.channel_id.say(helpers::get_info_f("command_deleted", &[&name]));
        }
    } else {
        let _ = msg.channel_id.say(helpers::get_error_f("command_not_found", &[&name]));
    }  
});

command!(edit(ctx, msg, args) {
    let name = match args.single::<String>() {
        Ok(val) => val,
        Err(why) => {
            let _ = msg.channel_id.say(&format!("Error: {}", why));
            return Ok(());
        },
    };

    let new_name = match args.single::<String>() {
        Ok(val) => val,
        Err(why) => {
            let _ = msg.channel_id.say(&format!("Error: {}", why));
            return Ok(());
        },
    };

    let new_url = match args.single::<String>() {
        Ok(val) => val,
        Err(why) => {
            let _ = msg.channel_id.say(&format!("Error: {}", why));
            return Ok(());
        },
    };

    let mut data = ctx.data.lock();
    let db = data.get_mut::<sqlite::Database>().unwrap();

    // check if command exists
    if try!(db.is_command(&name)) {
        let cmd = try!(db.get(&name));

        // check permissions
        if cmd.is_owner(msg.author.id.0) || has_permission(msg) {
            // check if new name conflicts
            if try!(db.is_command(&new_name)) {
                let _ = msg.channel_id.say(helpers::get_error_f("command_exists", &[&new_name]));
            } else {
                try!(db.edit(&name, &new_name, &new_url));
                let _ = msg.channel_id.say(helpers::get_info_f("command_updated", &[&name, &new_name, &new_url]));
            }
        } else { // no permissions
            let _ = msg.channel_id.say(helpers::get_error("command_edit_no_permission"));
        }
    } else {
        let _ = msg.channel_id.say(helpers::get_error_f("command_not_found", &[&name]));
    }  
});

command!(stat(ctx, msg, args) {
    let name = match args.single::<String>() {
        Ok(val) => val,
        Err(why) => {
            let _ = msg.channel_id.say(&format!("Error: {}", why));
            return Ok(());
        },
    };

    let mut data = ctx.data.lock();
    let db = data.get_mut::<sqlite::Database>().unwrap();

    let cmd = match db.get(&name) {
        Ok(val) => val,
        Err(e) => {
            let _ = msg.channel_id.say(&format!("Error: {}", e));
            return Ok(());
        }
    };

    let timestamp = Utc.timestamp(cmd.created as i64, 0).format("%Y-%m-%d %H:%M:%S").to_string();

    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .title(format!("Stats for {}", name))
            .field(|f| f
                .name("Response")
                .value(&cmd.url)
                .inline(false)
            )
            .field(|f| f
                .name("Times used")
                .value(&cmd.stat)
            )
            .field(|f| f
                .name("Added on")
                .value(timestamp)
            )
            .field(|f| f
                .name("Added by")
                .value(format!("<@{}>", &cmd.owner)
            ))));
});

command!(search(ctx, msg, args) {
    let search = match args.single::<String>() {
        Ok(val) => val,
        Err(why) => {
            let _ = msg.channel_id.say(&format!("Error: {}", why));
            return Ok(());
        },
    };

    let mut data = ctx.data.lock();
    let db = data.get_mut::<sqlite::Database>().unwrap();

    let results = try!(db.search(&search));

    let mut contents = String::new();

    if results.len() == 0 {
        let _ = msg.channel_id.say(helpers::get_error("search_no_results"));
        return Ok(());
    }

    for cmd in results {
            let _ = write!(contents, "{}\n", name=cmd.name);
    }

    let messages = split_message(&contents, Some("Search Results:"), true);

    // only 1 short message
    if messages.len() == 1 && messages[0].len() <= 300 {
        let _ = msg.channel_id.say(&messages[0]);
        return Ok(());
    }

    let dm = match msg.author.create_dm_channel() {
        Ok(val) => val,
        Err(_) => {
            let _ = msg.channel_id.say("Failed to send DM, maybe you don't have them enabled?");
            return Ok(());
        }
    };

    // send results
    for msg in messages {
        let _ = dm.say(&msg);
    }

    // send notification in channel
    if !msg.is_private() {
        let _ = msg.channel_id.say(":mailbox_with_mail: High number of search results, sent you a DM.");
    }
});


#[derive(Serialize, Deserialize)]
struct Command {
    commands: Map<String, Value>,
}

command!(import(ctx, msg, args) {
    let mut raw_json = args.full();

    // try reading file
    if raw_json.is_empty() && msg.attachments.len() > 0 {
        let json_bytes = match msg.attachments[0].download() {
            Ok(content) => content,
            Err(why) => {
                // println!(helpers::get_error_f("download_attachment", &[why]).to_string());
                let _ = msg.channel_id.say(helpers::get_error_f("download_attachment", &[&why.to_string()]));

                return Ok(());
            },
        };

        raw_json = match String::from_utf8(json_bytes) {
            Ok(content) => content,
            Err(why) => {
                let _ = msg.channel_id.say(helpers::get_error_f("utf8", &[&why.to_string()]));

                return Ok(());
            }
        };
    }

    let imported: Command = match serde_json::from_str(&raw_json) {
        Ok(val) => val,
        Err(why) => {
            let _ = msg.channel_id.say(helpers::get_error_f("json", &[&why.to_string()]));
            return Ok(());
        }
    };

    let mut data = ctx.data.lock();
    let db = data.get_mut::<sqlite::Database>().unwrap();

    let _ = msg.channel_id.say(helpers::get_info_f("import_started", &[&imported.commands.len().to_string()]));
    let mut existing = 0;

    match db.delete_all() {
        Ok(()) => {},
        Err(why) => {
            let _ = msg.channel_id.say(helpers::get_error_f("import_delete_all", &[&why.to_string()]));
            return Ok(());
        }
    };

    for (key, value) in imported.commands.iter() {
        if !try!(db.is_command(&key)) {
            try!(db.add(&key, &value.as_str().unwrap().to_string(), msg.author.id.0));
        } else {
            existing += 1;
        }
    }

    if existing > 0 {
        let _ = msg.channel_id.say(helpers::get_error_f("import_existing", &[&existing.to_string()]));
    } else {
        let _ = msg.channel_id.say(helpers::get_info("import_finished"));
    }
});