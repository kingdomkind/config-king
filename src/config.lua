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
        "fastfetch"
    },

    Aur = {
        {GlobalInstallLocation = "/home/pika/.config-king/aur/"},
        "downgrade",
        --"hyprland-git"
    },

    Flatpak = {
        --"org.gimp.GIMP",
    },
}