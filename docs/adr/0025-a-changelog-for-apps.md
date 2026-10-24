# 0025. A changelog for apps: what changed in each version, and what an app has to change

Date: 2026-10-24

## Status

Accepted

## Context

Patina had no release notes. What changed between versions was in the commits and the ADRs, which
say why a decision was made but not what an app has to do about it. An agent updating an app had to
diff Patina's source to find out. On 0.1.11, for example, every app with a menu bar needed
`.span(frame::header_span(ui))` on its `TitleBar`. Nothing but the session that made the change
said so.

Sector F apps build on `develop`, not on releases, so they take changes hours or days before a
version exists. Notes written only at release time would always be behind what the apps have
already pulled.

We considered one file per version in `docs/`. We chose a single root file instead, because that's
where tools and agents look first, and cargo publishes it with the crate, so an app's agent finds
it in the git checkout or the registry source without knowing Patina's layout.

## Decision

- **`CHANGELOG.md` at the repo root**, newest version first. Each version has these headings, any
  of which can be left out: **Upgrading** (what an app has to change, or should), **Added**,
  **Changed** and **Fixed**. An entry names its ADR where there is one.
- **Upgrading is written for an agent updating an app:** the call to add or remove, with where it
  goes, and what happens without it.
- **An `Unreleased` section at the top is updated in the same commit as the change.** A change that
  alters what apps write, or should write, isn't done until its Upgrading line is there. On a
  release, `Unreleased` is renamed to the version and date, and a new empty one starts.
- Versions 0.1.0 to 0.1.11 were backfilled from the tags, commits, ADRs and the public API's
  diffs.

## Consequences

- An agent updating an app reads one file after `cargo update -p patina-ui` and knows what to
  change. The workspace's `AGENTS.md` says to.
- Each Patina change carries one more edit. Internal changes with nothing for apps to do can
  leave the changelog alone.
- The changelog and the ADRs overlap. The ADR stays the record of why; the changelog only says
  what, and links to the ADR.
- An Upgrading line forgotten on `develop` is invisible until an app breaks or looks wrong, so it
  has to be checked in review.
