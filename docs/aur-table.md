# AUR Table

### Example Definitions:
```lua
Packages = {
    Aur = {

    }
}
```

```lua
Packages = {
    Aur = {
        --> Regular Package
        "package-name-here",

        --> Advanced Package
        {
            ["base"] = "nvidia-utils-beta",
            ["sub"] = {"nvidia-utils-beta", "opencl-nvidia-beta", "nvidia-settings-beta"},
        },

        --> Advanced Package with Url
        {
            ["base"] = "Rust-VPN-Handler", 
            ["sub"] = {"vpn_handler"}, 
            ["url"] = "https://github.com/kingdomkind/Rust-VPN-Handler.git",
        },
    }
    }
```

### Element Types:

**Regular Package:**

```lua
"package-name-here",
```
This will pull the package from aur, assuming the entered text is the only package that the pkgbuild will attempt to install, and that the pkgbase is the same as the entered text. For example, in this it will query:
> https://aur.archlinux.org/packages/package-name-here

and will attempt to git pull the repository from here.

**Advanced Package:**

Example without url:
```lua
{
    ["base"] = "nvidia-utils-beta",
    ["sub"] = {"nvidia-utils-beta", "opencl-nvidia-beta", "nvidia-settings-beta"},
},
```

Example with url:
```lua
{
    ["base"] = "Rust-VPN-Handler", 
    ["sub"] = {"vpn_handler"}, 
    ["url"] = "https://github.com/kingdomkind/Rust-VPN-Handler.git",
},
```

The ["base"] defines the url to pull from, ie. in the former example the url to pull from is:
> https://aur.archlinux.org/packages/nvidia-utils-beta

It also dictates the folder name within the aur folder of .config-king. In the case where an optional ["url"] is provided, this is the only functionality of it.

The ["sub"] key represents the sub-packages installed by the PKGBUILD. This can be seen in the pkgname section of the PKGBUILD. Often, this will be an array, and hence is defined as a table here. Please note that the value of ["base"] is not copied into the value of ["sub"] automatically and so you will need to redefine it in there if you wish for that sub package to be installed, as seen in the former example.

The ["url"] key is an optional key that allows you to download PKGBUILDs and their relevant files from repos that are not the AUR. It takes the git link to the repo of the package you wish to download, as seen in the latter example. 

*Please note that changing the url of an AUR package that is already cloned will not do anything - the url is only used the orginal git clone. If you want to change the url only, then comment out the relevant code, run config-king, then uncomment the code and run it again*


