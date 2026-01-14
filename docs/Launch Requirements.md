# **"Launch" Launcher \- Technical Specifications**

## **1\. Visual Requirements (The "Pill")**

* **Window:** gtk4::ApplicationWindow with decorated(false).  
* **CSS Styling:** Custom CSS provider to apply border-radius: 30px and a semi-transparent background with a subtle blur (if supported by the compositor).  
* **Positioning:** Centered at the top-third of the screen.

## **2\. Interaction Model**

* **Prefix Handling:** \- No prefix: Default to App Search.  
  * l : Switch context to Internal Settings.  
  * ss (Custom): Immediate command execution on Enter. 
  * `shift + super + touchpadoff`: as the default keyboard shortcut that brings up the search bar. The is reconfigurable in the settings.
* **Navigation:**  
  * Up/Down arrows to navigate the list.  
  * Enter to execute the top or selected item.  
  * Esc to close the launcher.

## **3\. Architecture: The "Tree" Approach**

* **The Root:** State struct managed by Arc\<Mutex\<State\>\>.  
* **The Branches (Logic):**  
  * SearchEngine: Coordinates which provider to query based on input string.  
  * AppCache: A background thread or periodic task to list .desktop files and check running status via procfs.  
* **The Leaves (Execution):**  
  * CommandExecutor: Handles std::process::Command calls for shortcuts and apps.  
  * SettingsHandler: Opens Adwaita Dialogs for internal configurations.

## **4\. Key Features**

* **Running App Priority:** Apps found in the process list are moved to the top of the search results and styled with bold text.  
* **Shortcut Registry:** A simple HashMap lookup that takes precedence over search.  
* **Settings Filtering:** Real-time filtering of internal "Launch" commands.