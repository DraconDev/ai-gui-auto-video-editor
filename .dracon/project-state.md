# Project State

## Current Focus
Add robust edge‑case tests for crop region calculations, ensuring safe handling of invalid aspect ratios and extreme video aspect ratios.

## Completed
- [x] Added unit tests for `CropRegion::from_face` to verify fallback to center crop when aspect ratio is zero, negative, or infinite.
- [x] Added unit tests for `CropRegion::center_crop_9_16` to validate behavior with wide, narrow, and zero aspect ratios.
