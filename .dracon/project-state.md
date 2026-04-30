# Project State

## Current Focus
Added comprehensive unit and integration tests for watermark functionality covering position parsing, coordinate conversion, text escaping, overlay string generation, scaling, and special‑character handling.

## Completed
- [x] add unit tests verifying all `WatermarkPosition` variants produce correct FFmpeg coordinate strings
- [x] add unit tests covering all textual aliases accepted by `parse_name`
- [x] add unit test ensuring special characters in text watermarks are properly escaped for FFmpeg drawtext
- [x] add unit tests confirming overlay position strings match expected FFmpeg syntax for each position
- [x] add integration test exercising `add_watermark` with scaling applied to both input and output videos
- [x] add integration test validating `add_text_watermark` handles special characters correctly in the generated filter chain
