# Project State

## Current Focus
Added dependency on `AutoReframeProcessor` for video cropping functionality.

## Context
This change prepares the editor module to utilize the new `AutoReframeProcessor` for automated video cropping, which was recently refactored in related commits.

## Completed
- [x] Added import for `AutoReframeProcessor` to enable future video cropping features

## In Progress
- [x] Implementation of video cropping functionality using the new processor

## Blockers
- Implementation of the actual cropping logic using `AutoReframeProcessor` is pending

## Next Steps
1. Implement video cropping using `AutoReframeProcessor`
2. Add corresponding test cases for the new functionality
