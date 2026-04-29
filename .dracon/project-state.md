# Project State

## Current Focus
Simplify toast system to success/error only and promote config-save feedback to success toast.

## Completed
- [x] Remove ToastKind::Info and ToastKind::Warning variants and related helper methods (add_info_toast, add_warning_toast).
- [x] Promote “Config saved” auto-save notification from info to success toast.
- [x] Delete unused draw_settings_metric helper (settings metric card UI).
- [x] Drop toast-specific rendering branches for info/warning in settings and reduce color/label branches.
- [x] Update Cargo.lock to latest dependency versions.
