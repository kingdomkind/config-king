# Symlinks Table

### Example Definitions:
```lua
Symlinks = {

}
```

```lua
Symlinks = {
    ["/home/pika/.config/hypr/hyprland.conf"] = "/home/pika/Config/hyprland.conf",
    ["/etc/default/grub"] = "/home/pika/Config/boot/grub",
    ["/etc/fstab"] = "/home/pika/Config/boot/fstab",
}
```

**Symlink:**
```lua
["/Directory/the/link/file/should/be/placed"] = "/the/directory/of/the/file/or/folder/to/be/mirrored/from"
```
The key is the directory the link file is placed (ie. the fake clone) and the value is the real folder / file. Although I have seen some programs do it the opposite way around, I like to think of it like the directory of X is being set to Y.