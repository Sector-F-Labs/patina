//! The menu model: what an app's menus say, with nothing about how they are drawn.

use egui::KeyboardShortcut;

/// What an item's mark says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mark {
    #[default]
    None,
    /// A checkbox, on or off.
    Check(bool),
    /// One of a group of which one is on. A checkmark on macOS, which has no radios.
    Radio(bool),
}

/// An item that belongs somewhere particular on some platforms. The item
/// always delivers its own action; the role only says where it lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// "About App": the app menu on macOS.
    About,
    /// "Settings…" or "Preferences…": the app menu on macOS, with ⌘, by default.
    Settings,
    /// "Quit": the app menu on macOS, with ⌘Q by default.
    Quit,
}

/// One line in a menu: an item, a separator or a submenu. The chaining
/// methods apply to what they name and leave the others alone.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry<A> {
    Item {
        label: String,
        action: A,
        shortcut: Option<KeyboardShortcut>,
        enabled: bool,
        mark: Mark,
        role: Option<Role>,
    },
    Separator,
    Submenu {
        label: String,
        enabled: bool,
        entries: Vec<Entry<A>>,
    },
}

impl<A> Entry<A> {
    /// An item that gives `action` when it is clicked or its shortcut pressed.
    pub fn item(label: impl Into<String>, action: A) -> Self {
        Self::Item {
            label: label.into(),
            action,
            shortcut: None,
            enabled: true,
            mark: Mark::None,
            role: None,
        }
    }

    pub fn separator() -> Self {
        Self::Separator
    }

    /// A menu inside a menu, to any depth.
    pub fn submenu(label: impl Into<String>, entries: impl IntoIterator<Item = Entry<A>>) -> Self {
        Self::Submenu {
            label: label.into(),
            enabled: true,
            entries: entries.into_iter().collect(),
        }
    }

    /// The shortcut shown beside an item, and handled for it.
    pub fn shortcut(mut self, to: KeyboardShortcut) -> Self {
        if let Self::Item { shortcut, .. } = &mut self {
            *shortcut = Some(to);
        }
        self
    }

    /// Greyed out and unclickable when `on` is false. An item or a submenu.
    pub fn enabled(mut self, on: bool) -> Self {
        match &mut self {
            Self::Item { enabled, .. } | Self::Submenu { enabled, .. } => *enabled = on,
            Self::Separator => {}
        }
        self
    }

    /// A checkbox item, `on` or not.
    pub fn check(mut self, on: bool) -> Self {
        if let Self::Item { mark, .. } = &mut self {
            *mark = Mark::Check(on);
        }
        self
    }

    /// A radio item, `on` or not.
    pub fn radio(mut self, on: bool) -> Self {
        if let Self::Item { mark, .. } = &mut self {
            *mark = Mark::Radio(on);
        }
        self
    }

    /// Where this item lives on platforms that care (see [`Role`]).
    pub fn role(mut self, to: Role) -> Self {
        if let Self::Item { role, .. } = &mut self {
            *role = Some(to);
        }
        self
    }
}

/// A menu in the bar: File, Edit, View.
#[derive(Debug, Clone, PartialEq)]
pub struct Menu<A> {
    pub label: String,
    pub entries: Vec<Entry<A>>,
}

/// An app's menus, left to right.
#[derive(Debug, Clone, PartialEq)]
pub struct Menus<A> {
    pub menus: Vec<Menu<A>>,
    /// The app's name as the menu bar shows it on macOS.
    pub app_name: Option<String>,
}

impl<A> Default for Menus<A> {
    fn default() -> Self {
        Self {
            menus: Vec::new(),
            app_name: None,
        }
    }
}

impl<A> Menus<A> {
    pub fn new() -> Self {
        Self::default()
    }

    /// The app's name as macOS's menu bar shows it, in capitals if you give it so
    /// ("Photograph"). A packaged app takes it from its `Info.plist`
    /// (`CFBundleName`); this is for a binary run on its own, which would
    /// show its file name ("photograph"). Not used elsewhere.
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Adds a menu to the right of the ones so far.
    pub fn menu(
        mut self,
        label: impl Into<String>,
        entries: impl IntoIterator<Item = Entry<A>>,
    ) -> Self {
        self.menus.push(Menu {
            label: label.into(),
            entries: entries.into_iter().collect(),
        });
        self
    }
}
