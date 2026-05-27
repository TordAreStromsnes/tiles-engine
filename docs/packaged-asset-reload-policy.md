# Packaged Asset Reload Policy

Issue: [#101](https://github.com/TordAreStromsnes/tiles-engine/issues/101)

Development hot reload is an editor workflow. Exported games should start from a
simpler and safer rule: bundled package content is immutable runtime input.

## Decision

For the MVP, exported games do not hot-reload packaged assets at runtime. The
runner loads package content from the export manifest `contentRoot`, validates
what it can, and treats that content as read-only until the process exits.

Official patches and user mods need explicit future policies. They must not
silently share the editor hot-reload path.

The first implementation slice is
[#121](https://github.com/TordAreStromsnes/tiles-engine/issues/121), which adds
a small packaged asset mount policy helper.

## Content Classes

| Class | MVP policy |
| --- | --- |
| Bundled package content | Immutable, read-only, loaded through `ExportManifest.assetBundles` and `contentRoot`. |
| Development editor source assets | Mutable, watched only by editor/desktop hot reload systems. |
| Official patch content | Deferred; later requires explicit patch manifest, version rules, and integrity checks. |
| User mod or override folders | Deferred; disabled by default and must not shadow packaged assets silently. |
| Community asset sync | Deferred online service concern, not part of local exported-game runtime. |

## Runtime Behavior

Packaged assets are mounted at startup. Runtime systems should keep stable asset
ids and loaded handles in memory. If package content changes on disk while a game
is running, the MVP runner does not try to detect or reload it.

Missing or invalid packaged content should be handled as follows:

- Critical launch assets, such as the manifest, entry scene, entry map, asset
  registry, and renderer metadata, fail launch with a user-facing error.
- Missing assets referenced by optional content should fail the owning feature
  load and record a clear runtime error. The runtime should not silently swap in
  editor placeholder assets.
- Content hash mismatch, when `ExportManifest.contentHashes` contains an entry,
  means the package is corrupt or modified. Release builds should reject it.
- Development exports may print more detail, but they should still avoid
  mutating packaged content.

## Immutable Versus Modifiable Boundaries

Editor hot reload can watch project-relative source paths because the creator is
actively editing those files. Exported-game runtime content has a different
trust model:

- It may live inside an installer-managed or read-only directory.
- It may be code-signed, hashed, or distributed by a store.
- It should be reproducible for support and bug reports.
- It should not accept arbitrary file replacements without an opt-in policy.

User-modifiable content should later use a separate root under user data, not
the package `contentRoot`. Official patches should use a patch manifest rather
than loose file replacement.

## Runtime And Editor Ownership

`tiles-core` and the exported runner own packaged asset mount policy,
manifest/content validation, and runtime error shapes.

The editor owns development hot reload, source asset watching, import UI, and
creator-facing diagnostics. React panels only display status returned by Rust.

## First Implementation Slice

Implement #121:

- Add a runtime-facing packaged asset mount policy helper.
- Classify bundled content as immutable by default.
- Report missing package assets and hash mismatches using a reusable error
  shape.
- Keep user override roots disabled by default.
- Allow the game runner to call the helper without depending on desktop editor
  code.

## Risks And Limitations

- Installer and store formats can make package directories read-only.
- Mod support and official patch support need different trust rules.
- Strict hash checks can block local tinkering unless development exports expose
  a clearer debug mode.
- A missing asset during gameplay should degrade predictably where possible, but
  critical boot assets should fail fast.
- Future patch manifests must define priority order, rollback behavior, and
  compatibility with save data.

## Open Questions

- Should release exports always reject hash mismatches, or should creators be
  able to opt into a looser local-modding mode?
- Should user mods be loaded before or after official patch manifests?
- Do shared community asset packs need their own signing/licensing metadata
  before a runtime can mount them?
