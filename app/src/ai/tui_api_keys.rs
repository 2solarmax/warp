use ai::api_keys::ApiKeyManager;
use settings::Setting as _;
use warpui::ModelContext;

use crate::settings::{AISettings, AISettingsChangedEvent};

/// Loads the TUI-owned file-backed API-key setting into the shared request
/// manager and keeps it synchronized when the settings file is hot-reloaded.
pub(crate) trait TuiApiKeyRefresher {
    fn subscribe_to_tui_api_key_settings(&mut self, ctx: &mut ModelContext<Self>)
    where
        Self: Sized;
}

impl TuiApiKeyRefresher for ApiKeyManager {
    fn subscribe_to_tui_api_key_settings(&mut self, ctx: &mut ModelContext<Self>) {
        let keys = AISettings::as_ref(ctx).tui_api_keys.value().clone();
        self.set_keys_from_settings(keys, ctx);

        ctx.subscribe_to_model(&AISettings::handle(ctx), |manager, _, event, ctx| {
            if matches!(event, AISettingsChangedEvent::TuiApiKeys { .. }) {
                let keys = AISettings::as_ref(ctx).tui_api_keys.value().clone();
                manager.set_keys_from_settings(keys, ctx);
            }
        });
    }
}

#[cfg(test)]
#[path = "tui_api_keys_tests.rs"]
mod tests;
