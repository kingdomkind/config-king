--> Example confirguration with some defaults people may need, demonstrates all possible options

Packages = {
    Official = {
        "vim",
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

        --> Example where the base package installs additional packages (ie. base packages and sub-packages)
        {
            --> Base package name (ie. the one to clone from on the AUR)
            ["base"] = "nvidia-utils-beta",
            --> Sub packages to install from the base package, note the nvidia-utils-beta package is not automatically installed
            ["sub"] = {"nvidia-utils-beta", "opencl-nvidia-beta", "nvidia-settings-beta"},
        },

        --> Example where you want to install a PKGBUILD not from the AUR, but from a custom git link
        {
            --> Base package name (needs to be the same name as the project, ie. the folder name)
            ["base"] = "Rust-VPN-Handler",
            --> Sub packages, as previous example also showed
            ["sub"] = {"vpn_handler"},
            --> url to download from. If it is omitted, it is assumed to be an AUR package
            ["url"] = "https://github.com/kingdomkind/Rust-VPN-Handler.git"
        }
    },

    Flatpak = {
        "com.bitwarden.desktop",
    },
}

Symlinks = {
	--> Example
	--  Link Directory	    Original Directory
	-- ["/etc/default/grub"] = "/home/user/myawesomeconfigfolder/grubconf"
}

--> Change this to your username instead! Must end in slash
InstallLocations = {
    ["Base"] = "/home/pika/.config-king/",
    ["Aur"] = "/home/pika/.config-king/aur/"
} 

--> Ran when build script finishes
function HookPost()
    print("Beep, build script finished!")
end