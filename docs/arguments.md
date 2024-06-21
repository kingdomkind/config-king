# Arguments

### Options

- DIRECTORY
    - Type: String
    - Default: current directory
    - Options: path to config.lua file
    - Example: DIRECTORY=/home/pika/config.lua
    - Description: Allows you to provide the path of where the config.lua file is to the program.

- PACKGE_REMOVE_WARN_LIMIT
    - Type: Unsigned Integer32
    - Default: 5
    - Options: Integer > 0
    - Example: PACKGE_REMOVE_WARN_LIMIT=0
    - Description: The number of packages that can be removed before the program warns you about the packages being removed, to help prevent any "accidental nukes" of the system. Setting it to 0 means that it always warns you of which packages are being removed. Set to an arbitrarily high number (that u32 can support) to not be warned.

- ROOT_CHECK
    - Type: Bool
    - Default: true
    - Options: true, false
    - Example: ROOT_CHECK=false
    - Description: Checks the program is running as root, so it can execute the necessary commands. Set to false if you have another method of ensuring it can run the necessary commands.

- SEE_STDERR
    - Type: Bool
    - Default: true
    - Options: true, false
    - Example: SEE_STDERR=true
    - Description: If set to true, prints out the stderr (warnings / errors) logs from any commands.

- SEE_STDOUT
    - Type: Bool
    - Default: true
    - Options: true, false
    - Example: SEE_STDOUT=true
    - Description: If set to true, prints out the stdout (any regular output) from any commands

- ASSUME_YES
    - Type: Bool
    - Default: true
    - Options: true, false
    - Example: ASSUME_YES=true
    - Description: If set to true, assumes the default answer to any program running. The name is somewhat a bit misleading, as it can sometimes be no (in case of package conflicts)

- DEFAULT_YES
    - Type: Bool
    - Default: true
    - Options: true, false
    - Example: DEFAULT_YES=true
    - Description: Changes the default behaviour of y/n questions config-king asks you (eg. removing a directory). Setting to true means that pressing enter assumes you pressed y, and setting false assumes that pressing enter means n.

*Please note that unrecognised arguments are simply skipped, and are not announced to the user.* 

[Back to Index](https://github.com/kingdomkind/config-king/blob/main/docs/index.md)

