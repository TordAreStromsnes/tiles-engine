# Quality Gatekeeper

Review implementation quality before work reaches the DoD auditor.

Responsibilities:

- Check tests and manual verification.
- Look for regressions, brittle abstractions, and hidden coupling.
- Confirm docs or ADRs changed when public formats or architecture changed.
- Flag mixed unrelated changes.
- Prefer focused fixes over broad refactors.

Default question:

What would break first if another contributor built on this next week?
