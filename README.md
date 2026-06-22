# Duskwell

A locally-hosted web dashboard and visualization layer for [Lore](https://lore.org/), Epic Games' next-generation version control system.

Duskwell gives both **visual creators** (artists, writers, producers) and **developers** a single place to browse a Lore repository, preview and visually diff binary assets, run full version-control operations, and explore the repo through a map of its structure and history. All from a single binary that runs on your machine.

## Why

Lore is binary-first and content-addressed. Duskwell leans into exactly these strengths:

- **Preview large binary assets directly:** images (PNG, TGA, JPG, PSD, EXR), 3D models (glTF, FBX, OBJ), Blender files, Unreal assets
- **Visual diff:** side-by-side, swipe, onion-skin, and pixel-difference modes
- **Hash-keyed preview cache:** unchanged assets across branches and revisions are never re-rendered
- **Lazy browsing:** sparse reads mean large repositories stay fast without a full download

## Status

Early development. See the [roadmap](#roadmap) for the current plan.

## Getting started

### Prerequisites

- Rust (stable) - [install via rustup](https://rustup.rs)
- Node.js 20+ and npm
- [Lore CLI](https://lore.org/) installed and authenticated

### Run

```sh
# Build the web UI and embed it into the binary
cargo xtask build-web

# Start the server (serves at http://127.0.0.1:3000)
cargo run -p toolbelt-core
```

Open `http://127.0.0.1:3000` in your browser.

### Development

```sh
# Run tests
cargo test

# Web dev server (with HMR, proxies API to the Rust server)
cd web && npm run dev

# Regenerate TypeScript types from Rust DTOs
cargo xtask codegen
```

## Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| 0 | Scaffolding & de-risking spikes | Done |
| 1 | Read core: file tree, status, revision history & DAG, text/markdown preview | Next |
| 2 | Image previews & visual diff (PNG to PSD to EXR) | Planned |
| 3 | Write path: stage, commit, branch, merge, lock, push | Planned |
| 4 | 3D previews (glTF/FBX/OBJ, .blend) | Planned |
| 5 | Repo map: circle-packing visualization sized by storage metrics | Planned |
| 6 | Developer analytics: fragment/dedup maps, storage hot spots | Planned |
| 7 | Unreal asset previews, Docker image, optional team auth | Planned |

> UEFN projects are out of scope for v1 (Oodle vs Zstd compression).

## Architecture

```
crates/lore-gateway      # Only crate that talks to Lore; traits + CLI adapter + in-memory fake
crates/toolbelt-core     # Axum HTTP server, routes, config
crates/toolbelt-preview  # Converter dispatch + content-hash preview cache
crates/toolbelt-repomap  # Merkle walk + metric aggregation
crates/toolbelt-types    # Shared DTOs (serde), auto-exported to TypeScript
web/                     # Vite + React + TypeScript frontend
xtask/                   # Build automation
```

Key invariants:
- `lore-gateway` is the only crate that touches the Lore client surface. Everything else depends on the `LoreRepo`/`LoreStore` traits and the in-memory fake, so the app is fully testable without a live Lore instance.
- Browsing never forces a full repository hydration. Tree loading is lazy, previews are fetched by content hash, and metadata probing uses byte-range reads where possible.
- Concurrent write conflicts surface explicitly: a branch that moved while you were working returns HTTP 409 with a "branch moved" payload, never a silent overwrite.

## Contributing

MIT licensed. Issues and pull requests welcome. Work is organized phase by phase; each phase has a clear "done when" acceptance criterion. A contribution guide is coming once Phase 1 lands.

## License

[MIT](LICENSE)
