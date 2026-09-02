<div align="center">

# 朝日 asahi

**asahi** is a dark mode daemon using DBus Portals

[![GPLv2](https://img.shields.io/badge/license-GPLv2-green)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html#SEC1)

</div>

### DEPENDENCIES

- xdg-desktop-portal
- busctl (systemd)

### INSTALLATION (Automatic)

Coming Soon!

### INSTALLATION (Manual)

Build the release binary, then install files to the following locations (all `/usr` destinations require root):

| Source                                                      | Destination                                                                    |
|-------------------------------------------------------------|--------------------------------------------------------------------------------|
| `target/release/asahi`                                      | `/usr/lib/xdg-desktop-portal-asahi`                                            |
| `scripts/asahictl`                                          | `/usr/bin/asahictl`                                                            |
| `configs/asahi.portal`                                      | `/usr/share/xdg-desktop-portal/portals/asahi.portal`                           |
| `configs/org.freedesktop.impl.portal.desktop.asahi.service` | `/usr/share/dbus-1/services/org.freedesktop.impl.portal.desktop.asahi.service` |
| `configs/xdg-desktop-portal-asahi.service`                  | `/usr/lib/systemd/user/xdg-desktop-portal-asahi.service`                       |

> **Note:** The D-Bus `.service` file and the systemd unit both hardcode `/usr/lib/xdg-desktop-portal-asahi`. If your distribution uses a different prefix (e.g. `/usr/libexec/`), update both files accordingly before installing.

Next, assign asahi as the `Settings` portal backend. The recommended location is the **user-level** `portals.conf`, which takes precedence over system-wide defaults and requires no root:

```ini
# ~/.config/xdg-desktop-portal/portals.conf
[preferred]
org.freedesktop.impl.portal.Settings=asahi
```

The key **must** be under `[preferred]`; bare keys outside a section are silently ignored. See the [upstream docs](https://flatpak.github.io/xdg-desktop-portal/docs/portals.conf.html) for the full format and lookup order.

Finally, reload the user service manager and restart the portal:

```
systemctl --user daemon-reload
systemctl --user restart xdg-desktop-portal
```

> **Note:** `daemon-reload` is required after installing a new unit; without it systemd will reject the D-Bus activation request. The unit also gates on `ConditionEnvironment=WAYLAND_DISPLAY` — if activation silently fails, check that your compositor has exported that variable into the systemd user environment (`systemctl --user show-environment`).

Alternatively, just run this

```shell
### 1. Build
cargo build --release

### 2. Install Files (System-wide)
sudo install -Dm755 target/release/asahi /usr/lib/xdg-desktop-portal-asahi
sudo install -Dm755 scripts/asahictl /usr/bin/asahictl
sudo install -Dm644 configs/asahi.portal /usr/share/xdg-desktop-portal/portals/asahi.portal
sudo install -Dm644 configs/org.freedesktop.impl.portal.desktop.asahi.service /usr/share/dbus-1/services/org.freedesktop.impl.portal.desktop.asahi.service
sudo install -Dm644 configs/xdg-desktop-portal-asahi.service /usr/lib/systemd/user/xdg-desktop-portal-asahi.service

### 3. Configure Portal (User-level, no root needed)
mkdir -p ~/.config/xdg-desktop-portal
cat << 'EOF' > ~/.config/xdg-desktop-portal/portals.conf
[preferred]
default=gtk
org.freedesktop.impl.portal.Settings=asahi
EOF

### 4. Reload and Restart (User session)
systemctl --user daemon-reload
systemctl --user restart xdg-desktop-portal
```

### CONFIGURATION

`asahi` reads configuration from two locations, merged at startup; the user-local file takes priority over the system-wide one:

| Scope       | Path                             |
|-------------|----------------------------------|
| System-wide | `/etc/asahi/config.toml`         |
| User-local  | `~/.config/asahi/config.toml`    |

Both files are optional. A sample config with all available keys and their defaults is provided at [`configs/config.toml`](configs/config.toml).

| Key                      | Default | Description                                                                     |
|--------------------------|---------|---------------------------------------------------------------------------------|
| `log_level`              | `info`  | Log verbosity (`error` `warn` `info` `debug` `trace`)                           |
| `override_lat`           | (off)   | (Optional) Override auto geolocation (must be set together with `override_lon`) |
| `override_lon`           | (off)   | (Optional) Override auto geolocation (must be set together with `override_lat`) |
| `location_ttl`           | `3600`  | Seconds before a location fix is refreshed                                      |
| `sunset_check_frequency` | `600`   | Seconds between sunrise/sunset checks                                           |
| `sunrise_offset`         | `0`     | Minutes to shift the light-mode trigger (± relative to sunrise)                 |
| `sunset_offset`          | `0`     | Minutes to shift the dark-mode trigger (± relative to sunset)                   |

### CLI UTILITY

[`scripts/asahictl`](scripts/asahictl) queries daemon state and sets manual overrides. Run `asahictl --help` for usage.

### D-BUS MANAGEMENT INTERFACE

In addition to `org.freedesktop.impl.portal.Settings`, `asahi` exposes a custom management interface:

* **Destination:** `org.freedesktop.impl.portal.desktop.asahi`
* **Object Path:** `/org/freedesktop/portal/desktop`
* **Interface:** `org.freedesktop.impl.portal.asahi.Control`

#### Methods
* `setManualDarkMode(int32)`: Sets or clears a manual theme override (`-1` = Automatic, `0` = No Preference, `1` = Dark, `2` = Light).

#### Properties (Read-Only)
* `currentTheme` (`u32`): Theme currently being broadcast (`0` = No Preference, `1` = Dark, `2` = Light).
* `isOverrideSet` (`bool`): Whether a manual override is active.
* `nextTransitionAt` (`string`): Expected time of next sunrise/sunset transition as an RFC 3339 UTC timestamp.
* `location` (`(double, double)`): Coordinates currently used for solar calculations.

### FIREFOX

Firefox ignores XDG Desktop Portal theme signals, listening only to GNOME's GSettings by default instead.
To fix this, set `widget.use-xdg-desktop-portal.settings` to `2` in `about:config`.
