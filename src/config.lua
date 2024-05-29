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
