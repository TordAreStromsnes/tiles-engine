# GitHub Setup

## Current Repo State

Repository: `TordAreStromsnes/tiles-engine`

Confirmed from the GitHub connector:

- Visibility: public.
- Default branch: `main`.
- Current user permissions: admin, maintain, push, triage, pull.

Confirmed through GitHub CLI on 2026-05-25:

- GitHub CLI is installed and authenticated as `TordAreStromsnes`.
- Project `3` exists at
  `https://github.com/users/TordAreStromsnes/projects/3`.
- Project title is `tiles-engine`.
- Standard labels from [docs/delivery-process.md](delivery-process.md) exist.
- Seed issues #1 through #13 exist and are added to project `3`.

The GitHub connector available here can inspect issues and PRs, but it does not
expose issue creation or GitHub Project creation. Use GitHub CLI for issue and
project write automation.

## Created Seed Issues

- #1: Create GitHub Project And Labels
- #2: Decide License And Contribution Boundaries
- #3: Run Stack Decision Spike
- #4: Define Project Format V0
- #5: Build Editor Shell Spike
- #6: Build Sprite And Asset Schema V0
- #7: Design Humanoid Character Creator MVP
- #8: Build Animation Clip Schema And Walk Cycle Spike
- #9: Build Tile Map And Portal Schema V0
- #10: Research Procedural World Generation Inputs
- #11: Design Scene Composer And Runtime Preview MVP
- #12: Design Generic Interaction Systems
- #13: Build Native Renderer Spike

## Created Scene Runtime Follow-Up Issues

- #31: Build Scene Entity Schema V0
- #32: Build Runtime Preview Loop Slice
- #33: Build Scene Composer Placement Prototype
- #34: Build Interaction Trigger Schema V0
- #35: Design Menus And Save Load After Runtime Preview

## CLI Setup

GitHub CLI needs repo and project access:

```powershell
gh auth login --hostname github.com --git-protocol https --web --scopes "repo,project"
gh auth refresh -s project
```

Create issues with:

```powershell
gh issue create --repo TordAreStromsnes/tiles-engine --title "..." --body-file issue.md --label "tiles-engine,type:research"
```

Add issues to project `3` with:

```powershell
gh project item-add 3 --owner TordAreStromsnes --url <issue-url>
```

## Automation Goal

Future automation can still improve:

- Project field assignment.
- Issue to PR linking checks.
- Duplicate detection against docs/backlog seed entries.
