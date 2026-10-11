# 0007. Unity in variety: the structure is shared, the skin varies, and GNOME is only a source of defaults

Date: 2026-10-11

## Status

Accepted

## Context

Patina's design language called its skinning idea "diversity within rules" without saying which
rules, and said window controls and corners "follow GNOME/libadwaita". That second part wasn't
true. GNOME's 15 px window radius and its red close button were only taken as defaults, and skins
already change both (Steam Classic is square). Reading Patina as a GNOME follower would also pull
it towards the look-alike apps it was set up to avoid.

What makes the variety work comes from towns. A terrace of identical houses painted in different
colours is beautiful, and so is a street of different buildings in one material. A street where
everything is the same is dull, and one where everything differs is noise. The School of Life's
[How to Make an Attractive City](https://www.youtube.com/watch?v=Hy4QjmKzF1c) argues for order and
variety in balance. Thomas Sharp's "unity in variety"
([*Town and Townscape*](https://researcharchive.ncl.ac.uk/sharp/galrev/tandt.html), 1968) is the
same idea.

## Decision

We will hold the structure constant and let the skin vary:

- **Shared by every app:** the frame's layout, where things go, how they behave, and the skin
  format's shared sections.
- **Varies:** each skin's colour, rounding and shadows, and each app's own skin section and parts.
- **What a skin may do:** restyle anything. It never moves things or changes behaviour.
- **Within a skin:** vary one thing boldly and keep the rest calm. Unusual shapes go with a narrow
  palette, and a bold palette goes with the stock shapes.

Patina does not follow GNOME or any other desktop's look. "Native" means behaving well on the
desktop (compositor moves and resizes, XDG light/dark), not copying its style. The docs call the
15 px radius and `#c01c28` close red defaults borrowed from GNOME, nothing more.

## Consequences

- Reviewing a skin or an app now has a clear question: does it keep the shared structure, and
  does it vary one thing at a time?
- Skin authors get guidance, not a rule the loader enforces. A loud skin that changes everything
  still loads.
- A future change to a default, such as the radius or the close red, needs no GNOME
  justification. It's just a default.
- ADR-0003 still says the radius is libadwaita's, as a record of where the number came from.
  That stays true.
- Revisit if an app needs to change the shared structure itself, e.g. a different header bar
  layout. That's a change to the constant, not a skin, and it needs its own ADR.
