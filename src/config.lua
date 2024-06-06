--> Example confirguration with some defaults people may need, demonstrates all possible options

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
        "gst-plugin-pipewire",
        "pipewire",
        "pipewire-alsa",
        "pipewire-jack",
        "pipewire-pulse",
        "wireplumber",
        "zram-generator",
    },

    Aur = {
        --> Regular Example
        "downgrade",
        --> Example where the base package installs additional packages
        --> Base pkg name first     then the additional packages to install follows (ie. the other ones in the pkg name)
        {"nvidia-utils-beta",       "opencl-nvidia-beta", "nvidia-settings-beta"},
    },

    Flatpak = {
    },
}

Symlinks = {
	--> Example
	--  Link Directory	    Original Directory
	-- ["/etc/default/grub"] = "/home/user/myawesomeconfigfolder/grubconf"
}

--> Change this to your username instead!
InstallLocations = {
    ["Save"] = "/home/pika/.config-king/save.king",
    ["Aur"] = "/home/pika/.config-king/aur/"
} 
