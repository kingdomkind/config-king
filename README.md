# config-king
An easy way to manage your arch-linux system.

## What Is It?
config-king is a package which allows you to manage your arch (or arch based) systems through a configuration file written in lua, similar to how it works on nixos (but significantly more limited in scope).

## What are the limitations?
config-king is only intended to allow you to install packages from the Arch repos, AUR and flatpaks. Beyond this, the main other feature it provides is the ability to define symlinks, which is the main way you should customise your arch linux installation using this. Ie, if you install package_x that has a configuration file, you could define a symlink from the package_x config location to your main configuration folder.

## Wiki
To be written when the project has reached version 1 (i.e when it becomes stable)

## Building
Simply clone the repo (git clone https://github.com/kingdomkind/config-king.git), then run "sudo cargo run" in the main project directory. You will need to have rust already installed since this is from source.

# STILL IN DEVELOPMENT, NOT READY FOR ACTUAL USAGE - NOT ALL PLANNED FEATURES IMPLEMENTED
