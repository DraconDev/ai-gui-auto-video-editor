# Project State

## Current Focus
Implement Keep mode for silence removal and conditionally hide advanced padding/min‑duration sliders

## Completed
- [x] Removed the “Remove Silence” toggle UI element and its associated state handling
- [x] Added a “Keep All” option to the silence mode dropdown and expanded the mode list to three variants
- [x] Updated the dropdown selector call to reflect the new three‑item mode_options array
- [x] Modified the post‑dropdown label to describe all three silence modes (Keep All, Cut, Speed Up)
- [x] Relocated the “Silence Padding” and “Min Silence Duration” sliders inside a conditional block that only renders them when the selected mode is not Keep
- [x] Adjusted the settings update logic so advanced options are only saved when applicable, preserving previous behavior for other modes
- [x] Preserved existing UI for stabilize, color correction, and reframe toggles without change
