pub mod domain;
pub mod interface;
pub mod infrastructure;
pub mod application;

// Re-export commonly used items for main.rs and tests
pub use infrastructure::ui::app_window::{build_ui, AppContext};
pub use application::use_cases::omnibar::Omnibar;
pub use application::use_cases::execute_command::ExecuteCommand;
pub use application::services::app_cache::AppCacheService;

pub use infrastructure::filesystem::desktop_entry_adapter::LinuxAppRepoAdapter;
pub use infrastructure::filesystem::procfs_adapter::ProcFsMonitorAdapter;
pub use infrastructure::filesystem::fs_adapter::LocalFileSystemAdapter;
pub use infrastructure::system::command_executor_adapter::SystemCommandExecutorAdapter;
pub use infrastructure::services::system_adapter::SystemAdapter;
pub use infrastructure::services::window_adapter::SystemWindowAdapter;
pub use interface::ports::{ISystemPower, IWindowRepository, IFileIndexer, IAppRepository, IProcessMonitor, IFileSystem, IShortcutRepository, IMacroRepository, ICalculator, IDictionaryService, ILLMService};
pub use infrastructure::services::calculator_adapter::MevalCalculatorAdapter;
pub use infrastructure::services::settings_store::SettingsStore;
pub use infrastructure::services::json_shortcut_adapter::JsonShortcutAdapter;
pub use infrastructure::services::json_macro_adapter::JsonMacroAdapter;
pub use infrastructure::services::dictionary_adapter::SmartDictionaryAdapter;
pub use infrastructure::services::llm_adapter::OllamaAdapter;
pub use infrastructure::services::file_indexer::FileIndexerAdapter;
pub use domain::model::App;

pub mod test_utils;
