use crate::domain::model::App;
use crate::interface::ports::{IAppRepository, IProcessMonitor};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct AppCacheService {
    cache: Arc<RwLock<Vec<App>>>,
    // We hold strong references to the real adapters to use them in the background thread
    // But we don't expose them directly.
    _handle: Option<thread::JoinHandle<()>>, 
}

impl AppCacheService {
    pub fn new(
        real_repo: Arc<dyn IAppRepository + Send + Sync>,
        real_monitor: Arc<dyn IProcessMonitor + Send + Sync>,
    ) -> Self {
        let cache = Arc::new(RwLock::new(Vec::new()));
        let cache_clone = cache.clone();

        // Initial fetch (blocking, so we have data immediately on startup)
        {
            let mut apps = real_repo.find_apps();
            real_monitor.update_app_status(&mut apps);
            let mut writer = cache.write().unwrap();
            *writer = apps;
        }

        // Background update task
        let handle = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(2)); // Refresh rate
                
                let mut apps = real_repo.find_apps();
                real_monitor.update_app_status(&mut apps);
                
                // Sort by running logic here or let Omnibar do it?
                // Omnibar sorts it anyway. But caching "ready-to-go" data is good.
                // Let's just store the raw updated list.
                
                if let Ok(mut writer) = cache_clone.write() {
                    *writer = apps;
                } else {
                    // Lock poisoned
                    break;
                }
            }
        });

        Self {
            cache,
            _handle: Some(handle),
        }
    }
}

impl IAppRepository for AppCacheService {
    fn find_apps(&self) -> Vec<App> {
        if let Ok(reader) = self.cache.read() {
            reader.clone()
        } else {
            vec![]
        }
    }
}

impl IProcessMonitor for AppCacheService {
    fn get_running_pids(&self) -> Vec<u32> {
        // We aren't caching raw PIDs, only the effect on Apps.
        // If something needs raw PIDs, we might need to cache them too.
        // Checking usage: Omnibar only calls update_app_status, never get_running_pids directly.
        vec![]
    }

    fn update_app_status(&self, apps: &mut [App]) {
        // The apps in our cache ALREADY have status updated.
        // But if this method is called with *external* apps (e.g. from a fresh search if we bypassed cache),
        // we might not be able to help much without caching PIDs.
        // HOWEVER: Omnibar gets apps FROM us (find_apps), then calls update_app_status on them.
        // So they are already updated.
        // We can just do nothing here, OR we can re-sync with our cache if matching names.
        
        // Optimization: logic has moved to background. 
        // If the passed apps match our cache, they are already up to date.
        // If Omnibar calls find_apps() -> gets cached structs (running=true/false) -> calls update_app_status,
        // we can essentially no-op or re-apply from cache.
        
        // Let's implement a cheap "re-apply from cache" just in case apps came from elsewhere?
        // Actually, Omnibar implementation:
        // let mut apps = self.app_repo.find_apps();
        // self.process_monitor.update_app_status(&mut apps);
        
        // If `self.app_repo` IS `AppCacheService`, then `find_apps` returns fully updated apps.
        // Then `update_app_status` is called. We can just return.
    }
}
