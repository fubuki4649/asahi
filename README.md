<div align="center">

# 朝日 asahi

**asahi** is a dark mode daemon using DBus Portals

[![GPLv2](https://img.shields.io/badge/license-GPLv2-green)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html#SEC1)

</div>

### DEPENDENCIES

 - xdg-desktop-portal
 - geoclue

### INSTALLATION (Manual)

1. Copy `configs/asahi.portal` to `/usr/share/xdg-desktop-portal/portals/asahi.portal`


2. Append `asahi;` to the `default` field in `/usr/share/xdg-desktop-portal/*-portals.config`


3. Finally, start `asahi`


See the [Arch Wiki](https://wiki.archlinux.org/title/XDG_Desktop_Portal#Configuration) for more information on 
configuring the XDG Desktop Portal

### TODO
- [ ] Provide systemd daemon files for asahi
