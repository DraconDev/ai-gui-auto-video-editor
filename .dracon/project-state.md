# Project State

## Current Focus
Standardized floating-point duration values in configuration and improved watermark filter syntax

## Completed
- [x] Updated `clip_max_duration` values to use floating-point format (90 → 90.0, 140 → 140.0) for consistency with other duration fields
- [x] Improved watermark filter syntax by using named parameters (`scale_val` and `overlay`) instead of positional arguments
