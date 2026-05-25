# Delivery Process

## Issue First

Most work should begin as an issue. An issue must include:

- Problem.
- Why now.
- Scope.
- Out of scope.
- Proposed approach.
- Acceptance criteria.
- Definition of done.
- Risks and open questions.
- Test expectations.

Tiny repo hygiene changes can skip issue creation, but any product, engine,
format, editor, or workflow change should have one.

## Agent Flow

1. Product architect clarifies intent and user value.
2. Grill-me facilitator interviews the highest-risk decision if needed.
3. Issue shaper converts the work into a ready issue.
4. Implementer builds the smallest useful change.
5. Architecture reviewer checks long-term fit.
6. Quality gatekeeper checks tests, regressions, and maintainability.
7. DoD auditor checks issue completion before merge.

## Definition Of Done

A change is done when:

- The issue acceptance criteria are met.
- The implementation is tested at the right level.
- The code or docs match the current architecture direction.
- New project/file formats are documented.
- User-facing flows have basic manual verification notes.
- Known limitations are captured in the issue or follow-up issues.
- No unrelated changes were mixed into the work.

## Labels

Recommended labels:

- `tiles-engine`
- `type:feature`
- `type:bug`
- `type:research`
- `type:decision`
- `type:docs`
- `type:quality`
- `area:editor`
- `area:engine-core`
- `area:renderer`
- `area:assets`
- `area:animation`
- `area:maps`
- `area:runtime`
- `area:github`
- `phase:0-foundation`
- `phase:1-spikes`
- `priority:p0`
- `priority:p1`
- `priority:p2`
- `risk:high`

## Project Board Fields

Recommended GitHub Project name: `Tiles Engine`

Recommended fields:

- Status: Backlog, Ready, In Progress, Review, Blocked, Done.
- Phase: Foundation, Spikes, Project Model, Editor Shell, Assets, Animation,
  Maps, Scene Runtime, Systems, Sharing.
- Area: Editor, Engine Core, Renderer, Assets, Animation, Maps, Runtime, GitHub.
- Priority: P0, P1, P2, P3.
- Risk: Low, Medium, High.
- Owner: human or agent role.

## Pull Request Expectations

PRs should include:

- Linked issue.
- Summary of behavior change.
- Testing performed.
- Screenshots or recordings for visual editor changes.
- Remaining risks.
- Follow-up issues if the work intentionally stops short.

## Licensing Expectations

Code, docs, schemas, tests, configuration, and repo-native tooling are
dual-licensed as `MIT OR Apache-2.0`. Asset contributions require an explicit
asset license before they are accepted. See [licensing.md](licensing.md) and
[../CONTRIBUTING.md](../CONTRIBUTING.md).
