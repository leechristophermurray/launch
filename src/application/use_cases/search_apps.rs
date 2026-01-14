use crate::domain::model::App;
use crate::interface::ports::{IAppRepository, IProcessMonitor};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::sync::Arc;

pub struct SearchApps {
    repo: Arc<dyn IAppRepository + Send + Sync>,
    monitor: Arc<dyn IProcessMonitor + Send + Sync>,
}

impl SearchApps {
    pub fn new(
        repo: Arc<dyn IAppRepository + Send + Sync>,
        monitor: Arc<dyn IProcessMonitor + Send + Sync>,
    ) -> Self {
        Self { repo, monitor }
    }

    pub fn execute(&self, query: &str) -> Vec<App> {
        let mut apps = self.repo.find_apps();
        
        // Update status - might be expensive to do every search, 
        // maybe optimise later or background it.
        self.monitor.update_app_status(&mut apps);

        if query.is_empty() {
            // Return top running apps or just a few apps?
            // If empty, maybe just nothing or pinned/running?
            // Let's return running apps first, then alphabetical.
            apps.sort_by(|a, b| {
                b.is_running.cmp(&a.is_running) // true > false
                    .then_with(|| a.name.cmp(&b.name))
            });
            return apps;
        }

        let matcher = SkimMatcherV2::default();
        let mut scored_apps: Vec<(i64, App)> = apps.into_iter().filter_map(|app| {
            let score = matcher.fuzzy_match(&app.name, query)?;
            Some((score, app))
        }).collect();

        // Sort by score descending
        scored_apps.sort_by(|a, b| b.0.cmp(&a.0));

        scored_apps.into_iter().map(|(_, app)| app).collect()
    }
}
