# Roadmap

## What is stable today

- `.gui` parsing, imports, merging, and validation
- CLI inspection commands
- HTML scan for page, section, layout, action, index, and dialog nodes
- nav extraction with several over-detection guards
- dialog trigger extraction and shared-layout promotion

## Current weak points

- ranking primary vs auxiliary navigation
- consolidating multiple URLs that represent one logical page
- distinguishing docs taxonomy from site-wide navigation at larger scale
- recognizing JS-only modal triggers without semantic attributes
- richer dialog ownership inference beyond page/layout scope

## Likely next steps

- stronger nav ranking and grouping
- page alias normalization
- broader dialog trigger heuristics
- richer docs/site taxonomy handling
- optional debug output for intermediate scan stages
