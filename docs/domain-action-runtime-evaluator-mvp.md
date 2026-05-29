# Domain Action Runtime Evaluator MVP

Issue: #147

The first domain action evaluator lives in `tiles-runtime`, not React. It loads a
validated trigger action document, initializes typed scoped variables from their
defaults, and evaluates curated action events against `RuntimePreview`.

## Supported Now

- `showDialogue`: appends a `RuntimeDialogueRequest`.
- `switchMap`: changes the active map and applies a world spawn when a matching
  world graph is supplied.
- `setVariable`: updates scoped runtime state owned by the evaluator.
- `setLayerVisibility` and `setLayerOpacity`: reuse the existing runtime layer
  action path.
- `spawnParticle`: appends a `RuntimeParticleRequest` placeholder.
- `giveItem`: preserved as a structured placeholder output with a warning until
  inventory exists.

## Diagnostics

The evaluator returns `RuntimeDomainEvaluation` with outputs and diagnostics.
Missing maps, missing layers, unsupported action kinds, absent events, and
placeholder systems report diagnostics instead of panicking.

## Deferred Integration

This issue does not wire the evaluator into the desktop editor or playtest
buttons. That should happen after the native playtest flow has a stable launch
boundary, so editor and exported games can call the same Rust action path.
