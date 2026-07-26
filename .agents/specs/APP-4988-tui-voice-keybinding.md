# Spec: TUI configurable hold key for voice input

Linear: APP-4988 — https://linear.app/warpdotdev/issue/APP-4988/tui-configurable-keybinding-to-start-voice-mode
Originating thread: https://warpdev.slack.com/archives/C0BDQDW8V5E/p1784988760470699
Relevant repo: `warp`

== PRODUCT ==

*Summary:* Add a TUI-specific setting for a push-to-talk modifier key. Pressing and holding the configured modifier starts voice recording; releasing the same physical modifier stops recording and begins transcription. The existing `ctrl-s` tap-to-start binding and `/voice` command remain independent and unchanged.

*Behavior:*
1. **Default remains unchanged.** `agents.voice.voice_input_hold_key` defaults to `none`. With that value, no hold-to-record interaction is installed and `ctrl-s` remains the default voice keybinding.
2. **Press starts recording.** When the session composer owns input and the terminal supports keyboard enhancement, pressing the configured modifier starts voice recording. Repeated presses and a press while another voice session is active do nothing.
3. **Release stops only the matching hold session.** Releasing the same left/right modifier that successfully started the hold session stops recording and begins transcription. Unrelated modifier releases do nothing.
4. **Other voice entry points stay independent.** Releasing the configured modifier must not stop voice started by `ctrl-s`, `/voice`, or another interaction. If the hold-started recording ends through Escape, Enter, failure, cancellation, or completion, the held-key marker is cleared before any later release can affect another recording.
5. **Existing stop controls remain available.** Escape and Enter continue to stop or cancel voice according to the existing TUI voice lifecycle. They remain a fallback if a terminal fails to deliver a modifier release.
6. **Supported values reflect TUI input capabilities.** Values are `none`, `alt_left`, `alt_right`, `control_left`, `control_right`, `shift_left`, `shift_right`, `super_left`, and `super_right`. `fn` is not accepted because crossterm cannot report it. Super/Command remains best-effort because many host terminals or operating systems intercept it.
7. **Left and right remain distinct.** A configured left modifier does not match the corresponding right modifier, and vice versa.
8. **Physical modifier semantics match the GUI.** If the configured modifier is held as part of a chord, voice is active for the duration that modifier is held. The chord's existing action still fires independently. For example, configuring `control_left` means a left-Control `ctrl-c` press both activates push-to-talk while Control is held and dispatches the existing `ctrl-c` action.
9. **Graceful degradation.** On terminals without Kitty keyboard enhancement support, the configured hold interaction is inactive. It does not swallow input or change ordinary keybindings.
10. **Settings apply live.** The session view reads `TuiVoiceSettings` while rendering and subscribes to changes. A changed hold key applies when the view rebuilds without re-registering the keymap.

== TECH ==

*Settings:*
- `app/src/settings/tui_voice.rs:16` defines the TUI-only `TuiVoiceInputHoldKey` enum. It intentionally excludes `Fn`.
- `app/src/settings/tui_voice.rs:30` registers `TuiVoiceSettings.voice_input_hold_key` at `agents.voice.voice_input_hold_key`, with `SettingSurfaces::TUI`, `SyncToCloud::Never`, and default `none`.
- The GUI keeps its existing `agents.voice.voice_input_toggle_key` setting and `VoiceInputToggleKey` type. The two settings have different accepted value sets and separate schema entries, so the schema generator requires no duplicate-path merge behavior.

*Terminal input protocol:*
- The lower `factory/tui-key-lifecycle` branch enables `DISAMBIGUATE_ESCAPE_CODES | REPORT_EVENT_TYPES | REPORT_ALL_KEYS_AS_ESCAPE_CODES` on enhancement-capable terminals and exposes repeat-aware `TuiEvent::KeyDown` plus general `TuiEvent::KeyUp`.
- `KeyEventDetails::physical_key` carries a platform-neutral `KeyCode` when crossterm reports physical identity. Modifier events therefore preserve left/right identity without a voice-specific core enum.
- Repeats remain key-down events for existing keymap, editor, and PTY behavior. Releases bypass the keymap and are not forwarded to the PTY.

*Dispatch and state ownership:*
- The lower branch's `TuiEventHandler::on_key_event` lets an element wrapper observe key-down and key-up events after its child declines them, with explicit propagation.
- `crates/warp_tui/src/terminal_session_view.rs:677` preserves the existing `tui:session:start_voice_input` `ctrl-s` binding and `StartVoiceInput` action.
- `crates/warp_tui/src/terminal_session_view.rs` wraps the active session tree with a generalized key-event handler and matches only the configured physical `KeyCode`. Initial key-down starts; key-up stops; repeated key-down propagates without retriggering.
- `TuiTerminalSessionAction::ToggleVoiceInput { key, state }` is separate from `StartVoiceInput`, so release handling remains independent of the ordinary keymap.
- The session stores the exact shared `KeyCode` that successfully started recording. Matching release clears the marker before stopping. Any `StateChanged` event leaving `Listening` also clears the marker.
- `crates/warp_tui/src/input/view.rs:366` exposes the existing voice model's stop operation to the session owner. The recording, transcription, Escape, and Enter lifecycle remains owned by the existing TUI voice model and input view.

*Compatibility and blast radius:*
- The lower branch owns terminal-protocol and general key-lifecycle compatibility. This branch only consumes physical modifier key-down/key-up events.
- Key-up bypasses keymap matching, so chord actions such as `ctrl-s`, `ctrl-c`, and `ctrl-v` still dispatch through their existing key-down paths. As in the GUI, the configured physical modifier also activates push-to-talk while it is held for a chord.
- The session-level wrapper handles only the configured physical key; unrelated and repeated key events propagate.
- If a host terminal does not report the configured modifier or its release, the feature degrades without affecting other input. Escape and Enter remain available to end recording.

*Validation:*
1. `cargo check -p warpui_core --features tui --all-targets` and `cargo check -p warp_tui --all-targets` compile cleanly.
2. The lower branch's complete `warpui_core` and `warp_tui` suites cover Press/Repeat/Release conversion, keymap behavior, child-first lifecycle dispatch, PTY forwarding, and keyboard protocol setup.
3. `cargo nextest run -p warp_tui -E 'test(/voice_hold|voice_input_keeps_ctrl_s|footer_renders_voice/)'` covers setting-to-modifier mapping, exact-side matching, release-to-stop, stale-hold cleanup, release after composer ownership changes, `ctrl-s` independence, and user guidance.
4. `cargo nextest run -p warp -E 'test(/tui_voice_setting/)'` covers the setting default, round trip, TOML path, local-only sync policy, and TUI-only schema surface.
5. `cargo nextest run -p warpui_core --features tui` and `cargo nextest run -p warp_tui` pass as crate-level regressions.
6. `./script/format` and the repository clippy command pass.
7. Real-terminal verification on a Kitty-protocol-capable terminal confirms: the configured physical modifier starts on Press and stops on Release; the opposite side does nothing; `none` disables the hold interaction; `ctrl-s` still starts independently; Escape/Enter still end recording; and a chord using the configured modifier continues to dispatch its normal action while voice remains active for the physical hold.
