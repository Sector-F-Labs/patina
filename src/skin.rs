//! Skins: TOML files in the suite skin format, which any Patina app can
//! read. Moved from UnAmp (its ADR-0024 and ADR-0008), with the split between
//! Patina and the apps in `docs/adr/0004-skins-in-patina.md`, and the format
//! for skin authors in `docs/skinning.md`.
//!
//! The shared sections (`[colors]`, `[controls]`, `[window]`) mean the same
//! in every app and are read here. Each app keeps its own parts under
//! `[app.<name>]` and reads them with [`Skin::app_section`]; other apps'
//! sections are left alone.
//!
//! "Default" is stock egui following the desktop's light/dark setting.
//! Others are built in ([`BUILT_IN`], plus any an app adds) or loaded from
//! the app's own skins folder ([`app_dir`]). Each app has its own folder;
//! a skin file still works in any app it's copied into.

use std::path::{Path, PathBuf};

use egui::{Color32, CornerRadius, Shadow, Stroke, Theme as EguiTheme, ThemePreference};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

use crate::frame::{FrameStyle, WINDOW_RADIUS};

pub const DEFAULT_SKIN: &str = "Default";
/// The skin format this build reads and writes.
pub const FORMAT: u32 = 1;

/// Skins built into Patina, as (file name, contents), offered by every app.
pub const BUILT_IN: &[(&str, &str)] = &[
    ("dark-orange.toml", include_str!("../skins/dark-orange.toml")),
    ("calendar.toml", include_str!("../skins/calendar.toml")),
    ("notebook.toml", include_str!("../skins/notebook.toml")),
    ("steam-classic.toml", include_str!("../skins/steam-classic.toml")),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Base {
    Dark,
    Light,
}

/// A skin file. Unknown keys are reported but never fatal, so a skin
/// written for a newer format or another app still loads.
#[derive(Debug, Clone, Deserialize)]
pub struct Skin {
    /// The skin format version; missing means UnAmp's pre-suite format.
    pub format: Option<u32>,
    pub name: String,
    #[serde(default)]
    pub author: Option<String>,
    /// `None` only for the Default skin, which follows the desktop.
    #[serde(default)]
    pub base: Option<Base>,
    /// Rounding for panels, menus and controls.
    #[serde(default)]
    pub corner_radius: Option<u8>,
    #[serde(default)]
    pub shadows: Option<bool>,
    #[serde(default)]
    pub colors: Colors,
    #[serde(default)]
    pub controls: Controls,
    #[serde(default)]
    pub window: WindowSkin,
    /// The `[app.*]` sections, unread until an app asks for its own.
    #[serde(default)]
    pub app: toml::Table,
    /// Where it was loaded from; `None` for built-ins.
    #[serde(skip)]
    pub path: Option<PathBuf>,
    /// Ignored keys and other things worth telling the author.
    #[serde(skip)]
    pub warnings: Vec<String>,
}

/// The base palette every app uses.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Colors {
    /// Panels and the windows inside the app.
    pub background: Option<Hex>,
    /// Inset areas: text fields, lists, slider tracks.
    pub surface: Option<Hex>,
    /// Alternate rows in lists.
    pub surface_alt: Option<Hex>,
    pub border: Option<Hex>,
    pub text: Option<Hex>,
    /// Headings, and text on hovered or pressed controls.
    pub text_strong: Option<Hex>,
    pub text_weak: Option<Hex>,
    /// Selection and anything that's on or current.
    pub accent: Option<Hex>,
    /// Text on top of `accent`.
    pub accent_text: Option<Hex>,
    pub link: Option<Hex>,
    pub warning: Option<Hex>,
    pub error: Option<Hex>,
}

/// Buttons, sliders, checkboxes, drop-downs and other widgets.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Controls {
    pub background: Option<Hex>,
    pub hover: Option<Hex>,
    pub pressed: Option<Hex>,
    pub text: Option<Hex>,
    pub text_hover: Option<Hex>,
    pub border: Option<Hex>,
}

/// The app's own window frame: its title bar, edge and corners.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WindowSkin {
    pub title_bar: Option<Hex>,
    pub title_text: Option<Hex>,
    pub border: Option<Hex>,
    /// Corner rounding of the whole window; 15 px (borrowed from GNOME) when unset.
    pub corner_radius: Option<u8>,
    /// Minimise, maximise and close: skinned apart from `[controls]`.
    #[serde(default)]
    pub controls: WindowControls,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WindowControls {
    pub icon: Option<Hex>,
    pub icon_hover: Option<Hex>,
    pub hover: Option<Hex>,
    pub pressed: Option<Hex>,
    pub close_hover: Option<Hex>,
    pub close_icon_hover: Option<Hex>,
}

/// A colour written as `#RRGGBB` or `#RRGGBBAA`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hex(pub Color32);

impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        parse_hex(&s)
            .map(Hex)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid colour {s:?}, expected #RRGGBB or #RRGGBBAA")))
    }
}

fn parse_hex(s: &str) -> Option<Color32> {
    let hex = s.strip_prefix('#')?;
    let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    match hex.len() {
        6 => Some(Color32::from_rgb(byte(0)?, byte(2)?, byte(4)?)),
        8 => Some(Color32::from_rgba_unmultiplied(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
        _ => None,
    }
}

impl Skin {
    pub fn default_skin() -> Self {
        Self {
            format: Some(FORMAT),
            name: DEFAULT_SKIN.to_string(),
            author: None,
            base: None,
            corner_radius: None,
            shadows: None,
            colors: Colors::default(),
            controls: Controls::default(),
            window: WindowSkin::default(),
            app: toml::Table::new(),
            path: None,
            warnings: Vec::new(),
        }
    }

    /// Whether this skin follows the desktop's light/dark setting, as only
    /// Default does; any other fixes its `base`.
    pub fn follows_desktop(&self) -> bool {
        self.base.is_none()
    }

    /// Parses a skin. Keys in the shared sections that this build doesn't
    /// know are listed in `warnings` rather than rejected.
    pub fn parse(toml_text: &str) -> Result<Self, String> {
        let table: toml::Table = toml::from_str(toml_text).map_err(|e| e.to_string())?;
        if !table.contains_key("format") {
            return Err(format!(
                "written in the old skin format; add `format = {FORMAT}` and update it as described in Patina's docs/skinning.md"
            ));
        }
        let (mut skin, ignored): (Skin, _) = deserialize(toml::Value::Table(table), "")?;
        skin.warnings.extend(ignored);
        match skin.format {
            Some(f) if f > FORMAT => skin.warnings.push(format!(
                "written for skin format {f}; this build reads format {FORMAT}, so newer parts are left out"
            )),
            _ => {}
        }
        Ok(skin)
    }

    /// The app's own `[app.<name>]` section, and warnings for keys `T`
    /// doesn't know. A missing section is `T::default()`; a bad value is an
    /// error.
    pub fn app_section<T: DeserializeOwned + Default>(&self, name: &str) -> Result<(T, Vec<String>), String> {
        match self.app.get(name) {
            Some(value) => deserialize(value.clone(), &format!("app.{name}.")),
            None => Ok((T::default(), Vec::new())),
        }
    }

    /// Applies the skin to egui. Default follows the desktop; any other
    /// fixes egui's light or dark theme, in both of egui's slots so a desktop switch can't
    /// swap it out.
    pub fn apply(&self, ctx: &egui::Context) {
        let Some(base) = self.base else {
            ctx.set_theme(ThemePreference::System);
            for slot in [EguiTheme::Dark, EguiTheme::Light] {
                ctx.set_style_of(slot, egui::Style { visuals: slot.default_visuals(), ..Default::default() });
            }
            return;
        };
        let theme = match base {
            Base::Dark => EguiTheme::Dark,
            Base::Light => EguiTheme::Light,
        };
        let visuals = self.visuals(theme.default_visuals());
        ctx.set_theme(theme);
        for slot in [EguiTheme::Dark, EguiTheme::Light] {
            ctx.set_style_of(slot, egui::Style { visuals: visuals.clone(), ..Default::default() });
        }
    }

    /// `base` (egui's visuals for this skin's `base`) with the skin's
    /// colours, rounding and shadows applied. Anything unset keeps `base`'s.
    pub fn visuals(&self, mut v: egui::Visuals) -> egui::Visuals {
        let c = &self.colors;
        let k = &self.controls;
        let get = |h: Option<Hex>| h.map(|h| h.0);

        if let Some(bg) = get(c.background) {
            v.panel_fill = bg;
            v.window_fill = bg;
            v.widgets.noninteractive.bg_fill = bg;
            v.widgets.noninteractive.weak_bg_fill = bg;
        }
        if let Some(surface) = get(c.surface) {
            v.extreme_bg_color = surface;
            v.text_edit_bg_color = Some(surface);
            v.code_bg_color = surface;
            // Slider and checkbox tracks: inset, so they stand out from
            // button-coloured backgrounds.
            v.widgets.inactive.bg_fill = surface;
        }
        if let Some(alt) = get(c.surface_alt) {
            v.faint_bg_color = alt;
        }
        if let Some(border) = get(c.border) {
            v.window_stroke.color = border;
            v.widgets.noninteractive.bg_stroke.color = border;
        }
        if let Some(border) = get(k.border).or(get(c.border)) {
            v.widgets.inactive.bg_stroke = Stroke::new(1.0, border);
            v.widgets.hovered.bg_stroke.color = border;
            v.widgets.active.bg_stroke.color = border;
            v.widgets.open.bg_stroke.color = border;
        }
        if let Some(text) = get(c.text) {
            v.widgets.noninteractive.fg_stroke.color = text;
        }
        if let Some(text) = get(k.text).or(get(c.text)) {
            v.widgets.inactive.fg_stroke.color = text;
            v.widgets.open.fg_stroke.color = text;
        }
        if let Some(strong) = get(k.text_hover).or(get(c.text_strong)) {
            v.widgets.hovered.fg_stroke.color = strong;
            v.widgets.active.fg_stroke.color = strong;
        }
        if let Some(weak) = get(c.text_weak) {
            v.weak_text_color = Some(weak);
        }
        if let Some(accent) = get(c.accent) {
            v.selection.bg_fill = accent;
        }
        if let Some(accent_text) = get(c.accent_text) {
            v.selection.stroke.color = accent_text;
        }
        // egui paints buttons with `weak_bg_fill` and slider/checkbox tracks
        // with `bg_fill` (set from `surface` above for idle controls).
        for (fill, widget, track) in [
            (get(k.background), &mut v.widgets.inactive, false),
            (get(k.hover), &mut v.widgets.hovered, true),
            (get(k.pressed), &mut v.widgets.active, true),
            (get(k.background), &mut v.widgets.open, true),
        ] {
            if let Some(fill) = fill {
                widget.weak_bg_fill = fill;
                if track || get(c.surface).is_none() {
                    widget.bg_fill = fill;
                }
            }
        }
        if let Some(link) = get(c.link) {
            v.hyperlink_color = link;
        }
        if let Some(warning) = get(c.warning) {
            v.warn_fg_color = warning;
        }
        if let Some(error) = get(c.error) {
            v.error_fg_color = error;
        }

        if let Some(r) = self.corner_radius {
            let radius = CornerRadius::same(r);
            v.window_corner_radius = radius;
            v.menu_corner_radius = radius;
            for w in [
                &mut v.widgets.noninteractive,
                &mut v.widgets.inactive,
                &mut v.widgets.hovered,
                &mut v.widgets.active,
                &mut v.widgets.open,
            ] {
                w.corner_radius = radius;
            }
        }
        if self.shadows == Some(false) {
            v.window_shadow = Shadow::NONE;
            v.popup_shadow = Shadow::NONE;
        }
        v
    }

    /// The window frame's colours: the `[window]` section over the frame
    /// drawn in `v` (from [`FrameStyle::from_visuals`]), so window controls
    /// fall back to the `[controls]` colours applied to `v`. Call with the
    /// active visuals each frame, as Default's change with the desktop.
    pub fn frame_style(&self, v: &egui::Visuals) -> FrameStyle {
        let w = &self.window;
        let wc = &w.controls;
        let base = FrameStyle::from_visuals(v);
        let or = |h: Option<Hex>, fallback: Color32| h.map(|h| h.0).unwrap_or(fallback);
        FrameStyle {
            title_bar: or(w.title_bar, base.title_bar),
            title_text: or(w.title_text, base.title_text),
            border: or(w.border, base.border),
            radius: w.corner_radius.or(self.corner_radius).unwrap_or(WINDOW_RADIUS),
            controls: crate::frame::WindowControlColors {
                icon: or(wc.icon, base.controls.icon),
                icon_hover: or(wc.icon_hover, base.controls.icon_hover),
                hover: or(wc.hover, base.controls.hover),
                pressed: or(wc.pressed, base.controls.pressed),
                close_hover: or(wc.close_hover, base.controls.close_hover),
                close_icon_hover: or(wc.close_icon_hover, base.controls.close_icon_hover),
                ..base.controls
            },
        }
    }
}

/// Deserializes `value`, listing keys `T` doesn't know (prefixed with
/// `prefix`) instead of failing on them.
fn deserialize<T: DeserializeOwned>(value: toml::Value, prefix: &str) -> Result<(T, Vec<String>), String> {
    let mut ignored = Vec::new();
    let out = serde_ignored::deserialize(value, |path| ignored.push(format!("unknown key `{prefix}{path}` ignored")))
        .map_err(|e| e.to_string())?;
    Ok((out, ignored))
}

/// The app's own skins folder, `~/.config/<app>/skins`, next to its config.
/// Pass the name of the app's config folder (`"notebook"`, `"unamp"`). Load
/// from it, and point the skin menu's Copy and Open at it (ADR-0011).
pub fn app_dir(app: &str) -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(app).join("skins"))
}

/// The folder every app used to share, `~/.config/patina/skins`.
#[deprecated(since = "0.1.7", note = "each app keeps its own skins: use `app_dir` (ADR-0011)")]
pub fn shared_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("patina").join("skins"))
}

/// What `export_built_ins` did, by file name.
#[derive(Debug, Default, PartialEq)]
pub struct Export {
    pub copied: Vec<String>,
    /// Already present; left untouched so edits survive.
    pub skipped: Vec<String>,
}

/// Writes `built_ins` into `dir` so they can be edited there. Never
/// overwrites: an existing file of the same name is kept as is.
pub fn export_built_ins(dir: &Path, built_ins: &[(&str, &str)]) -> Result<Export, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut export = Export::default();
    for (file, text) in built_ins {
        let target = dir.join(file);
        if target.exists() {
            export.skipped.push(file.to_string());
            continue;
        }
        std::fs::write(&target, text).map_err(|e| format!("{}: {e}", target.display()))?;
        export.copied.push(file.to_string());
    }
    Ok(export)
}

/// Default, the built-ins, then the skins in `dirs`, in order, sorted by
/// name. A later skin with an earlier one's name replaces it, so a user's
/// copy replaces a built-in. Returns the skins, and load errors and
/// warnings to show the user.
pub fn load_all(built_ins: &[(&str, &str)], dirs: &[PathBuf]) -> (Vec<Skin>, Vec<String>) {
    let mut skins = vec![Skin::default_skin()];
    let mut errors = Vec::new();
    let add = |skins: &mut Vec<Skin>, skin: Skin| {
        if skin.name != DEFAULT_SKIN {
            skins.retain(|t| t.name != skin.name);
            skins.push(skin);
        }
    };
    for (file, text) in built_ins {
        match Skin::parse(text) {
            Ok(skin) => {
                errors.extend(skin.warnings.iter().map(|w| format!("built-in {file}: {w}")));
                add(&mut skins, skin);
            }
            Err(e) => errors.push(format!("built-in {file}: {e}")),
        }
    }
    for dir in dirs {
        let (found, dir_errors) = load_dir(dir);
        errors.extend(dir_errors);
        for skin in found {
            add(&mut skins, skin);
        }
    }
    skins[1..].sort_by_key(|t| t.name.to_lowercase());
    (skins, errors)
}

fn load_dir(dir: &Path) -> (Vec<Skin>, Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (Vec::new(), Vec::new());
    };
    let mut skins = Vec::new();
    let mut errors = Vec::new();
    for path in entries.flatten().map(|e| e.path()) {
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        match std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|t| Skin::parse(&t)) {
            Ok(mut skin) => {
                errors.extend(skin.warnings.iter().map(|w| format!("{name}: {w}")));
                skin.path = Some(path);
                skins.push(skin);
            }
            Err(e) => errors.push(format!("{name}: {e}")),
        }
    }
    (skins, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("patina-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn built_ins_parse_cleanly_with_distinct_names() {
        let mut names = Vec::new();
        for (file, text) in BUILT_IN {
            let skin = Skin::parse(text).unwrap_or_else(|e| panic!("{file}: {e}"));
            assert!(skin.warnings.is_empty(), "{file}: {:?}", skin.warnings);
            assert!(!names.contains(&skin.name), "{file}: {} twice", skin.name);
            names.push(skin.name);
        }
        assert_eq!(names, ["Dark Orange", "Calendar", "Notebook", "Steam Classic"]);
    }

    #[test]
    fn export_copies_built_ins_once_and_keeps_edits() {
        let dir = temp_dir("export");
        let first = export_built_ins(&dir, BUILT_IN).unwrap();
        let files: Vec<&str> = BUILT_IN.iter().map(|(f, _)| *f).collect();
        assert_eq!(first.copied, files);
        let path = dir.join("dark-orange.toml");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), BUILT_IN[0].1);

        std::fs::write(&path, "edited").unwrap();
        let second = export_built_ins(&dir, BUILT_IN).unwrap();
        assert!(second.copied.is_empty());
        assert_eq!(second.skipped, files);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "edited");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_user_skin_replaces_a_built_in_of_the_same_name() {
        let dir = temp_dir("load");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("mine.toml"), "format = 1\nname = \"Dark Orange\"\nbase = \"light\"").unwrap();
        std::fs::write(dir.join("broken.toml"), "format = 1\nname = \"B\"\n[colors]\ntext = \"green\"").unwrap();
        std::fs::write(dir.join("notes.txt"), "not a skin").unwrap();
        let (skins, errors) = load_all(BUILT_IN, std::slice::from_ref(&dir));
        let names: Vec<&str> = skins.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["Default", "Calendar", "Dark Orange", "Notebook", "Steam Classic"]);
        let mine = &skins[2];
        assert_eq!(mine.base, Some(Base::Light));
        assert!(mine.path.is_some());
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].starts_with("broken.toml"), "{errors:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_hex_colours() {
        assert_eq!(parse_hex("#4C5844"), Some(Color32::from_rgb(0x4C, 0x58, 0x44)));
        assert_eq!(
            parse_hex("#00000080"),
            Some(Color32::from_rgba_unmultiplied(0, 0, 0, 0x80))
        );
        assert_eq!(parse_hex("4C5844"), None);
        assert_eq!(parse_hex("#4C58"), None);
        assert_eq!(parse_hex("#GG5844"), None);
    }

    #[test]
    fn built_in_skins_parse_cleanly() {
        for (file, text) in BUILT_IN {
            let skin = Skin::parse(text).unwrap_or_else(|e| panic!("{file}: {e}"));
            assert!(skin.warnings.is_empty(), "{file}: {:?}", skin.warnings);
        }
    }

    #[test]
    fn unset_colours_keep_egui_defaults() {
        let skin = Skin::parse("format = 1\nname = \"Tiny\"\nbase = \"light\"\n[colors]\naccent = \"#FF0000\"").unwrap();
        let v = skin.visuals(egui::Visuals::light());
        assert_eq!(v.selection.bg_fill, Color32::RED);
        assert_eq!(v.panel_fill, egui::Visuals::light().panel_fill);
    }

    #[test]
    fn window_controls_are_skinned_apart_from_controls() {
        let skin = Skin::parse(
            "format = 1\nname = \"W\"\nbase = \"dark\"\n\
             [controls]\nhover = \"#111111\"\n\
             [window.controls]\nhover = \"#222222\"\nclose_hover = \"#333333\"",
        )
        .unwrap();
        let v = skin.visuals(egui::Visuals::dark());
        assert_eq!(v.widgets.hovered.weak_bg_fill, Color32::from_gray(0x11));
        let frame = skin.frame_style(&v);
        assert_eq!(frame.controls.hover, Color32::from_gray(0x22));
        assert_eq!(frame.controls.close_hover, Color32::from_gray(0x33));
    }

    #[test]
    fn window_controls_fall_back_to_control_colours() {
        let skin = Skin::parse("format = 1\nname = \"W\"\nbase = \"dark\"\n[controls]\nhover = \"#111111\"").unwrap();
        let v = skin.visuals(egui::Visuals::dark());
        assert_eq!(skin.frame_style(&v).controls.hover, Color32::from_gray(0x11));
    }

    #[test]
    fn unknown_keys_are_warnings_and_other_apps_are_skipped() {
        let skin = Skin::parse(
            "format = 1\nname = \"X\"\n[colors]\nbakground = \"#000000\"\n\
             [app.photograph]\nloupe = \"#FFFFFF\"\n[app.unamp]\nspectrom = []",
        )
        .unwrap();
        assert_eq!(skin.warnings, ["unknown key `colors.bakground` ignored"]);
    }

    #[derive(Debug, Default, Deserialize)]
    struct Mine {
        glow: Option<Hex>,
    }

    #[test]
    fn an_app_reads_its_own_section_with_its_own_warnings() {
        let skin = Skin::parse(
            "format = 1\nname = \"X\"\n[app.mine]\nglow = \"#FF0000\"\nglwo = 1\n[app.other]\nx = 1",
        )
        .unwrap();
        assert!(skin.warnings.is_empty(), "{:?}", skin.warnings);
        let (mine, warnings) = skin.app_section::<Mine>("mine").unwrap();
        assert_eq!(mine.glow, Some(Hex(Color32::RED)));
        assert_eq!(warnings, ["unknown key `app.mine.glwo` ignored"]);
        let (absent, _) = skin.app_section::<Mine>("absent").unwrap();
        assert_eq!(absent.glow, None);
        assert!(Skin::parse("format = 1\nname = \"X\"\n[app.mine]\nglow = \"red\"").unwrap().app_section::<Mine>("mine").is_err());
    }

    #[test]
    fn dark_orange_is_photographs_look() {
        let skin = Skin::parse(BUILT_IN[0].1).unwrap();
        let v = skin.visuals(egui::Visuals::dark());
        assert_eq!(v.selection.bg_fill, Color32::from_rgb(0xE9, 0x54, 0x20));
        assert_eq!(v.panel_fill, Color32::from_gray(0x30));
        assert_eq!(v.window_corner_radius, CornerRadius::same(6));
        assert_eq!(skin.frame_style(&v).radius, 15);
    }

    #[test]
    fn bad_values_are_still_errors() {
        let err = Skin::parse("format = 1\nname = \"X\"\n[colors]\ntext = \"green\"").unwrap_err();
        assert!(err.contains("invalid colour"), "{err}");
    }

    #[test]
    fn newer_formats_load_with_a_warning() {
        let skin = Skin::parse("format = 7\nname = \"Future\"\n[hologram]\nglow = 3").unwrap();
        assert!(skin.warnings.iter().any(|w| w.contains("format 7")), "{:?}", skin.warnings);
    }

    #[test]
    fn the_old_format_is_rejected_with_directions() {
        let err = Skin::parse("name = \"Old\"\n[player]\ntime = \"#FFFFFF\"").unwrap_err();
        assert!(err.contains("old skin format") && err.contains("format = 1"), "{err}");
    }

}
