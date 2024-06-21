# Flatpak Table

### Example Definitions:
```lua
Packages = {
    Flatpak = {

    }
}
```

```lua
Packages = {
    Flatpak = {
        "com.bitwarden.desktop",
        "org.gimp.GIMP",
    }
}
```

### Element Types:

**Regular Package:**
```lua
"Application-ID-name-here",
```
Simply put the application ID (the last part of an url on flathub).