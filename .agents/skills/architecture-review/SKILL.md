---
name: architecture-review
description: Review Tiles Engine technical plans, code changes, data schemas, renderer/editor boundaries, runtime systems, and asset pipeline decisions for maintainability, performance, portability, and future game-type support.
---

# Architecture Review

Use this skill before merging changes that affect engine foundations, project
formats, editor/runtime boundaries, renderer choices, or reusable systems.

## Review Questions

- Does the design keep editor UI separate from reusable engine core?
- Is important behavior represented as data where practical?
- Can the change support top-down and side-scroller games?
- Does it avoid blocking future isometric or 2.5D work?
- Are file formats versioned and validated?
- Is the abstraction justified by current complexity?
- Can the logic be tested outside the full editor?

## Preferred Patterns

- Data-first project models.
- Small crates/modules with clear ownership.
- Explicit schemas for assets, maps, scenes, animations, and rules.
- Spikes before high-risk infrastructure.
- Follow-up issues for intentionally deferred architecture work.

## Output Shape

```markdown
**Architecture Result**
Approve | Approve with follow-ups | Request changes

**Findings**
...

**Follow-Ups**
...
```
