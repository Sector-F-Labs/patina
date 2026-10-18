# 0019. A touchpad flick keeps scrolling after the fingers lift, through `patina::kinetic`

Date: 2026-10-19

## Status

Proposed. The user asked why Telegram Desktop's touchpad scrolling "feels like it has
acceleration" and Patina apps' doesn't, and agreed to this approach. It becomes accepted once
the feel has been tried on a real touchpad.

## Context

On macOS the OS makes kinetic scrolling: after the fingers lift it keeps sending scroll events
that slow down, and egui apps glide without doing anything. Linux has no such layer. libinput
reports the fingers' movement and then `axis_stop` when they lift, and the toolkit (GTK4,
Telegram's own Qt code) makes up the glide. winit 0.30 turns `axis_stop` into a scroll with phase
`Ended` on Wayland; egui 0.35 uses the phases only to switch off its wheel smoothing during a
touchpad scroll, and zeroes the scroll at the lift. `ScrollArea`'s own kinetic velocity is for
drag-to-scroll only. So every Patina app stops dead when the fingers lift.

Doing it inside each app's scroll areas was ruled out: Reader alone has several, and each app
would need the same code. Waiting for egui to do it would leave the apps without it indefinitely.

## Decision

Patina gives apps `kinetic::glide`, called from `App::raw_input_hook` like `ime` and `frame`'s
hooks, so every scroll area in the app glides with no other change:

- While the fingers are down, it notes the scroll each frame. At the lift it holds back the
  `End`, and if the fingers moved at 300 points a second or more over the last 100 ms, it adds a
  scroll each frame that slows exponentially (time constant 0.35 s), then sends the `End` once
  it's below 15 points a second. Its scrolls keep egui in its touchpad state, so egui doesn't
  smooth them again.
- Moving the fingers again takes over from the glide; a click, a key or a touch stops it where
  it is. Fingers held still before lifting don't glide, so a careful scroll stops where it's
  left. A command-modifier scroll (zoom) doesn't glide.
- It does nothing on macOS, where the OS glides already. Mouse wheels and X11 never send the
  `End`, so they don't glide.

## Consequences

- Patina apps on Wayland scroll on a touchpad the way GTK4 apps and Telegram do, for one line
  in each app's input hook.
- The feel rests on three constants picked by reasoning, not by hand on a touchpad yet. They may
  need tuning, and might one day be a setting.
- libinput's hold gesture, which GTK uses to stop a glide as soon as fingers touch the pad, isn't
  in winit, so a glide stops only once the fingers move. Revisit if winit exposes it.
- A glide scrolls whatever is under the pointer, so moving the pointer mid-glide can carry it to
  another scroll area, as on macOS.
- If egui starts gliding on its own, this should go, or the two will add up.
