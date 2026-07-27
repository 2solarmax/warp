use ai::api_keys::{ApiKeyManager, ApiKeys, CustomEndpoint, CustomEndpointModel};
use settings::Setting as _;
use warpui::{App, SingletonEntity};

use super::TuiApiKeyRefresher;
use crate::settings::AISettings;
use crate::test_util::settings::initialize_settings_for_tests;

#[test]
fn tui_file_backed_byok_keys_are_loaded_into_agent_request_settings() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        let configured = ApiKeys {
            anthropic: Some("sk-ant-tui".to_owned()),
            custom_endpoints: vec![CustomEndpoint {
                name: "Local Claude".to_owned(),
                url: "https://provider.example/v1".to_owned(),
                api_key: "endpoint-key".to_owned(),
                models: vec![CustomEndpointModel {
                    name: "claude-sonnet".to_owned(),
                    alias: None,
                    config_key: "tui-config-key".to_owned(),
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        AISettings::handle(&app).update(&mut app, |settings, ctx| {
            settings
                .tui_api_keys
                .set_value(configured.clone(), ctx)
                .unwrap();
        });

        ApiKeyManager::handle(&app).update(&mut app, |manager, ctx| {
            manager.subscribe_to_tui_api_key_settings(ctx);
        });

        ApiKeyManager::as_ref(&app).read(&app, |manager, _| {
            let request_keys = manager
                .api_keys_for_request(true, false, None)
                .expect("TUI settings should provide a provider key");
            assert_eq!(request_keys.anthropic, "sk-ant-tui");

            let providers = manager
                .custom_model_providers_for_request(true)
                .expect("TUI settings should provide custom endpoint metadata");
            assert_eq!(providers.providers[0].api_key, "endpoint-key");
            assert_eq!(
                providers.providers[0].models[0].config_key,
                "tui-config-key"
            );
        });
    });
}
