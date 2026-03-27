# Synology Download Center Telegram Bot

A Telegram bot for Synology NAS that lets you send `.torrent` files from Telegram to start downloads via Download Station. Packaged as a native Synology SPK — install it from Package Center, configure through the built-in settings UI, and you're ready to go.

Tested on **Synology DS223** (RTD1619B), but should work on any ARM64-based Synology NAS running DSM 7.0+.

## How It Works

1. You send a `.torrent` file to your Telegram bot
2. The bot saves it to a watch folder on your NAS
3. Download Station automatically picks it up and starts the download

No Synology credentials are stored or required — the bot communicates with Download Station through the filesystem using its built-in auto-download (folder watch) feature.

## Setup

### 1. Create a Telegram Bot

1. Open Telegram and message [@BotFather](https://t.me/BotFather)
2. Send `/newbot`
3. Choose a display name (e.g. "My NAS Downloads")
4. Choose a username ending in `bot` (e.g. `my_nas_dl_bot`)
5. BotFather will reply with a **bot token** — save it for the next step

### 2. Find Your Telegram User ID

Message [@userinfobot](https://t.me/userinfobot) on Telegram. It will reply with your numeric user ID. This is used to restrict the bot so only you can use it.

### 3. Configure Download Station

1. Open **Download Station** on your NAS
2. Go to **Settings** (gear icon)
3. Under **BT** tab, check **"Enable auto download"**
4. Set the **auto-download folder** to a path like `/volume1/watch`
5. Under **General** tab, set your preferred **default destination folder**
6. Click **Apply**

### 4. Install the Package

1. Download the `.spk` file from [Releases](https://github.com/dmitryduka/synology-download-center-telegram-bot/releases)
2. Open **Package Center** on your NAS
3. Click **Manual Install** and upload the `.spk` file
4. The installation wizard will ask for:
   - Your Telegram bot token
   - Your Telegram user ID(s)
   - The watch folder path (must match what you set in Download Station)
5. Start the package

### 5. Configure via Settings UI

After installation, click **Open** in Package Center (or find "Telegram Download Bot" in the DSM main menu) to access the settings page. From there you can change:

- Telegram bot token
- Authorized user IDs (comma-separated for multiple users)
- Watch folder path

## Usage

Send a `.torrent` file to your bot on Telegram. The bot will confirm it was received and placed in the watch folder. Download Station takes care of the rest.

**Commands:**
- `/help` — Show help message

## Building from Source

### Requirements

- **Rust** (stable, 1.70+) — [Install via rustup](https://rustup.rs/)
- **Zig** — needed for cross-compilation (`brew install zig` on macOS)
- **cargo-zigbuild** — `cargo install cargo-zigbuild`
- **Rust target** — `rustup target add aarch64-unknown-linux-musl`
- **Node.js** and **npm** — for building the DSM UI components

### Build

Clone the repo and run:

```bash
make spk
```

This will:
1. Cross-compile the Rust binary for `aarch64-unknown-linux-musl`
2. Build the Vue.js DSM UI with webpack
3. Assemble the `.spk` package

The output is `SynoTelegramBot-0.1.0-rtd1619b.spk`.

### Deploy to NAS

You can deploy directly if your NAS is accessible via SCP:

```bash
make install NAS_HOST=192.168.1.100 NAS_USER=admin
```

Then install the `.spk` through Package Center's Manual Install.

### Project Structure

```
src/                    Rust source code
  main.rs               Entry point
  config.rs             TOML configuration
  web.rs                Settings HTTP API (port 8008)
  synology/watcher.rs   Torrent file dropper
  telegram/bot.rs       Telegram bot setup
  telegram/handlers.rs  Message handlers
  ui.html               Embedded settings page
ui/                     DSM UI integration
  dsm/config            DSM desktop menu entry
spk/                    Synology package files
  INFO                  Package metadata
  conf/privilege        Package privilege config
  scripts/              Lifecycle scripts (start/stop/install)
  WIZARD_UIFILES/       Installation wizard
webapi/                 CGI proxy (unused — reserved for future)
```

## License

[MIT](LICENSE)
