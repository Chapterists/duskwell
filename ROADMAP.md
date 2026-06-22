# Duskwell Roadmap

Duskwell is an open source, locally-hosted web dashboard and visualization layer for
[Lore](https://lore.org/), Epic Games' next-generation version control system. It gives both
**visual creators** (artists, writers, producers working with Unreal) and **developers** a single
place to browse a Lore repository, preview and visually diff binary assets, run full version-control
operations, and explore the repo through a map of its structure and history.

This roadmap covers the v1 web release. It is a plan, not a promise; order and scope may shift as
the project (and Lore itself, which is pre-1.0) evolve.

## Why Duskwell

Lore is binary-first and content-addressed, which makes it possible to do things other version
control tooling does poorly: preview and diff large binary assets (images, PSDs, EXRs, 3D models,
Blender files, Unreal assets) directly, cache those previews by content hash so unchanged assets are
never re-rendered, and fetch only what a view needs so browsing never forces a full download. Duskwell
leans into exactly these strengths.

## Principles

- **Two audiences, one app.** A gallery-style creator view and a deeper developer view, switchable.
- **Bind to the library, not the CLI.** Duskwell integrates through the Lore client library, not by
  parsing CLI output.
- **Lazy and content-addressed.** Sparse reads and hash-keyed preview caching keep it fast on very
  large repositories.
- **Graceful degradation.** Every asset shows hash, size, type, history, lock state and a download,
  even when a rich preview is not available.
- **Web first.** A desktop build comes later and reuses the same web UI.

## v1 — Web dashboard

### Phase 0 — Scaffolding and spikes
Project skeleton (Rust core + web app served from a single binary) and the de-risking spikes:
mapping the Lore client API behind a stable internal seam, confirming how much storage and history
detail is reachable, choosing a live-update mechanism, and confirming the asset converters.

### Phase 1 — Read core
Browse a real repository end to end: file tree, status, revision history and the revision graph, with
text and markdown preview and text diff. Loads lazily so large repos stay responsive.

### Phase 2 — Image previews and visual diff
The first creator-facing milestone. Preview and visually compare images (PNG, TGA, JPG, then PSD,
then EXR) with side-by-side, swipe, onion-skin and pixel-difference modes. Previews are cached by
content hash and dedupe across branches and revisions.

### Phase 3 — Write path (full version control)
Stage, commit, branch, merge, lock and push from the UI, with careful handling of concurrent changes
so a branch that moved underneath you is surfaced clearly rather than silently overwritten.

### Phase 4 — 3D previews
In-browser viewing of glTF, FBX and OBJ with a metadata diff, and Blender (.blend) previews via the
file's embedded thumbnail, with richer renders where a headless Blender is available.

### Phase 5 — Repo map
A circle-packing / treemap visualization of the repository (inspired by Git-Truck), sizing and
coloring files by metrics like bytes, fragment count, recency, author, asset type, lock status and
storage cost, with a revision time-slider and click-through to any asset's preview.

### Phase 6 — Developer analytics
Fragment and deduplication maps, a content-addressed tree explorer, storage hot spots and integrity
views, built on the same data as the repo map.

### Phase 7 — Unreal assets, Docker and team mode
Best-effort metadata previews for Unreal assets, a Docker image that bundles the asset converters and
a shared preview cache, optional authentication for small-team network use, and a multi-platform
release pipeline.

> Note: UEFN projects are out of scope for v1. They previously used Oodle compression rather than the
> Zstandard that Lore uses, so they are not yet compatible.

## v2 and beyond

- **Desktop client** wrapping the same web UI in a lightweight native shell.
- Deeper merge tooling for binary assets and pluggable per-content-type preview/diff handlers.
- Editor integrations.

## Contributing

Duskwell is MIT licensed. Issues and pull requests are welcome. Work is organized phase by phase;
each phase has a clear "done when" acceptance criterion. See `CONTRIBUTING.md` (coming soon) for
build instructions and conventions.
