# Known issues

Problems seen in Patina apps that aren't fixed yet, with what's known so far.

## Windows stop redrawing on GNOME Shell until minimised

**Seen:** 2026-10-11, GNOME Shell 50.1 on Wayland, NVIDIA (Vulkan), eframe 0.35 / winit 0.30.13.
Photograph 0.4.4 and Notebook froze within half an hour of each other. Both came back after being
minimised and restored.

**What happens:**

- On Wayland, winit sends `RedrawRequested` only after the compositor's frame callback for the
  window's last frame arrives (`FrameCallbackState::Requested` in winit's Wayland backend).
  GNOME Shell stopped sending those callbacks while the windows were still on screen, so no
  redraw ever came.
- Input still arrives, but every repaint it asks for waits behind the missing callback, so the
  window looks frozen.
- **An app that repaints continuously spins** (Photograph: about 90% of a core). eframe's
  `check_redraw_requests` sets `ControlFlow::Poll` and calls `request_redraw` on every pass. Its
  guard for hidden windows doesn't help: winit can't tell that a window is minimised on Wayland.
- **An app that repaints on demand just sleeps** (Notebook).
- Minimising and restoring makes GNOME Shell send frame callbacks again.

**Evidence:**

- `eu-stack` and `perf` of the stuck processes showed the main thread in winit's
  `pump_events`/`loop_dispatch`, with no egui or wgpu frames at all.
- From the same period, GNOME Shell's journal repeatedly logged
  `meta_window_set_stack_position_no_sync: assertion 'window->stack_position >= 0' failed`. A
  window missing from the stack would plausibly be treated as hidden and get no frame callbacks.
  That link is unproven.

**Suspected:** a GNOME Shell bug, possibly set off by moving or raising windows. Patina's custom
title bar hands every drag to the compositor (`StartDrag`). Not reproduced on purpose yet.

**Workaround:** minimise and restore the window.

**To look into:**

1. Run an app with `WAYLAND_DEBUG=client` logging to a file until it freezes. The log will show
   the last `wl_surface.frame` request that never got a `wl_callback.done`. Note the time of
   GNOME Shell's stack assertion.
2. Find what triggers it: dragging by the title bar, maximising, workspaces, screen lock.
3. Report it to GNOME (mutter) with the log.
4. Check whether a newer winit times out a missing frame callback. If it does, decide whether
   Patina should carry a workaround until GNOME is fixed.
