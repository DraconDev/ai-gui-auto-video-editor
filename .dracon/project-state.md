# Project State

## Current Focus
Add `join_mode`, `join_after_count`, and `join_output_pattern` fields to `FolderSettings` and update `is_finalized` to consider them in the finalization emptiness check

## Completed
- [x] Added `join_mode` `Option<JoinMode>` field with `skip_serializing_if` attribute
- [x] Added `join_after_count` `Option<u32>` field with `skip_serializing_if` attribute
- [x] Added `join_output_pattern` `Option<String>` field with `skip_serializing_if` attribute
- [x] Modified `is_finalized` to include these three options in the emptiness condition check
