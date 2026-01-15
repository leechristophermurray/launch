# **"Launch" Launcher \- Technical Specifications**

## **1\. Visual Requirements (The "Pill")**

* **Window:** gtk4::ApplicationWindow with decorated(false).  
* **CSS Styling:** Custom CSS provider to apply border-radius: 30px and a semi-transparent background with a subtle blur (if supported by the compositor).  
* **Positioning:** Centered at the top-third of the screen.

## **2\. Interaction Model**

* **Prefix Handling:** \- No prefix: Default to App Search.  
  * `ss `: Shortcut execution on Enter. Shortcuts are configurable through launch's settings
  * `l `: Switch context to Internal Settings. This gives the option to open the shortcut configurator.
  * `x `: to immediately execute a shell command in a terminal.
    * Example: `x top` opens `btop`/`top` in a new terminal window.
    * Supported terminals: `gnome-terminal`, `ptyxis`, `x-terminal-emulator`.
  * `f `: (lower case F followed by space) to browse the filesystem.
    * Start typing a path (e.g., `f /home`) to filter.
    * Use `Right Arrow` to enter directories.
    * Use `Left Arrow` to go up.
    * `Enter` on a file opens it with `xdg-open`.
    * `Enter` on a directory opens it in `nautilus`.
  * `m `: Macro execution on Enter. Macros are configurable through launch's settings.
  * `! `: system actions on Enter. System Action are:
    * Suspend the system
    * Reboot the system
    * Hibernate the system
    * Power off the system
    * Lock the system
  * `c `: calculate as the user types, with support for latex math notation.
  * `shift + super + touchpadoff`: as the default keyboard shortcut that brings up the search bar. The is reconfigurable in the settings.
* **Navigation:**  
  * Up/Down arrows to navigate the list.  
  * Enter to execute the top or selected item.  
  * Esc to close the launcher.

## **3\. Architecture: The "Tree" Approach**

* **The Root:** State struct managed by `Arc<Mutex<State>>`.  
* **The Branches (Logic):**  
  * SearchEngine: Coordinates which provider to query based on input string.  
  * AppCache: A background thread or periodic task to list .desktop files and check running status via procfs.  
* **The Leaves (Execution):**  
  * CommandExecutor: Handles std::process::Command calls for shortcuts and apps.  
  * SettingsHandler: Opens Adwaita Dialogs for internal configurations.

## **4\. Key Features**

* **Daemon Mode:** Launch runs as a background daemon by default.
  * Run `launch toggle` to show/hide the window.
  * Bind `launch toggle` to a global shortcut in your Desktop Environment settings (e.g., `Super+Space`).
  * The window automatically hides when it loses focus or when an app is launched.
* **Running App Priority:** Apps found in the process list are moved to the top of the search results and styled with bold text and a faint border.  
* **Configurable Shortcuts:** Shortcuts are configurable through launch's settings as a "Shortscuts"Options which opens a dialog to add/edit/delete the shortcuts or reset to defaults. Shortcuts are a combination of keycodes and modifiers.
* **Shortcut Registry:** A simple HashMap lookup
* **Settings Filtering:** Real-time filtering of internal "Launch" commands.
* **Macro Registry:** A simple HashMap lookup
* **Macro Execution:** Macros are executed in the order they are defined. Macros are configurable through launch's settings as a "Macros". Options which opens a dialog to add/edit/delete the macros or reset to defaults. These are a series actions (shortcuts, commands, folders, apps) that are executed in order.
* 