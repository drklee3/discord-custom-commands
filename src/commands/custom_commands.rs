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

command!(commands(ctx, msg, _args) {
    let mut data = ctx.data.lock();
    let db = data.get_mut::<sqlite::Database>().unwrap();

    let commands = try!(db.all());

    let mut contents = "```Available Commands:\n".to_string();
    for cmd in commands {
            let _ = write!(contents, "{}\n", cmd.name);
    }

    let _ = write!(contents, "```");

    let _ = msg.channel_id.say(&contents);
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
    } else { // return if no guild found
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
                    .name("Url")
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

    let mut contents = "```Search Results:\n".to_string();

    if results.len() == 0 {
        let _ = msg.channel_id.say(helpers::get_error("search_no_results"));
        return Ok(());
    }

    for cmd in results {
            let _ = write!(contents, "- {}\n", name=cmd.name);
    }

    let _ = write!(contents, "```");

    let _ = msg.channel_id.say(&contents);
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