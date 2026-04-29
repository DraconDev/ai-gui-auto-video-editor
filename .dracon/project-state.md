# Project State
This commit refactors the toast system to present only success and error toasts, simplifies their appearance, and introduces a persistent click handler for dismissing them.

## Completed
- Updated Cargo.lock to match latest dependency versions
- Refactored the toast logic to display only relevant error/toast messages
- Improved visual consistency by standardizing toast size, color, and layout
- Added a manual click dismiss feature within the UI for better control
- Ensured proper removal of stale dismissed toasts during user interaction
