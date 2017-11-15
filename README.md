# discord-custom-commands
[![Build Status](https://travis-ci.org/drklee3/discord-custom-commands.svg?branch=master)](https://travis-ci.org/drklee3/discord-custom-commands)

A custom commands bot for [Discord](https://discordapp.com/) written in [Rust](https://www.rust-lang.org/) with [serenity-rs](https://github.com/zeyla/serenity).  Commands are stored in a SQLite database using [rusqlite](https://github.com/jgallagher/rusqlite).

Mainly for learning purposes, so there's probably a lot of weird stuff going on in here.

# Commands
```
Meta
  ~help [command]
    Shows the help message.
  ~ping
    Pong!
  ~latency
    Calculates the heartbeat latency between the shard and the gateway.
  ~info
    Gives info about the bot.
  ~shutdown
    Gracefully shuts down the bot. (owners only)

Custom Commands
  ~commands
    Lists all available commands
  ~top
    Lists the top 10 most used commands
  ~add [name] [url]
    Adds a custom command
  ~delete [name]
    Deletes a custom command.
    Limited to the creator of a command or members with MANAGE_GUILD permissions.
  ~edit [name] [new name] [new url]
    Edits an existing command.
    Limited to the creator of a command or members with MANAGE_GUILD permissions.
  ~stat [name]
    Shows informatino about a custom command.
  ~search [name]
    Searches for a custom command.
  ~import [json]  (or attach a JSON file)
    Imports command from json file or message.  Deletes all existing commands.

Misc
  ~play [code block]
    Evaluates Rust code in the playground.
```