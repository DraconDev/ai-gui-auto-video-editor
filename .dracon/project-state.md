# Project State

## Current Focus
Refactor folder watcher to limit attempted operations and improve memory management

## Context
The folder watcher was using a HashSet to track attempted operations, which could grow indefinitely. This change replaces it with a VecDeque to enforce a maximum size limit (10,000 items) to prevent memory bloat.

## Completed
- [x] Added MAX_ATTEMPTED constant to limit tracked operations
- [x] Replaced HashSet with VecDeque for bounded tracking

## In Progress
- [ ] None

## Blockers
- None

## Next Steps
1. Verify memory usage with large numbers of files
2. Consider adding metrics to monitor attempted operations
