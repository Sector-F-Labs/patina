# egui pitfalls

Traps any egui app can fall into, found in Patina apps. Unlike [known-issues.md](known-issues.md),
each has a known cause and a fix.

## An idle window burning a core

An egui app should sleep when nothing changes. If it uses a whole core while idle, something is
asking for a new frame on every frame, and that frame asks again. Nothing looks wrong on screen.
The only sign is CPU, so check it:

```bash
# CPU ticks (1/100 s) the app used in 5 s while idle; under ~25 is fine, ~500 is a full core.
a=$(awk '{print $14+$15}' /proc/$PID/stat); sleep 5; b=$(awk '{print $14+$15}' /proc/$PID/stat); echo $((b-a))
```

To find what asks, print `ctx.repaint_causes()` at the top of `App::ui`. It names the source line
of every repaint request behind the frame. Print `ui.input(|i| i.raw.events.clone())` too: a
frame with no repaint cause but the same event every time is being woken by the platform.
`WAYLAND_DEBUG=client` shows what the app sends the compositor each frame.

Three causes so far:

### A viewport command every frame

`ctx.send_viewport_cmd` asks for a repaint itself. Sending one unconditionally from `App::ui`,
even one that changes nothing like the same window title, makes every frame ask for the next.
Teleprinter did this with its title. Remember what was sent and send only a change.

### The input method answering egui (Wayland)

While a text field has focus, egui-winit tells the input method where the cursor is after every
frame that had any input (still the case in egui-winit 0.36). On GNOME's Wayland the input method
answers every such update with an empty `Ime(Preedit)` event. That's input, so the next frame
updates the input method again, and so on. It happens with egui's `TextEdit` and with any widget
that sets `PlatformOutput::ime`, as Notebook's editor does.

Fix: call [`ime::drop_repeated_preedits`](../src/ime.rs) from `App::raw_input_hook`. A second
empty preedit in a row changes nothing (the first already ended any composition), so dropping it
ends the loop without breaking typing through an input method.

### Requesting focus every frame (Wayland)

`Memory::request_focus` and `Response::request_focus` also interrupt any input-method
composition. egui-winit does that by disabling and re-enabling text input, which GNOME answers
with an empty preedit, so the next frame runs and requests focus again. The filter above can't
stop this one: the request, not the event, starts it. md-chat kept its message box focused this
way. Request focus only when the widget doesn't have it:

```rust
if !response.has_focus() {
    response.request_focus();
}
```

A focused text field still repaints a few times a second to blink its cursor. That's expected.
