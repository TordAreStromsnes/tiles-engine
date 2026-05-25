---
name: grill-me
description: Stress-test Tiles Engine product, architecture, roadmap, licensing, issue scope, and definition-of-done decisions through one focused question at a time. Use when a decision is high risk, ambiguous, likely to affect future engine/editor work, or when the user asks to be grilled.
---

# Grill Me

Use this skill to turn an unclear plan into explicit decisions.

## Workflow

1. Restate the current intention in one concise paragraph.
2. Identify the highest-risk unresolved decision.
3. Ask one focused question.
4. Include a recommended answer.
5. Explain why the answer matters for Tiles Engine.
6. After the user answers, update confirmed assumptions.
7. Continue until the issue or plan has clear scope, tradeoffs, and definition
   of done.

## Output Shape

```markdown
**Current Understanding**
...

**Question**
...

**Recommended Answer**
...

**Why This Matters**
...
```

When the decision is mature enough:

```markdown
**Locked Decisions**
...

**Remaining Risks**
...

**Next Action**
...
```

## High-Risk Tiles Engine Topics

Use this skill especially for:

- Stack and renderer choice.
- First game genre target.
- Character creator scope.
- Humanoid versus non-human rig families.
- Project file format.
- Flexible tile size rules.
- Animation representation.
- Runtime scripting and AI logic.
- Asset sharing and licensing.
- Definition of done for large systems.

## Rules

- Ask one question at a time.
- Recommend an answer instead of staying neutral.
- Prefer concrete tradeoffs over broad brainstorming.
- If code or docs can answer the question, inspect them before asking.
- End with a next action that can become an issue.
