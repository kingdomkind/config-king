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
        "neofetch",
        "gnome",
    },

    Aur = {
        {GlobalInstallLocation = "/home/pika/.config-king/aur/"},
        --"downgrade",
        "yay"
    },

    Flatpak = {
    },
}