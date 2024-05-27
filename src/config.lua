Packages = {
    Official = {
        "nano",
        "base",
        "base-devel",
        "git",
        "grub",
        "linux",
        "linux-firmware",
        "networkmanager",
        "openssh",
        "rustup",
        "ttf-jetbrains-mono-nerd",
        "flatpak",
        --"fastfetch",
    },

    Aur = {
        --"downgrade",
    },

    Flatpak = {
    },
}

Symlinks = {
    ["/home/pika/test2"] = "/home/pika/test2",
    --["/home/pika/test3"] = "/home/pika/config-king",
}

InstallLocations = {
    ["Save"] = "/home/pika/.config-king/save.king",
    ["Aur"] = "/home/pika/.config-king/aur/"
}