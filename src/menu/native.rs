//! The menu bar on macOS, through `muda` (ADR-0036).
//!
//! The model is rebuilt into an `NSMenu` whenever what it shows changes, and
//! a click comes back as the action the item was built with. Items with a
//! [`Role`] go to the app menu, which this builds: About, Settings, Services,
//! Hide, Hide Others, Show All and Quit. None of the items is a predefined
//! native one, because those send selectors egui isn't on the receiving end of.
//!
//! `muda`'s items aren't `Send`, and egui runs on the main thread on macOS, so
//! the state lives in a thread local.

use super::model::{Entry, Mark, Menus, Role};
use muda::{
    CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::Accelerator,
};
use std::any::Any;
use std::cell::RefCell;
use std::str::FromStr;
use std::sync::Mutex;

/// Ids of clicked items, from `muda`'s handler (any thread) to the next frame.
static CLICKED: Mutex<Vec<String>> = Mutex::new(Vec::new());

struct Built {
    /// What the menu shows. Equal signatures mean no rebuild.
    signature: String,
    /// The actions, indexed by item id.
    actions: Vec<Box<dyn Any>>,
    // Held so the menu stays alive.
    _menu: Menu,
}

thread_local! {
    static BUILT: RefCell<Option<Built>> = const { RefCell::new(None) };
}

pub(super) fn show<A: Clone + 'static>(ctx: &egui::Context, menus: &Menus<A>) -> Option<A> {
    static HANDLER: std::sync::Once = std::sync::Once::new();
    HANDLER.call_once(|| {
        let ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            if let Ok(mut c) = CLICKED.lock() {
                c.push(e.id.0);
            }
            ctx.request_repaint();
        }));
    });

    // Clicks resolve against what was built when they were made, even if the
    // model has changed since.
    let clicked: Vec<String> = CLICKED
        .lock()
        .map(|mut c| std::mem::take(&mut *c))
        .unwrap_or_default();
    let mut out = None;
    BUILT.with(|b| {
        if let Some(built) = &*b.borrow() {
            for id in clicked {
                if let Some(a) = id
                    .parse::<usize>()
                    .ok()
                    .and_then(|i| built.actions.get(i))
                    .and_then(|a| a.downcast_ref::<A>())
                {
                    out = Some(a.clone());
                }
            }
        }
    });

    let mut sig = format!("{:?}", menus.app_name);
    let mut actions: Vec<Box<dyn Any>> = Vec::new();
    let mut roles = Vec::new();
    let mut bar = Vec::new();
    for menu in &menus.menus {
        sig.push_str(&format!("[{}", menu.label));
        let entries = lower(&menu.entries, &mut sig, &mut actions, &mut roles);
        sig.push(']');
        bar.push((menu.label.clone(), entries));
    }
    let same = BUILT.with(|b| b.borrow().as_ref().is_some_and(|b| b.signature == sig));
    if !same {
        if let Some(name) = &menus.app_name {
            // The menu bar's title for the app comes from the process name when there's
            // no bundle to say.
            use objc2_foundation::{NSProcessInfo, NSString};
            NSProcessInfo::processInfo().setProcessName(&NSString::from_str(name));
        }
        let menu = build(bar, roles, menus.app_name.as_deref().unwrap_or("App"));
        menu.init_for_nsapp();
        BUILT.with(|b| {
            *b.borrow_mut() = Some(Built {
                signature: sig,
                actions,
                _menu: menu,
            });
        });
    }
    out
}

/// An entry with its actions taken out into a list, so what's left is plain data.
enum Lowered {
    Item {
        id: usize,
        label: String,
        accelerator: Option<Accelerator>,
        enabled: bool,
        check: Option<bool>,
    },
    Separator,
    Submenu {
        label: String,
        enabled: bool,
        entries: Vec<Lowered>,
    },
}

struct RoleItem {
    role: Role,
    item: Lowered,
}

fn accelerator(s: &egui::KeyboardShortcut) -> Option<Accelerator> {
    let m = s.modifiers;
    let mut parts = Vec::new();
    if m.command {
        parts.push("CmdOrCtrl");
    } else if m.mac_cmd {
        parts.push("Cmd");
    }
    if m.ctrl && !m.command {
        parts.push("Ctrl");
    }
    if m.alt {
        parts.push("Alt");
    }
    if m.shift {
        parts.push("Shift");
    }
    let key = s.logical_key.name();
    parts.push(key);
    Accelerator::from_str(&parts.join("+")).ok()
}

fn lower<A: Clone + 'static>(
    entries: &[Entry<A>],
    sig: &mut String,
    actions: &mut Vec<Box<dyn Any>>,
    roles: &mut Vec<RoleItem>,
) -> Vec<Lowered> {
    let mut out = Vec::new();
    for e in entries {
        match e {
            Entry::Separator => {
                sig.push('-');
                out.push(Lowered::Separator);
            }
            Entry::Submenu {
                label,
                enabled,
                entries,
            } => {
                sig.push_str(&format!("({label}/{enabled}:"));
                let entries = lower(entries, sig, actions, roles);
                sig.push(')');
                out.push(Lowered::Submenu {
                    label: label.clone(),
                    enabled: *enabled,
                    entries,
                });
            }
            Entry::Item {
                label,
                action,
                shortcut,
                enabled,
                mark,
                role,
            } => {
                let id = actions.len();
                actions.push(Box::new(action.clone()));
                let default = match role {
                    Some(Role::Quit) => Some(egui::KeyboardShortcut::new(
                        egui::Modifiers::COMMAND,
                        egui::Key::Q,
                    )),
                    Some(Role::Settings) => Some(egui::KeyboardShortcut::new(
                        egui::Modifiers::COMMAND,
                        egui::Key::Comma,
                    )),
                    _ => None,
                };
                let shortcut = shortcut.or(default);
                sig.push_str(&format!(
                    "<{label}/{enabled}/{mark:?}/{shortcut:?}/{role:?}>"
                ));
                let item = Lowered::Item {
                    id,
                    label: label.clone(),
                    accelerator: shortcut.as_ref().and_then(accelerator),
                    enabled: *enabled,
                    check: match mark {
                        Mark::None => None,
                        Mark::Check(on) | Mark::Radio(on) => Some(*on),
                    },
                };
                match role {
                    Some(role) => roles.push(RoleItem { role: *role, item }),
                    None => out.push(item),
                }
            }
        }
    }
    out
}

fn build(bar: Vec<(String, Vec<Lowered>)>, roles: Vec<RoleItem>, app: &str) -> Menu {
    let menu = Menu::new();
    let _ = menu.append(&app_menu(roles, app));
    for (label, entries) in bar {
        let sub = Submenu::new(label, true);
        fill(&sub, entries);
        let _ = menu.append(&sub);
    }
    menu
}

fn app_menu(mut roles: Vec<RoleItem>, name: &str) -> Submenu {
    let app = Submenu::new(name, true);
    let mut take = |role: Role| {
        roles
            .iter()
            .position(|r| r.role == role)
            .map(|i| roles.remove(i).item)
    };
    let about = take(Role::About);
    let settings = take(Role::Settings);
    let quit = take(Role::Quit);
    let mut added = false;
    for item in [about, settings].into_iter().flatten() {
        add(&app, item);
        added = true;
    }
    if added {
        let _ = app.append(&PredefinedMenuItem::separator());
    }
    let _ = app.append(&PredefinedMenuItem::services(None));
    let _ = app.append(&PredefinedMenuItem::separator());
    let _ = app.append(&PredefinedMenuItem::hide(None));
    let _ = app.append(&PredefinedMenuItem::hide_others(None));
    let _ = app.append(&PredefinedMenuItem::show_all(None));
    if let Some(quit) = quit {
        let _ = app.append(&PredefinedMenuItem::separator());
        add(&app, quit);
    }
    app
}

fn fill(into: &Submenu, entries: Vec<Lowered>) {
    // A separator at either end, or two in a row (after a role item moved
    // out), is dropped.
    let mut last_separator = true;
    let mut pending = None;
    for e in entries {
        if matches!(e, Lowered::Separator) {
            if !last_separator {
                pending = Some(e);
            }
            last_separator = true;
            continue;
        }
        if let Some(s) = pending.take() {
            add(into, s);
        }
        last_separator = false;
        add(into, e);
    }
}

fn add(into: &Submenu, entry: Lowered) {
    match entry {
        Lowered::Separator => {
            let _ = into.append(&PredefinedMenuItem::separator());
        }
        Lowered::Submenu {
            label,
            enabled,
            entries,
        } => {
            let sub = Submenu::new(label, enabled);
            fill(&sub, entries);
            let _ = into.append(&sub as &dyn IsMenuItem);
        }
        Lowered::Item {
            id,
            label,
            accelerator,
            enabled,
            check: None,
        } => {
            let _ = into.append(&MenuItem::with_id(
                id.to_string(),
                label,
                enabled,
                accelerator,
            ));
        }
        Lowered::Item {
            id,
            label,
            accelerator,
            enabled,
            check: Some(on),
        } => {
            let _ = into.append(&CheckMenuItem::with_id(
                id.to_string(),
                label,
                enabled,
                on,
                accelerator,
            ));
        }
    }
}
