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
        "fastfetch",
    },

    Aur = {
        {GlobalInstallLocation = "/home/pika/.config-king/aur/"},
        "downgrade",
    },

    Flatpak = {
    },
}

Symlinks = {
    --["/home/pika/test2"] = "/home/pika/test",
    ["/home/pika/test3"] = "/home/pika/test"
}