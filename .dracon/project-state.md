# ProjectState

## Current Focus
One line: Update parse_name tests to reflect that whitespace and empty strings are no longer accepted.

## Completed
- [x] Revised test_parse_name_whitespace to assert None for inputs with leading/trailing whitespace, confirming parse_name no longer trims whitespace.
- [x] Revised test_parse_name_empty to assert None for an empty string, confirming empty input is rejected.
