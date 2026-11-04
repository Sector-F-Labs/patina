//! SVG files with their CSS variables filled in. Diagrams are often written
//! as `:root{--bg:..}` plus a dark `@media` block and `fill:var(--bg)`, which
//! browsers handle and the rasteriser (resvg) doesn't. This loader hands egui
//! the SVG with each `var(--x)` replaced by its value, the dark set when the
//! UI is dark.

use egui::load::{Bytes, BytesLoadResult, BytesLoader, BytesPoll, LoadError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A file's URI and whether the UI was dark.
type Key = (String, bool);

#[derive(Default)]
pub struct SvgFiles {
    cache: Mutex<HashMap<Key, Arc<[u8]>>>,
}

fn ready(bytes: Arc<[u8]>) -> BytesLoadResult {
    Ok(BytesPoll::Ready { size: None, bytes: Bytes::Shared(bytes), mime: Some("image/svg+xml".into()) })
}

impl BytesLoader for SvgFiles {
    fn id(&self) -> &str {
        egui::generate_loader_id!(SvgFiles)
    }

    fn load(&self, ctx: &egui::Context, uri: &str) -> BytesLoadResult {
        let Some(path) = uri.strip_prefix("file://").filter(|p| p.ends_with(".svg")) else {
            return Err(LoadError::NotSupported);
        };
        let dark = ctx.global_style().visuals.dark_mode;
        let key = (uri.to_string(), dark);
        if let Some(bytes) = self.cache.lock().expect("svg cache").get(&key) {
            return ready(bytes.clone());
        }
        let text = std::fs::read_to_string(path).map_err(|e| LoadError::Loading(e.to_string()))?;
        let bytes: Arc<[u8]> = resolve(&text, dark).into_bytes().into();
        self.cache.lock().expect("svg cache").insert(key, bytes.clone());
        ready(bytes)
    }

    fn forget(&self, uri: &str) {
        self.cache.lock().expect("svg cache").retain(|(u, _), _| u != uri);
    }

    fn forget_all(&self) {
        self.cache.lock().expect("svg cache").clear();
    }

    fn byte_size(&self) -> usize {
        self.cache.lock().expect("svg cache").values().map(|b| b.len()).sum()
    }
}

/// `svg` with its `var(--name)`s replaced by the values `:root` gives them,
/// overridden by the dark `@media` block when `dark`.
pub fn resolve(svg: &str, dark: bool) -> String {
    // The rasteriser maps `sans-serif` and `monospace` to Arial and Courier
    // New, which a Linux machine often lacks: name families it may have.
    let svg = svg
        .replace(",sans-serif", ",Noto Sans,DejaVu Sans,Liberation Sans,Ubuntu,Cantarell,sans-serif")
        .replace(",monospace", ",DejaVu Sans Mono,Liberation Mono,Noto Sans Mono,Ubuntu Mono,monospace");
    let svg = svg.as_str();
    if !svg.contains("var(") {
        return svg.to_string();
    }
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut offset = 0;
    // `:root{..}` blocks: those outside a media query are the base, the one
    // inside `@media (prefers-color-scheme: dark)` the dark override.
    while let Some(found) = svg[offset..].find(":root{") {
        let at = offset + found;
        let before = &svg[..at];
        let in_dark = before.rfind("@media").is_some_and(|m| {
            let tail = &before[m..];
            tail.contains("dark") && tail.matches('{').count() == 1
        });
        let body_start = at + ":root{".len();
        let Some(len) = svg[body_start..].find('}') else { break };
        if !in_dark || dark {
            for decl in svg[body_start..body_start + len].split(';') {
                if let Some((name, value)) = decl.split_once(':')
                    && name.trim().starts_with("--")
                {
                    vars.insert(name.trim().to_string(), value.trim().to_string());
                }
            }
        }
        offset = body_start + len;
    }
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(at) = rest.find("var(") {
        out.push_str(&rest[..at]);
        let inner_start = at + 4;
        let Some(close) = rest[inner_start..].find(')') else { break };
        let inner = &rest[inner_start..inner_start + close];
        let (name, fallback) = inner.split_once(',').unwrap_or((inner, ""));
        out.push_str(vars.get(name.trim()).map(String::as_str).unwrap_or(fallback.trim()));
        rest = &rest[inner_start + close + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = "<svg><style>:root{--bg:#fff;--t:#111;}@media (prefers-color-scheme: dark){:root{--bg:#000;}}\ntext{fill:var(--t)}</style><rect fill=\"var(--bg)\"/></svg>";

    #[test]
    fn variables_take_light_or_dark_values() {
        let light = resolve(SVG, false);
        assert!(light.contains("fill=\"#fff\"") && light.contains("fill:#111"));
        let dark = resolve(SVG, true);
        assert!(dark.contains("fill=\"#000\"") && dark.contains("fill:#111"));
    }

    #[test]
    fn an_svg_without_variables_is_unchanged() {
        assert_eq!(resolve("<svg/>", true), "<svg/>");
        assert!(resolve("<text style=\"font-family:a,sans-serif\"/>", true).contains("Noto Sans"));
    }
}
