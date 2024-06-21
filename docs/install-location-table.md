# InstallLocations Table

### Example Definitions:
```lua
InstallLocations = {

}
```

```lua
InstallLocations = {
    ["Base"] = "/home/pika/.config-king/",
    ["Aur"] = "/home/pika/.config-king/aur/"
}
```

**All keys are required.**
The ["Base"] sets the directory that config-king should put generic files, such as save.king (the temporary save file, to retain information between runs).
The ["Aur"] sets the directory that config-king should put the folders for AUR applications that it attempts to install, and this should automatically be cleaned up as AUR packages are added and removed.

*Please note that changing the values of the keys will not remove the previous location, nor transfer other content (eg. save.king), so ideally should just be set upon original run, and not really changed, although the functionality is still there if one needs it.*

[Back to Index](https://github.com/kingdomkind/config-king/blob/main/docs/index.md)
