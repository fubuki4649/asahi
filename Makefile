# ------------------------------------------------------------------------------
# asahi — Makefile
#
# Standard targets: build, install, install-config, uninstall, uninstall-config, clean
#
# Variables (all overridable on the command line):
#   PREFIX   — installation prefix              (default: /usr)
#   DESTDIR  — staging root for package builds  (default: empty)
#
# Run as your normal user. Commands that write to system paths use sudo internally.
# When DESTDIR is set, portals.conf and systemctl steps are skipped (staging mode).
#
# Examples:
#   make install                          # build, install, configure, and activate
#   make install PREFIX=/usr/local        # install to /usr/local
#   make install DESTDIR=/tmp/pkg         # stage under /tmp/pkg/usr for packaging
#   make uninstall                        # remove all installed files and deactivate
#   make install-config                   # write user-level portals.conf only
#   make uninstall-config                 # remove portals.conf entry only
# ------------------------------------------------------------------------------

PREFIX  ?= /usr
DESTDIR ?=

BINARY     := target/release/asahi
BINARY_DST := $(DESTDIR)$(PREFIX)/lib/xdg-desktop-portal-asahi

PORTAL_SRC     := configs/asahi.portal
PORTAL_DST     := $(DESTDIR)$(PREFIX)/share/xdg-desktop-portal/portals/asahi.portal

DBUS_SVC_SRC   := configs/org.freedesktop.impl.portal.desktop.asahi.service
DBUS_SVC_DST   := $(DESTDIR)$(PREFIX)/share/dbus-1/services/org.freedesktop.impl.portal.desktop.asahi.service

SYSTEMD_SVC_SRC := configs/xdg-desktop-portal-asahi.service
SYSTEMD_SVC_DST := $(DESTDIR)$(PREFIX)/lib/systemd/user/xdg-desktop-portal-asahi.service

ASAHICTL_SRC   := scripts/asahictl
ASAHICTL_DST   := $(DESTDIR)$(PREFIX)/bin/asahictl

PORTALS_CONF   := $(HOME)/.config/xdg-desktop-portal/portals.conf
PORTAL_SETTING := org.freedesktop.impl.portal.Settings=asahi

.PHONY: all build install install-config uninstall uninstall-config clean help

all: build

# ------------------------------------------------------------------------------

build:
	cargo build --release

# ------------------------------------------------------------------------------

install: build
	sudo install -Dm755 $(BINARY)          $(BINARY_DST)
	sudo install -Dm755 $(ASAHICTL_SRC)    $(ASAHICTL_DST)
	sudo install -Dm644 $(PORTAL_SRC)      $(PORTAL_DST)
	sudo install -Dm644 $(DBUS_SVC_SRC)    $(DBUS_SVC_DST)
	sudo sed -i 's|Exec=.*|Exec=$(PREFIX)/lib/xdg-desktop-portal-asahi|' $(DBUS_SVC_DST)
	sudo install -Dm644 $(SYSTEMD_SVC_SRC) $(SYSTEMD_SVC_DST)
	sudo sed -i 's|ExecStart=.*|ExecStart=$(PREFIX)/lib/xdg-desktop-portal-asahi|' $(SYSTEMD_SVC_DST)
	@if [ -z '$(DESTDIR)' ]; then \
		$(MAKE) --no-print-directory install-config; \
		systemctl --user daemon-reload; \
		systemctl --user restart xdg-desktop-portal; \
		systemctl --user try-restart xdg-desktop-portal-asahi; \
	fi

# ------------------------------------------------------------------------------
# install-config: user-level portals.conf (no root required, no DESTDIR support)
#
# Adds "org.freedesktop.impl.portal.Settings=asahi" under [preferred], creating
# the file if absent. If [preferred] already exists, the key is appended to that
# section. If the key is already present, this is a no-op.
# ------------------------------------------------------------------------------

install-config:
	@if grep -qF '$(PORTAL_SETTING)' '$(PORTALS_CONF)' 2>/dev/null; then \
		echo "$(PORTALS_CONF): already configured, nothing to do"; \
	elif grep -qF '[preferred]' '$(PORTALS_CONF)' 2>/dev/null; then \
		sed -i '/^\[preferred\]/a $(PORTAL_SETTING)' '$(PORTALS_CONF)'; \
		echo "$(PORTALS_CONF): appended $(PORTAL_SETTING) under [preferred]"; \
	else \
		mkdir -p "$$(dirname '$(PORTALS_CONF)')"; \
		printf '[preferred]\n$(PORTAL_SETTING)\n' >> '$(PORTALS_CONF)'; \
		echo "$(PORTALS_CONF): created with [preferred] section"; \
	fi

# ------------------------------------------------------------------------------

uninstall:
	@if [ -z '$(DESTDIR)' ]; then \
		systemctl --user stop xdg-desktop-portal-asahi 2>/dev/null || true; \
	fi
	sudo rm -f $(BINARY_DST)
	sudo rm -f $(ASAHICTL_DST)
	sudo rm -f $(PORTAL_DST)
	sudo rm -f $(DBUS_SVC_DST)
	sudo rm -f $(SYSTEMD_SVC_DST)
	@if [ -z '$(DESTDIR)' ]; then \
		$(MAKE) --no-print-directory uninstall-config; \
		systemctl --user daemon-reload; \
		systemctl --user restart xdg-desktop-portal; \
	fi

# ------------------------------------------------------------------------------
# uninstall-config: remove the portals.conf entry added by install-config.
# No root required. No-op if the entry is not present.
# ------------------------------------------------------------------------------

uninstall-config:
	@if [ ! -f '$(PORTALS_CONF)' ]; then \
		echo "$(PORTALS_CONF): not found, nothing to do"; \
	elif ! grep -qF '$(PORTAL_SETTING)' '$(PORTALS_CONF)'; then \
		echo "$(PORTALS_CONF): entry not present, nothing to do"; \
	else \
		sed -i '/^$(PORTAL_SETTING)$$/d' '$(PORTALS_CONF)'; \
		echo "$(PORTALS_CONF): removed $(PORTAL_SETTING)"; \
	fi

# ------------------------------------------------------------------------------

clean:
	cargo clean

# ------------------------------------------------------------------------------

help:
	@echo "Targets:"
	@echo "  build             Build the release binary (default)"
	@echo "  install           Build, install, configure, and activate"
	@echo "  install-config    Write user-level portals.conf only"
	@echo "  uninstall         Remove all installed files and deactivate"
	@echo "  uninstall-config  Remove portals.conf entry only"
	@echo "  clean             Remove build artifacts"
	@echo ""
	@echo "Variables:"
	@echo "  PREFIX=$(PREFIX)"
	@echo "  DESTDIR=$(DESTDIR)"
