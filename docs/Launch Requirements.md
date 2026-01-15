# **"Launch" Launcher \- Technical Specifications**

## **1\. Visual Requirements (The "Pill")**

* **Window:** gtk4::ApplicationWindow with decorated(false).  
* **CSS Styling:** Custom CSS provider to apply border-radius: 30px and a semi-transparent background with a subtle blur (if supported by the compositor).  
* **Backdrop Blur:** Use the HwRender capabilities of GNOME to create a beautiful, blurred transparency effect that matches the shell's top bar.
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
  * `! `: system actions and system controls on Enter. System Actions and controls are:
    * Suspend the system
    * Reboot the system
    * Hibernate the system
    * Power off the system
    * Lock the screen/gnome-session-manager
    * Mute speakers
    * Mute microphone
    * Mute all
    * toggle night light (gnome)
    * toggle dark mode (gnome)
    * toggle Do Not Disturb (gnome)
  * `c `: calculate as the user types, with support for latex math notation.
  * `d `: define words typed in the omnibar/searchbar. pressing enter launched a google search for the definition.
  * `w `: window and workspace switcher. A visual list of open windows categorized by GNOME Workspace. User could type "Firefox" and see results for "Switch to Firefox (Workspace 2)" or "New Firefox Window (Current Workspace)."
  * `
  * Each result should have sub-actions shown when `ctrl + enter` is pressed. This would pull from the result's type and the current context (e.g., for a file: "Open," "Show in Folder," "Email as Attachment").
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
* **Macro Actions:** Users should be able to choose an action type (launch application, execute command, open folder, open file, type text, sleep) and a target (e.g., the name of the application, the command to execute, the folder to open, the file to open, the text to type, the duration to sleep). This should be configurable through a dialog. The user should be able to reorder the action items in the macro.
* **Libadwaita Search Provider:** act as a "Super-Host" for existing GNOME Search Providers. This allows it to instantly pull results from GNOME Contacts, Calendar, Files (Nautilus), and even specialized apps like "GNOME Clocks" or "Characters" using the same D-Bus APIs.

### 🧠 Intelligence & Contextual Awareness

* LLM-Powered System Queries: A local-first AI (using Ollama or similar) that can answer questions about your system or files (e.g., "Where did I save that design document from last Tuesday?" or "What's the grep command to find IP addresses?").
* Semantic Search: Search by concept rather than just filename. Typing "tax stuff" should surface PDFs containing "Revenue," "Invoice," or "W2" even if the filename is SCAN_001.pdf.
* Contextual Actions: If you have a terminal open, the omnibar should offer "Run in current tab" as a primary action for scripts or commands.
