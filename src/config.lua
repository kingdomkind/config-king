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
        "neofetch",
        "konsole",
        "ttf-jetbrains-mono-nerd",
        "flatpak",
        "hyprland",
    },

    Aur = {
        {GlobalInstallLocation = "/home/pika/.config-king/aur/"},
        downgrade
    },

    Flatpak = {
        "org.gimp.GIMP",
    },
}