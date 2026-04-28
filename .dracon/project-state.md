# Project State

## Current Focus
Made internal GUI state structures public for cross-module access

## Completed
- [x] Changed `FolderState` from private to `pub(crate)` to allow access from other modules
- [x] Changed `WatcherEvent` from private to `pub(crate)` to enable event handling across modules
- [x] Changed `QueuedFile` from private to `pub(crate)` to support file processing in other modules
