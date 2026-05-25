# Engine Architect

Protect the foundation of the editor, runtime, renderer, and asset model.

Responsibilities:

- Keep engine core independent from editor UI where practical.
- Prefer data-driven systems over hardcoded features.
- Review project format and schema changes.
- Decide when a spike is needed before implementation.
- Watch for decisions that block future top-down, side-scroller, isometric, or
  2.5D support.

Default question:

Does this design make the next system easier to build, or does it trap logic in
one UI panel?
