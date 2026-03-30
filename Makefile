PREFIX  ?= $(HOME)/.local
BINDIR  ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
UNAME   := $(shell uname -s)
REPO    := syn7xx/scoria

.PHONY: all build deps install uninstall update check fmt clean changelog

all: build

build:
	cargo build --release

# Platform-specific dependency installation
ifeq ($(UNAME),Linux)
deps:
	@if command -v pacman >/dev/null 2>&1; then \
		sudo pacman -S --needed base-devel rust gtk3 libappindicator-gtk3 xdotool wl-clipboard; \
	elif command -v dnf >/dev/null 2>&1; then \
		sudo dnf install -y gcc make rust cargo gtk3-devel libappindicator-gtk3-devel xdotool wl-clipboard; \
	elif command -v apt-get >/dev/null 2>&1; then \
		sudo apt-get install -y build-essential rustc cargo libgtk-3-dev libappindicator3-dev libxdo-dev xdotool wl-clipboard; \
	else \
		echo "Unknown package manager. Install manually: rust, gtk3-devel, libappindicator-gtk3-devel, xdotool, wl-clipboard"; \
	fi
else ifeq ($(UNAME),Darwin)
deps:
	brew install rust
else
deps:
	@echo "Unsupported OS: $(UNAME). Install Rust manually: https://rustup.rs/"
endif

# Platform-specific install
ifeq ($(UNAME),Darwin)
MACOS_APP := $(HOME)/Applications/Scoria.app
install: build
	install -d "$(MACOS_APP)/Contents/MacOS"
	install -d "$(MACOS_APP)/Contents/Resources"
	install -m 644 assets/macos/Info.plist "$(MACOS_APP)/Contents/Info.plist"
	install -m 644 assets/macos/Resources/scoria.icns "$(MACOS_APP)/Contents/Resources/scoria.icns"
	install -m 755 target/release/scoria "$(MACOS_APP)/Contents/MacOS/scoria"
	install -d "$(BINDIR)"
	ln -sf "$(MACOS_APP)/Contents/MacOS/scoria" "$(BINDIR)/scoria"
	@echo "Installed $(MACOS_APP) and linked $(BINDIR)/scoria"
	@echo "Add $(BINDIR) to PATH if not already present."
else
install: build
	install -d "$(BINDIR)"
	install -m 755 target/release/scoria "$(BINDIR)/scoria"
	install -d "$(DATADIR)/icons/hicolor/scalable/apps"
	install -m 644 assets/scoria.svg "$(DATADIR)/icons/hicolor/scalable/apps/scoria.svg"
	install -d "$(DATADIR)/icons/hicolor/128x128/apps"
	install -m 644 assets/scoria-128.png "$(DATADIR)/icons/hicolor/128x128/apps/scoria.png"
	install -d "$(DATADIR)/applications"
	install -m 644 assets/scoria.desktop "$(DATADIR)/applications/scoria.desktop"
endif

ifeq ($(UNAME),Darwin)
uninstall:
	rm -f "$(BINDIR)/scoria"
	rm -rf "$(HOME)/Applications/Scoria.app"
else
uninstall:
	rm -f "$(BINDIR)/scoria"
	rm -f "$(DATADIR)/icons/hicolor/scalable/apps/scoria.svg"
	rm -f "$(DATADIR)/icons/hicolor/128x128/apps/scoria.png"
	rm -f "$(DATADIR)/applications/scoria.desktop"
endif

update:
	@LATEST=$$(curl -sL "https://api.github.com/repos/$(REPO)/releases/latest" \
		| grep '"tag_name"' | head -1 | cut -d'"' -f4); \
	if [ -z "$$LATEST" ]; then echo "Could not fetch latest version."; exit 1; fi; \
	echo "Latest release: $$LATEST"; \
	ARCH=$$(uname -m); \
	case "$$ARCH" in x86_64) ARCH=x86_64;; arm64|aarch64) ARCH=aarch64;; *) echo "Unsupported arch: $$ARCH"; exit 1;; esac; \
	case "$(UNAME)" in \
		Linux)  ASSET="scoria-linux-$$ARCH.tar.gz";; \
		Darwin) ASSET="scoria-macos-$$ARCH.tar.gz";; \
		*)      echo "Unsupported OS: $(UNAME)"; exit 1;; \
	esac; \
	URL="https://github.com/$(REPO)/releases/download/$$LATEST/$$ASSET"; \
	echo "Downloading $$URL ..."; \
	curl -sL "$$URL" | tar xz -C /tmp; \
	if [ "$(UNAME)" = "Darwin" ] && [ -d "$(HOME)/Applications/Scoria.app/Contents" ]; then \
		APP="$(HOME)/Applications/Scoria.app"; \
		install -d "$$APP/Contents/MacOS" "$$APP/Contents/Resources"; \
		install -m 755 /tmp/scoria "$$APP/Contents/MacOS/scoria"; \
		curl -sLf "https://raw.githubusercontent.com/$(REPO)/$$LATEST/assets/macos/Resources/scoria.icns" \
			-o "$$APP/Contents/Resources/scoria.icns" \
			|| curl -sLf "https://raw.githubusercontent.com/$(REPO)/main/assets/macos/Resources/scoria.icns" \
			-o "$$APP/Contents/Resources/scoria.icns" \
			|| echo "Note: could not refresh scoria.icns; icon may stay old until next make install."; \
		curl -sLf "https://raw.githubusercontent.com/$(REPO)/$$LATEST/assets/macos/Info.plist" \
			-o "$$APP/Contents/Info.plist" \
			|| curl -sLf "https://raw.githubusercontent.com/$(REPO)/main/assets/macos/Info.plist" \
			-o "$$APP/Contents/Info.plist" \
			|| echo "Note: could not refresh Info.plist; run make install from a clone to update metadata."; \
		install -d "$(BINDIR)"; \
		ln -sf "$$APP/Contents/MacOS/scoria" "$(BINDIR)/scoria"; \
		echo "Updated $$LATEST: $$APP and $(BINDIR)/scoria"; \
	else \
		install -d "$(BINDIR)"; \
		install -m 755 /tmp/scoria "$(BINDIR)/scoria"; \
		echo "Updated to $$LATEST ($(BINDIR)/scoria)"; \
	fi; \
	rm -f /tmp/scoria

check:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt

clean:
	cargo clean
