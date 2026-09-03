<div align="center">

# 朝日 asahi

**asahi** is a dark mode daemon using DBus Portals

[![GPLv2](https://img.shields.io/badge/license-GPLv2-green)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html#SEC1)

</div>

### DEPENDENCIES

- xdg-desktop-portal
- busctl (systemd)

### INSTALLATION

```
make install
```

`make install` builds the binary, installs all system files, configures `portals.conf`, and restarts `xdg-desktop-portal` in one step. `PREFIX` defaults to `/usr`; override for non-standard layouts (e.g. `make install PREFIX=/usr/local`). `DESTDIR` is supported for staged package builds (portals.conf and systemctl steps are skipped automatically).

### INSTALLATION (Manual)

Build the release binary, then install files to the following locations (all `/usr` destinations require root):

| Source                                                      | Destination                                                                    |
|-------------------------------------------------------------|--------------------------------------------------------------------------------|
| `target/release/asahi`                                      | `/usr/lib/xdg-desktop-portal-asahi`                                            |
| `scripts/asahictl`                                          | `/usr/bin/asahictl`                                                            |
| `configs/asahi.portal`                                      | `/usr/share/xdg-desktop-portal/portals/asahi.portal`                           |
| `configs/org.freedesktop.impl.portal.desktop.asahi.service` | `/usr/share/dbus-1/services/org.freedesktop.impl.portal.desktop.asahi.service` |
| `configs/xdg-desktop-portal-asahi.service`                  | `/usr/lib/systemd/user/xdg-desktop-portal-asahi.service`                       |

> **Note:** The D-Bus `.service` file and the systemd unit both hardcode the binary path. If your distribution uses a prefix other than `/usr` (e.g. `/usr/libexec/`), update the `Exec=` / `ExecStart=` lines in both config files before installing.

Add the following to `~/.config/xdg-desktop-portal/portals.conf` under `[preferred]` (bare keys not belonging to a section are silently ignored):

```ini
[preferred]
org.freedesktop.impl.portal.Settings=asahi
```

Then:

```
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
* `nextTransitionAt` (`string`): Expected time of next sunrise/sunset transition as an RFC 3339 timestamp in the local timezone.
* `location` (`(double, double)`): Coordinates currently used for solar calculations.

### FIREFOX

Firefox ignores XDG Desktop Portal theme signals, listening only to GNOME's GSettings by default instead.
To fix this, set `widget.use-xdg-desktop-portal.settings` to `2` in `about:config`.
