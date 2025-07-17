<div align="center">

# 朝日 asahi

**asahi** is a dark mode daemon using DBus Portals

[![GPLv2](https://img.shields.io/badge/license-GPLv2-green)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html#SEC1)

</div>

### DEPENDENCIES

- xdg-desktop-portal
- geoclue

### INSTALLATION (Manual)

1. Copy the binary to `/usr/lib/xdg-desktop-portal-asahi` (or wherever else portals are stored on the system)

2. Copy config files
    - `configs/asahi.portal` to `/usr/share/xdg-desktop-portal/portals/asahi.portal`
    - `configs/org.freedesktop.impl.portal.desktop.asahi.service` to `/usr/share/dbus-1/services/org.freedesktop.impl.portal.desktop.asahi.service`
    - `configs/xdg-desktop-portal-asahi.service` to `/usr/lib/systemd/user/xdg-desktop-portal-asahi.service`


3. Append the following line to the end of the active `*-portals.conf` file


    org.freedesktop.impl.portal.Settings=asahi


For help identifying the active config file, read the XDG Desktop Portal docs [here](https://flatpak.github.io/xdg-desktop-portal/docs/portals.conf.html#description)

4. Finally, restart `xdg-desktop-portal`


    systemctl --user restart xdg-desktop-portal


See the [Arch Wiki](https://wiki.archlinux.org/title/XDG_Desktop_Portal#Configuration) for more information on
configuring the XDG Desktop Portal

### TODO
- [ ] Provide a makefile
- [ ] Find a firefox workaround
- [ ] A CLI to pair the daemon with

