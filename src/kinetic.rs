//! Kinetic scrolling on a touchpad: a flick keeps the view gliding after the
//! fingers lift, slowing to a stop. See ADR-0019.
//!
//! macOS sends the glide itself, as more scroll events. Linux leaves it to the
//! app: libinput reports the fingers' movement and then that they lifted,
//! which winit turns into a scroll with phase `End`. egui ends the scroll
//! there. This holds back that `End`, measures how fast the fingers were
//! moving just before it, and adds a scroll each frame that slows
//! exponentially, ending it once it's too slow to see. Moving the fingers
//! again, a click or a key stops the glide where it is.
//!
//! Mouse wheels never send `End`, and nor does X11, so neither glides.

use egui::{Event, MouseWheelUnit, TouchPhase, Vec2};

/// How far back from the lift the fingers' speed is measured, in seconds.
/// Fingers held still for longer than this before lifting don't glide.
const SAMPLE_WINDOW: f64 = 0.1;

/// How quickly a glide slows: its speed falls by a factor of e every this
/// many seconds, and it travels this many seconds' worth of its starting
/// speed in all.
const TIME_CONSTANT: f32 = 0.35;

/// Points per second below which a lift doesn't glide, so a slow, careful
/// scroll stops where it was left.
const MIN_START_SPEED: f32 = 300.0;

/// Points per second below which a glide ends.
const MIN_SPEED: f32 = 15.0;

#[derive(Clone, Default)]
struct State {
    /// Between a touchpad scroll's `Start` and `End`.
    in_touch: bool,
    /// Each recent frame's time and the scroll the fingers made in it.
    samples: Vec<(f64, Vec2)>,
    /// A glide under way: its speed in points per second, and when it last moved.
    glide: Option<(Vec2, f64)>,
}

fn state_id() -> egui::Id {
    egui::Id::new("patina-kinetic-scroll")
}

/// Turns the end of a touchpad scroll into a glide. Call from
/// `App::raw_input_hook`, before anything else reads the scroll events.
pub fn glide(ctx: &egui::Context, raw_input: &mut egui::RawInput) {
    if cfg!(target_os = "macos") {
        return;
    }
    let Some(now) = raw_input.time else {
        return;
    };
    let mut state = ctx
        .data(|d| d.get_temp::<State>(state_id()))
        .unwrap_or_default();

    if state.glide.is_some() && raw_input.events.iter().any(stops_a_glide) {
        state.glide = None;
        // A scroll of the fingers' own goes on from the held-back `End`;
        // anything else needs it, so egui's scroll stops.
        if !raw_input.events.iter().any(is_touchpad_scroll) {
            raw_input.events.insert(0, end(raw_input.modifiers));
        }
    }

    let mut frame_delta = Vec2::ZERO;
    let mut lifted = false;
    raw_input.events.retain(|event| {
        let Event::MouseWheel {
            unit, delta, phase, ..
        } = event
        else {
            return true;
        };
        match phase {
            TouchPhase::Start => {
                state.in_touch = true;
                state.samples.clear();
            }
            TouchPhase::Move if state.in_touch && *unit == MouseWheelUnit::Point => {
                frame_delta += *delta;
            }
            TouchPhase::End if state.in_touch => {
                state.in_touch = false;
                lifted = true;
                return false;
            }
            TouchPhase::Cancel => state.in_touch = false,
            _ => {}
        }
        true
    });

    if frame_delta != Vec2::ZERO {
        state.samples.push((now, frame_delta));
    }
    state.samples.retain(|(t, _)| now - t <= SAMPLE_WINDOW);

    if lifted {
        let velocity = velocity(&state.samples, now);
        state.samples.clear();
        if velocity.length() >= MIN_START_SPEED && !raw_input.modifiers.command {
            state.glide = Some((velocity, now));
        } else {
            raw_input.events.push(end(raw_input.modifiers));
        }
    } else if let Some((velocity, since)) = state.glide {
        let dt = (now - since) as f32;
        let decay = (-dt / TIME_CONSTANT).exp();
        // The distance the decaying speed covers over `dt`, exactly, so a
        // glide goes as far at any frame rate.
        let delta = velocity * TIME_CONSTANT * (1.0 - decay);
        let velocity = velocity * decay;
        raw_input.events.push(Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta,
            phase: TouchPhase::Move,
            modifiers: raw_input.modifiers,
        });
        if velocity.length() < MIN_SPEED {
            state.glide = None;
            raw_input.events.push(end(raw_input.modifiers));
        } else {
            state.glide = Some((velocity, now));
        }
    }

    if state.glide.is_some() {
        ctx.request_repaint();
    }
    ctx.data_mut(|d| d.insert_temp(state_id(), state));
}

/// The fingers' speed over the samples, in points per second. The oldest
/// sample only marks when the measured stretch began.
fn velocity(samples: &[(f64, Vec2)], now: f64) -> Vec2 {
    let [(first, _), rest @ ..] = samples else {
        return Vec2::ZERO;
    };
    let duration = (now - first) as f32;
    if rest.is_empty() || duration <= 0.0 {
        return Vec2::ZERO;
    }
    rest.iter().fold(Vec2::ZERO, |sum, (_, delta)| sum + *delta) / duration
}

fn is_touchpad_scroll(event: &Event) -> bool {
    matches!(
        event,
        Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            ..
        }
    )
}

fn stops_a_glide(event: &Event) -> bool {
    match event {
        Event::MouseWheel { .. } => true,
        Event::PointerButton { pressed, .. } | Event::Key { pressed, .. } => *pressed,
        Event::Touch { .. } => true,
        _ => false,
    }
}

fn end(modifiers: egui::Modifiers) -> Event {
    Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: Vec2::ZERO,
        phase: TouchPhase::End,
        modifiers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wheel(phase: TouchPhase, y: f32) -> Event {
        Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::new(0.0, y),
            phase,
            modifiers: Default::default(),
        }
    }

    /// Runs one frame's input, at `time`, through the hook and returns what's left.
    fn frame(ctx: &egui::Context, time: f64, events: Vec<Event>) -> Vec<Event> {
        let mut input = egui::RawInput {
            time: Some(time),
            events,
            ..Default::default()
        };
        glide(ctx, &mut input);
        input.events
    }

    /// A flick at 20 points every 16 ms, 1250 points per second, ending with the lift at `t`.
    fn flick(ctx: &egui::Context) -> f64 {
        frame(ctx, 0.0, vec![wheel(TouchPhase::Start, 0.0)]);
        let mut t = 0.0;
        for _ in 0..6 {
            t += 0.016;
            frame(ctx, t, vec![wheel(TouchPhase::Move, 20.0)]);
        }
        t += 0.016;
        assert_eq!(frame(ctx, t, vec![wheel(TouchPhase::End, 0.0)]), vec![]);
        t
    }

    fn scrolled(events: &[Event]) -> f32 {
        events
            .iter()
            .map(|event| match event {
                Event::MouseWheel { delta, .. } => delta.y,
                _ => 0.0,
            })
            .sum()
    }

    #[test]
    fn a_flick_glides_slower_and_slower_then_ends() {
        let ctx = egui::Context::default();
        let mut t = flick(&ctx);
        let mut steps = vec![];
        loop {
            t += 0.016;
            let events = frame(&ctx, t, vec![]);
            steps.push(scrolled(&events));
            if events.last() == Some(&wheel(TouchPhase::End, 0.0)) {
                break;
            }
            assert!(steps.len() < 200, "the glide never ended");
        }
        assert!(steps.windows(2).all(|w| w[1] < w[0]));
        // About the starting speed times the time constant.
        let distance: f32 = steps.iter().sum();
        assert!((300.0..500.0).contains(&distance), "glided {distance}");
    }

    #[test]
    fn fingers_held_still_before_lifting_dont_glide() {
        let ctx = egui::Context::default();
        frame(&ctx, 0.0, vec![wheel(TouchPhase::Start, 0.0)]);
        frame(&ctx, 0.016, vec![wheel(TouchPhase::Move, 20.0)]);
        frame(&ctx, 0.032, vec![wheel(TouchPhase::Move, 20.0)]);
        let lift = frame(&ctx, 0.5, vec![wheel(TouchPhase::End, 0.0)]);
        assert_eq!(lift, vec![wheel(TouchPhase::End, 0.0)]);
        assert_eq!(frame(&ctx, 0.516, vec![]), vec![]);
    }

    #[test]
    fn moving_the_fingers_again_takes_over_from_the_glide() {
        let ctx = egui::Context::default();
        let t = flick(&ctx);
        frame(&ctx, t + 0.016, vec![]);
        let events = vec![wheel(TouchPhase::Start, 0.0), wheel(TouchPhase::Move, 3.0)];
        assert_eq!(frame(&ctx, t + 0.032, events.clone()), events);
        assert_eq!(frame(&ctx, t + 0.048, vec![]), vec![]);
    }

    #[test]
    fn a_click_stops_the_glide() {
        let ctx = egui::Context::default();
        let t = flick(&ctx);
        frame(&ctx, t + 0.016, vec![]);
        let click = Event::PointerButton {
            pos: Default::default(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        };
        assert_eq!(
            frame(&ctx, t + 0.032, vec![click.clone()]),
            vec![wheel(TouchPhase::End, 0.0), click]
        );
        assert_eq!(frame(&ctx, t + 0.048, vec![]), vec![]);
    }

    #[test]
    fn a_mouse_wheel_passes_through() {
        let ctx = egui::Context::default();
        let notch = Event::MouseWheel {
            unit: MouseWheelUnit::Line,
            delta: Vec2::new(0.0, -1.0),
            phase: TouchPhase::Move,
            modifiers: Default::default(),
        };
        assert_eq!(frame(&ctx, 0.0, vec![notch.clone()]), vec![notch]);
        assert_eq!(frame(&ctx, 0.016, vec![]), vec![]);
    }
}
