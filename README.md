# config-king
An easy way to declaratively manage your arch system.

## What Is It?
config-king is a script which allows you to manage your arch (or arch based) systems through a configuration file written in lua, similar to how it works on nixos (but significantly more limited in scope). It is only intended to allow you to install packages from arch repos, AUR, flatpaks and any custom git links with PKGBUILDs, although more functionality for build hooks is planned in future so the user can customise it with lua.

Alongside this, the other main feature is to allow you to create custom symlinks to help you manage your configuration. This is the main way config-king intends you customise your arch linux installation with. Ie, if you install package_x that has a configuration file, you could define a symlink from the package_x config location to your main configuration folder.

## Limitations
- When using symlinks, the directories / paths involved must not have any double quotation marks in them ("). (likely overcomeable in future, if someone desires this functionality).
- Unable to support package groups, each individual package needs to be explicitly installed (not sure if it is possible to overcome, as pacman does not keep logs of where a package was installed from, ie. if it was installed via a group, to my knowledge).
- When specifying urls for custom git links, changing the url will not update that package, as the url is only used for the original clone. Simply remove the entry from the table, rebuild, and then put the new entry on the table
- config-king is still in development and so breaking changes will be made where necessary, and will be reflected in an updated config.lua file demonstrating the new syntax, although it is fairly stable now. The top of main.rs has TODOs that are currently being worked upon.

## How to use
See [config.lua](https://github.com/kingdomkind/config-king/blob/main/src/config.lua) for examples for how to use everything. There are comments explaining each section, although it's fairly self explanatory.

## Arguments
- directory=whatever/directory/you/want/config.lua -> changes the directory the config.lua file is sourced. If not specified, it will be assumed to be in the same directory as main.rs.

## Building / Installation
Simply clone the repo (git clone https://github.com/kingdomkind/config-king.git), then run "cargo run" in the main project directory. You will need to have rust already installed since this is from source. For long term use, you can add something similar to the following in your bashrc or other shell language:

```bash
alias build-config='cargo run --manifest-path /home/user/path/to/cloned/config-king/Cargo.toml --directory=/home/user/your-config-repo/config.lua'
```
