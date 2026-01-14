# Launch

A sleek, pill-shaped application launcher for Linux, built with Rust and GTK4. Designed with strict adherence to "Tree Architecture" for stability and maintainability.

![Concept](https://via.placeholder.com/600x100?text=Launch+Pill+UI+Concept)

## Features

-   **Sleek Design**: Transparent, pill-shaped UI (`border-radius: 30px`).
-   **Fast Search**: Fuzzy search through your installed applications (`.desktop` files).
-   **Visual Feedback**: Displays application icons.
-   **Keyboard Navigation**:
    -   `Up` / `Down`: Navigate results.
    -   `Enter`: Launch selected application.
    -   `Ctrl + 1-9`: Quick launch the Nth result.
    -   `Escape`: Close the launcher.
-   **Status Indicators**: Highlights running applications (bold text).

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
