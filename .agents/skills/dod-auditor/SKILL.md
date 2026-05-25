---
name: dod-auditor
description: Audit completed Tiles Engine issues, pull requests, plans, and implementation summaries against their definition of done, acceptance criteria, tests, documentation, and follow-up requirements.
---

# DoD Auditor

Use this skill after an implementation or documentation task claims to be done.

## Workflow

1. Read the issue or task statement.
2. Extract every acceptance criterion and definition-of-done item.
3. Inspect the delivered code, docs, tests, and verification notes.
4. Mark each item pass, fail, or unclear.
5. Require follow-up issues for intentionally deferred work.
6. Return one of: pass, pass with follow-ups, fail.

## Audit Focus

- Does the delivered work match the issue?
- Are tests or manual verification appropriate for the risk?
- Did docs or ADRs change when architecture or file formats changed?
- Are limitations visible rather than hidden?
- Are unrelated changes mixed into the result?

## Output Shape

```markdown
**DoD Result**
Pass | Pass with follow-ups | Fail

**Checklist**
- [pass/fail/unclear] ...

**Required Follow-Ups**
...
```
