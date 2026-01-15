# Launch

A sleek, pill-shaped application launcher for Linux, built with Rust and GTK4. Designed with strict adherence to "Tree Architecture" for stability and maintainability.


![AppSearch](docs/images/screenshots/app_search.png)

## Features

-   **Sleek Design**: Transparent, pill-shaped UI (`border-radius: 30px`).
-   **Fast Search**: Fuzzy search through your installed applications (`.desktop` files).
-   **Visual Feedback**: Displays application icons.
-   **Keyboard Navigation**:
    -   `Up` / `Down`: Navigate results.
    -   `Enter`: Launch selected application (or Enter directory).
    -   `Right Arrow`: Enter highlighted directory (File Browser).
    -   `Left Arrow`: Go up one directory level (File Browser).
    -   `Ctrl + 1-9`: Quick launch the Nth result.
    -   `Escape`: Close the launcher.
-   **Status Indicators**: Highlights running applications (bold text).

## Usage

### 🚀 Prefixes

Launch uses strict prefixes to route your queries to different providers:

| Prefix | Name | Description | Example |
| :--- | :--- | :--- | :--- |
| `x ` | **Execute** | Run a shell command in a terminal wrapper. | `x top` (runs `gnome-terminal -- top`) |
| `f ` | **Files** | Browse the filesystem. Use `Left`/`Right` keys to navigate. | `f /home/user/` |
| `ss ` | **Shortcuts** | Run a saved shortcut. | `ss term` |
| `m ` | **Macros** | Execute a sequence of commands (Macro). | `m dev-setup` |
| `c ` | **Calc** | Solve math expressions (supports basic LaTeX!). | `c \sqrt{16} * 2` |
| `! ` | **System** | Power operations (suspend, reboot, poweroff). | `! reboot` |
| `l ` | **Launch** | Internal commands (Settings, About, Quit). | `l settings` |

### ⚙️ Configuration

Launch is fully configurable via an interactive UI.

1.  Type `l settings` (or `l set`) and hit **Enter**.
2.  Use the **Tabs** to switch between **Shortcuts** and **Macros**.
3.  **Add**: Click "Add" to define a new item.
4.  **Delete**: Select an item and click "Delete" to remove it.

Changes are persisted to `~/.config/launch/settings.json`.

#### Example `settings.json`
```json
{
  "shortcuts": {
    "term": "gnome-terminal",
    "web": "firefox"
  },
  "macros": [
    {
      "name": "morning",
      "actions": [
        "notify-send 'Good Morning'",
        "firefox https://news.ycombinator.com"
      ]
    }
  ]
}
```

### 👻 Daemon Mode
Launch runs as a background daemon.
-   Run `launch toggle` to show/hide the window.
-   Bind `launch toggle` to a global shortcut in your Desktop Environment (e.g., `Super+Space`).

## Tech Stack

-   **Language**: [Rust](https://www.rust-lang.org/) (2024 Edition)
-   **GUI**: [GTK4](https://gtk.org/)
-   **Architecture**: Tree Architecture (Domain -> Application -> Interface -> Infrastructure)

## Installation

### Prerequisites

-   Rust & Cargo
-   GTK4 development libraries (e.g., `libgtk-4-dev` on Debian/Ubuntu, `gtk4-devel` on Fedora)

### Building from Source

```bash
git clone https://github.com/leechristophermurray/launch.git
cd launch
cargo run --release
```

### Building Packages (.deb / .rpm)

The project includes a helper script to generate Linux packages.

```bash
./build_packages.sh
```

Artifacts will be output to:
-   `target/debian/*.deb`
-   `target/generate-rpm/*.rpm`

## Architecture

This project follows the **Tree Architecture** pattern:

-   🔴 **Domain**: Pure business entities (`App`, `Shortcut`).
-   🟡 **Application**: Use cases (`SearchApps`, `ExecuteCommand`).
-   🟢 **Interface**: abstract Ports (`IAppRepository`).
-   🔵 **Infrastructure**: Concrete Adapters (`LinuxAppRepoAdapter`, `AppWindow`).

## License

MIT License. See [LICENSE](LICENSE) for details.
