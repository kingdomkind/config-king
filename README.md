# config-king
An easy way to declaratively manage your arch system.

## What Is It?
config-king is a script which allows you to manage your arch (or arch based) systems through a configuration file written in lua, similar to how it works on nixos (but significantly more limited in scope). It is only intended to allow you to install packages from arch repos, AUR, flatpaks and any custom git links with PKGBUILDs.

Alongside this, the other main feature is to allow you to create custom symlinks to help you manage your configuration, eg. by linking configuration directories to a central place.

## Things you should know
- We are unable to support package groups, each individual package needs to be explicitly installed (not sure if it is possible to overcome, as pacman does not keep logs of where a package was installed from, ie. if it was installed via a group, to my knowledge). Packages from package groups will all explicitly need to be stated.
- breaking changes will be made where necessary, and will be reflected in an updated config.lua file demonstrating the new syntax alongside the wiki,although it is fairly stable now.

## How to use / configure
See the [docs / wiki!](https://github.com/kingdomkind/config-king/blob/main/docs/index.md).
Alternatively, you can see the [config I use](https://github.com/kingdomkind/pika-archconfig).

## Building / Installation
Simply clone the repo (git clone https://github.com/kingdomkind/config-king.git), then run "cargo run" in the main project directory. You will need to have rust already installed since this is from source. For long term use, you can add something similar to the following in your bashrc or other shell language (customising it to your needs):

```bash
alias build-config='cargo run --manifest-path /home/user/path/to/cloned/config-king/Cargo.toml -- DIRECTORY=/home/user/your-config-repo/config.lua'
```
