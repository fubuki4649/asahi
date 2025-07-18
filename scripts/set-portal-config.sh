#!/bin/bash

# Make sure xdg-desktop-portal exists is running
# If xdg-desktop-portal isnt running, the next section will spawn a daemon and not return
if ! systemctl --user status xdg-desktop-portal | grep running > /dev/null; then
    echo "Error: xdg-desktop-portal user service is not running. Please start that first." >&2
    exit 1
fi

# Gets path of the currently used *-portals.conf
CONFPATH=$(/usr/lib/xdg-desktop-portal --verbose 2>&1 | grep "Using portal configuration file" | grep -o "'/[^']*'" | tr -d "'")
echo "Portal config found at $CONFPATH"

# Checks if the conf file is already set to use `asahi`. If not, modify the conf to do so
LINE='org.freedesktop.impl.portal.Settings=asahi'

# Does the file already contain the line?
if ! grep -Fxq -- "$LINE" "$CONFPATH"; then
    # Append the line using sudo
    echo "$LINE" | sudo tee -a "$CONFPATH" > /dev/null
    echo "Added $LINE to $CONFPATH"
else
    echo "$CONFPATH is already ok. No changes made"
fi