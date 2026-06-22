# lore-gateway spike notes — Phase 0

These notes answer the four de-risking spikes that gate all feature work (CHA-57).
Tested against Lore CLI v0.8.3+201, installed at `~/.local/bin/lore`.

---

## S1 — Lore Rust client crate API

**Finding:** The Lore Rust client crate is **not publicly available**. It is not on
crates.io, and no SDK package was found locally or in any registry. The CLI binary
(`~/.local/bin/lore`) is a statically-linked Mach-O ARM64 executable — no library
artifacts, no headers.

**Impact on architecture:** The "bind in-process to the Lore Rust crate" decision
can not be fulfilled today. The locked decision stands as the long-term target, but
the Phase 1 implementation will use the CLI as the data layer. This is acceptable
because `lore-gateway` is the *only* crate that touches this surface; swapping the
CLI adapter for an in-process crate adapter later is a single-crate change.

**CLI capability coverage — everything we need is there:**

| Operation | CLI command |
|-----------|-------------|
| Repo info | `lore repository info` |
| Branch list | `lore branch list` |
| Branch create/switch | `lore branch create`, `lore branch switch` |
| Branch merge/push | `lore branch merge`, `lore branch push` |
| Revision history | `lore history [--branch] [--revision]` |
| Revision info + delta | `lore revision info [--delta] [--metadata]` |
| Revision diff (path list) | `lore revision diff` |
| File tree | `lore file info <path>` (directory lists children) |
| File info (size, hash) | `lore file info [--revision] [--local]` |
| File content | `lore file write` (write target path), or read file after `lore sync` |
| Status | `lore status [--scan]` |
| Stage/unstage | `lore stage`, `lore unstage` |
| Reset | `lore reset` |
| Commit | `lore commit` / `lore revision commit` |
| Lock/unlock | `lore lock <path>` |
| Push | `lore push` |

**Output format:** CLI output is human-readable text. None of the commands expose
a `--json` or `--format` flag in the v0.8.3 help text. The CLI adapter will parse
structured text output. We should design parsers defensively and version-pin the
CLI binary so output format changes are explicit.

**`live-lore` feature flag plan:** Keep the default build compiling without any
Lore dependency. The feature flag `live-lore` gates the CLI-subprocess adapter.
When the Rust crate becomes available, it replaces the subprocess adapter behind
the same flag.

**Trait coverage:** The `LoreRepo` and `LoreStore` traits in `src/traits.rs` match
this capability set. No additional operations are needed for Phase 1.

---

## S2 — Storage and Merkle introspection

**Finding:** The CLI exposes substantial storage metadata but fragment-level
introspection is partial.

| What we need | Available? | Command |
|-------------|-----------|---------|
| Per-file size | Yes | `lore file info [--local]` |
| Per-file content hash | Yes | `lore file info` |
| Per-revision delta info | Yes | `lore revision info --delta` |
| Repo-level storage stats | Partial | `lore repository dump`, `lore repository verify` |
| Fragment-level dedup map | Uncertain | `lore repository store immutable query` (format TBD) |
| Shared store info | Yes | `lore shared-store info` |
| GC / integrity check | Yes | `lore repository gc`, `lore repository verify` |

**For Phase 5 (repo map):** Per-file size and hash are sufficient for the
circle-packing visualization. Fragment-level metrics are a Phase 6 concern.

**For Phase 6 (developer analytics):** `lore repository store immutable query`
needs to be tested against a real repo to understand its output. Dedup savings
calculation may require cross-referencing file hashes across branches. This is
a known risk to be resolved when implementing Phase 6 — it does not block Phases
1–5.

---

## S3 — Change notifications

**Finding:** Native change notification subscription is available.

```
lore notification subscribe [seconds]
```

This is a blocking command that emits events on stdout for the given duration
(or indefinitely if `seconds` is omitted).

`lore service run/start/stop` manages a per-repository background service process
that presumably drives these notifications.

**Plan:** Spawn `lore notification subscribe` as a long-running child process,
read its stdout line by line, and forward events over a WebSocket to connected
browser clients. This gives push-based live updates without polling.

For Phase 1 (read-only), per-request CLI calls are sufficient — no subscription
needed. The subscription becomes important in Phase 3 (write path) where the
UI needs to reflect concurrent changes from other clients.

When the Lore Rust crate becomes available, the subprocess is replaced by an
in-process subscription, which is a `lore-gateway`-only change.

---

## S4 — Converter availability and licensing

| Converter | Purpose | Status | License | Notes |
|-----------|---------|--------|---------|-------|
| OpenImageIO | PNG/TGA/JPG/PSD/EXR previews | Not installed | BSD-3 / MIT | `brew install openimageio` (v3.1.14.1 available) |
| Assimp | FBX/OBJ → glTF | Not installed | BSD-3 | `brew install assimp` (v6.0.5 available) |
| Blender (headless) | `.blend` previews + renders | Installed (`/opt/homebrew/bin/blender` v5.1.2) | GPL-3 | Spawned as subprocess — not linked, so the standalone binary is fine. Docker image bundles it explicitly. |

**Licensing notes:**
- OpenImageIO and Assimp are permissively licensed; they can be bundled in both
  the standalone binary and the Docker image without restriction.
- Blender is GPL-3. Since Duskwell spawns it as a child process (not linking
  against its libraries), Duskwell itself is not a GPL derivative. The Docker
  image bundles Blender explicitly; users of the standalone binary need it
  pre-installed or can skip `.blend` rich previews (`.blend` embedded thumbnails
  are extracted directly without Blender).

**Graceful degradation plan:**
- If OpenImageIO is absent: serve images as raw bytes (browser decodes common
  formats natively). PSD/EXR previews are unavailable; show a placeholder with
  hash, size, and download link.
- If Assimp is absent: no FBX/OBJ → glTF conversion; show metadata only.
- If Blender is absent: extract embedded `.blend` thumbnails (no full render);
  show them.

**For Phase 2:** Install OpenImageIO locally via Homebrew before implementing
image preview endpoints. Pin the version in `xtask` and document in the build
guide.
