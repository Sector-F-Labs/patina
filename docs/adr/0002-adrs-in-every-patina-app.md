# 0002. Recommend ADRs for every Patina app, and keep them for Patina itself

Date: 2026-10-11

## Status

Accepted

## Context

Photograph and UnAmp both record their architecture decisions as ADRs: Michael Nygard's format,
one numbered file per decision in `docs/adr/`, an index README, and a Claude Code skill in
`.claude/skills/adr/` that adds one. UnAmp copied the whole setup from Photograph on purpose.

The records have paid off. Reversals stay visible (Photograph dropped and then restored macOS
over several ADRs). They also say when to revisit something: UnAmp's ADR-0005 named the trigger
that led to this crate.

Patina's decisions affect every app built on it, so they need the same care. The user asked
for ADRs here, and for the practice to be a recommendation for apps built this way.

## Decision

- **Patina keeps ADRs** in `docs/adr/`, with the same template, index, status rules and
  `.claude/skills/adr` skill as Photograph and UnAmp.
- **Every Patina app should keep ADRs the same way.** `docs/design.md` lists it as a principle,
  and new apps copy `docs/adr/README.md`, `0000-template.md` and the skill from here.
- **ADRs Claude writes stay `Proposed`** until the user signs off, as in the apps.
- **No renumbering:** an app's ADRs about a moved component stay in the app. When the app
  switches to Patina, it records that in a new ADR that points to Patina's.

## Consequences

- One convention across the family: someone who knows one repo can find the reasoning in the
  others.
- Decisions about shared code get recorded once, here, rather than in each app.
- It's a little overhead for small decisions. The README's "when to write one" rules limit
  this.
- Recommending ADRs doesn't enforce them. An app that skips them is still a Patina app.
