//! Stops a repaint loop between egui and the input method on Wayland. See
//! docs/egui-pitfalls.md.
//!
//! While a text field has focus, egui-winit tells the input method where the
//! cursor is after every frame that had input. On GNOME's Wayland the input
//! method answers each time with an empty preedit, which is input, so the
//! next frame sends the cursor again: a full core spent redrawing an idle
//! window. A second empty preedit in a row changes nothing (the first already
//! ended any composition), so dropping it ends the loop and leaves typing
//! through an input method as it was.

use egui::{Event, ImeEvent};

fn last_was_empty_id() -> egui::Id {
    egui::Id::new("patina-ime-last-preedit-empty")
}

/// Drops an empty IME preedit that follows another empty one. Call from
/// `App::raw_input_hook`.
pub fn drop_repeated_preedits(ctx: &egui::Context, raw_input: &mut egui::RawInput) {
    let mut last_was_empty = ctx
        .data(|d| d.get_temp::<bool>(last_was_empty_id()))
        .unwrap_or(false);
    raw_input.events.retain(|event| match event {
        Event::Ime(ImeEvent::Preedit { text, .. }) if text.is_empty() => {
            !std::mem::replace(&mut last_was_empty, true)
        }
        Event::Ime(ImeEvent::Preedit { .. } | ImeEvent::Commit(_)) => {
            last_was_empty = false;
            true
        }
        _ => true,
    });
    ctx.data_mut(|d| d.insert_temp(last_was_empty_id(), last_was_empty));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preedit(text: &str) -> Event {
        Event::Ime(ImeEvent::Preedit {
            text: text.into(),
            active_range_chars: None,
        })
    }

    /// Runs one frame's input through the filter and returns what's left.
    fn filter(ctx: &egui::Context, events: Vec<Event>) -> Vec<Event> {
        let mut input = egui::RawInput {
            events,
            ..Default::default()
        };
        drop_repeated_preedits(ctx, &mut input);
        input.events
    }

    #[test]
    fn keeps_the_first_empty_preedit_and_drops_repeats() {
        let ctx = egui::Context::default();
        assert_eq!(filter(&ctx, vec![preedit("")]), vec![preedit("")]);
        // The input method's answers to egui moving the IME area, frame after frame.
        assert_eq!(filter(&ctx, vec![preedit("")]), vec![]);
        assert_eq!(filter(&ctx, vec![preedit(""), preedit("")]), vec![]);
    }

    #[test]
    fn a_composition_keeps_its_closing_empty_preedit() {
        let ctx = egui::Context::default();
        filter(&ctx, vec![preedit("")]);
        assert_eq!(
            filter(&ctx, vec![preedit("ce"), preedit("")]),
            vec![preedit("ce"), preedit("")]
        );
        let commit = Event::Ime(ImeEvent::Commit("测".into()));
        assert_eq!(
            filter(&ctx, vec![commit.clone(), preedit("")]),
            vec![commit, preedit("")]
        );
    }

    #[test]
    fn other_events_pass_through() {
        let ctx = egui::Context::default();
        filter(&ctx, vec![preedit("")]);
        let text = Event::Text("a".into());
        assert_eq!(filter(&ctx, vec![text.clone(), preedit("")]), vec![text]);
    }
}
