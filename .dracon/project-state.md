# Project State

## Current Focus
Add duplicate folder functionality, export/import config file handling, and logging of actions.

## Completed
- [x] Implemented duplicate_folder method that clones a folder, appends “_copy” to its paths, updates selection, logs the action, and saves config
- [x] Added export_config_to method that serializes watch folders to pretty JSON and writes to a specified path, logging the export
- [x] Added import_config_from method that reads JSON from a path, deserializes into folders, resets selection, logs the import, and saves config
