TARGET = aarch64-unknown-linux-musl
NAS_HOST ?= 192.168.1.100
NAS_USER ?= admin
BINARY = target/$(TARGET)/release/syno-bot
UI_DIST = ui/dist
SPK_NAME = SynoTelegramBot-0.1.0-rtd1619b.spk
SPK_DIR = spk
PACKAGE_DIR = $(SPK_DIR)/package

.PHONY: all binary ui spk clean install

all: spk

# Cross-compile the Rust binary for Synology DS223 (aarch64)
binary:
	cargo zigbuild --release --target $(TARGET)

# Build the Vue.js DSM native UI
ui:
	cd ui && npm install && npx webpack --mode production

# Assemble the SPK package
spk: binary ui
	@echo "==> Assembling SPK package..."
	@mkdir -p $(PACKAGE_DIR)/bin $(PACKAGE_DIR)/ui/images
	@# Copy binary
	@cp $(BINARY) $(PACKAGE_DIR)/bin/syno-bot
	@# Copy UI files (DSM desktop integration)
	@cp ui/config $(PACKAGE_DIR)/ui/
	@cp ui/redirect.html $(PACKAGE_DIR)/ui/
	@# TODO: Add proper icon files. For now create placeholders.
	@if [ ! -f $(PACKAGE_DIR)/ui/images/icon_64.png ]; then \
		printf '\x89PNG\r\n\x1a\n' > $(PACKAGE_DIR)/ui/images/icon_64.png; \
	fi
	@# Create package.tgz
	@cd $(PACKAGE_DIR) && tar czf ../package.tgz *
	@# Make scripts executable
	@chmod +x $(SPK_DIR)/scripts/*
	@# Create the .spk file
	@cd $(SPK_DIR) && tar cf ../$(SPK_NAME) INFO package.tgz scripts conf WIZARD_UIFILES
	@rm -f $(SPK_DIR)/package.tgz
	@echo "==> Built $(SPK_NAME) ($$(du -h $(SPK_NAME) | cut -f1))"

# Deploy to NAS (requires sshpass and NAS accessibility)
install: spk
	@echo "==> Uploading SPK to NAS..."
	scp $(SPK_NAME) $(NAS_USER)@$(NAS_HOST):/tmp/
	@echo "==> Install via DSM Package Center: Manual Install -> /tmp/$(SPK_NAME)"

clean:
	cargo clean
	rm -rf $(PACKAGE_DIR) $(SPK_DIR)/package.tgz $(SPK_NAME)
	rm -rf ui/dist ui/node_modules
