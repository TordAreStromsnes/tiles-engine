# Top-Down RPG Project Template

Issue: #151

The project template layer wraps deterministic starter generation in a project
creation flow. It creates normal `.tilesproj` folders and records template
provenance in `manifest.json` so generated projects can be audited, regenerated,
or migrated later.

## Templates

The MVP catalog contains two top-down RPG/adventure choices:

- `template.project.top-down-adventure.starter.v0`: starter terrain, generated
  PNG sprites, starter world, scene, guide dialogue, and trigger actions.
- `template.project.top-down-adventure.empty.v0`: manifest, registry, folders,
  and top-down defaults only.

Both templates use:

- `topDown` as the initial project target
- `gridFourWay` movement
- five character view assumptions: front, back, left, right, and top-down
- project-local editable assets
- PNG as the pixel sprite image format
- `safety.top-down-rpg.standard.v0` as the initial safety budget profile marker

Side-scroller support remains a future template instead of being mixed into the
top-down starter. This keeps the first project path focused while preserving the
engine schemas that can support side-scroller worlds later.

## Desktop Flow

The Tauri desktop shell exposes:

- `list_project_templates`
- `create_project_from_template`

The create command rejects non-empty target folders, generates or copies all
starter files into the target project folder, and writes manifest/registry files
with template provenance. Starter content is generated in Rust and emitted as
normal JSON and PNG files, not hidden built-ins.
