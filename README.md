# discord-custom-commands
[![Build Status](https://travis-ci.org/drklee3/discord-custom-commands.svg?branch=master)](https://travis-ci.org/drklee3/discord-custom-commands)

A custom commands bot for [Discord](https://discordapp.com/) written in [Rust](https://www.rust-lang.org/) with [serenity-rs](https://github.com/zeyla/serenity).  
Mainly for learning purposes.

# Commands
```
~help [command]
    Shows the help message.
~commands
    Lists all available commands
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
```