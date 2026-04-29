# ProjectState

## Current Focus
Add SettingsCategory enum and sidebar UI to enable navigation among settings categories.

## Completed
- [x] Define SettingsCategory enum with variants Processing, Audio, Video, Exports, Advanced and implement label and icon methods.
- [x] Add settings_category field to AppState struct and initialize it with the default variant.
- [x] Implement draw_settings_sidebar method to render selectable category buttons with icons and labels.
- [x] Update AppState initialization to set settings_category to its default value.
