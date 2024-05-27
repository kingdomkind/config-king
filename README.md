# config-king
An easy way to manage your arch-linux system.

## What Is It?
config-king is a package which allows you to manage your arch (or arch based) systems through a configuration file written in lua, similar to how it works on nixos (but significantly more limited in scope).

## What are the limitations / capabilities?
config-king is only intended to allow you to install packages from the Arch repos, AUR and flatpaks. Beyond this, the main other feature it provides is the ability to define symlinks, which is the main way you should customise your arch linux installation using this. Ie, if you install package_x that has a configuration file, you could define a symlink from the package_x config location to your main configuration folder.

Technical Limitations:
- When using symlinks, the directories involved must not have any double quotation marks in them ("). (likely overcomeable in future, if someone desires this functionality).
- Unable to support package groups, each individual package needs to be explicitly installed (not sure if it is possible to overcome, as pacman does not keep logs of where a package was installed from, ie. if it was installed via a group, to my knowledge).

## Wiki
To be written when the project has reached version 1 (i.e when it becomes stable). The example config file is pretty self explanatory however and short.

## Building
Simply clone the repo (git clone https://github.com/kingdomkind/config-king.git), then run "sudo cargo run" in the main project directory. You will need to have rust already installed since this is from source.

# STILL IN DEVELOPMENT, NOT READY FOR ACTUAL USAGE - NOT ALL PLANNED FEATURES IMPLEMENTED
--> As of this commit, i will begin running this on my own system (eating my own "dogfood"), all commits beyond this point will henceforth be properly named and come with their respective changes. The previous commit history is erratic as i was testing on a virtual machine and had to push and pull from github to test anything. If anyone does want to use this, especially before the first release where I consider this to be reliable, please familiarise yourself with the codebase, it's quite short and i've left quite a few comments everywhere, it's important you know how it works so when it doesn't you can know what went wrong. Furthermore, to prevent any major damage, when the program tries to remove above 5 packages it asks for confirmation, and if it tries to remove a directory it lists the directory it wants to remove then asks for confirmation. Although this can be tedious, it's very needed to ensure things don't go boom (although i've never had an instance where i needed to decline it). 
