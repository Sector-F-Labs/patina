//! A drop shadow for the window, asked of the compositor.
//!
//! A Patina app draws its own frame, with no system decorations, so on
//! Wayland nothing draws a shadow around it unless it asks: GNOME and most
//! compositors add none to a window that draws its own frame. [`request`]
//! asks through KDE's `org_kde_kwin_shadow` protocol, which KWin and the
//! Patina compositor draw: the app hands over eight tiles (the four corners
//! and the four sides) and how far they reach, and the compositor draws them
//! outside the window. The window's own surface stays the size of the window,
//! so tiling, snapping and maximising see the window and not its shadow, which
//! is what drawing the shadow in a margin of the surface can't do through
//! winit (it never lets an app set the window geometry,
//! rust-windowing/winit#3438). A compositor without the protocol draws
//! nothing, and the call does nothing.
//!
//! Call it from the app's `ui` or `update`, with eframe's `Frame` (the window's
//! handle isn't reachable from egui's `Context`, which is why the frame's other
//! helpers can't do it for the app): it does its work once per window. Skip it
//! when the app leaves the frame to the desktop.

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// How the shadow looks, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    /// The blur's standard deviation. The shadow reaches about three times
    /// this far.
    pub blur: f32,
    /// How far the shadow is moved down; a shadow lit from above sits lower.
    pub drop: f32,
    /// How dark it is where it meets the window's edge, 0 to 1 (half of it,
    /// there; the blur is half in, half out).
    pub opacity: f32,
    /// How far the shadow's shape is grown beyond the window's on every side
    /// before it is blurred; negative shrinks it.
    pub spread: f32,
    /// How far the shadow is moved to the right; negative moves it left.
    pub offset_x: f32,
    /// Its colour, red, green and blue.
    pub color: [u8; 3],
}

impl Default for Style {
    fn default() -> Self {
        Self {
            blur: 8.0,
            drop: 3.0,
            opacity: 0.5,
            spread: 0.0,
            offset_x: 0.0,
            color: [0, 0, 0],
        }
    }
}

/// The most any of a style's sizes is taken to be, in pixels: tiles stay
/// small whatever a config says.
const MOST: f32 = 64.0;

/// Asks the compositor to draw `window`'s shadow, in the default [`Style`].
/// Does nothing on other systems than Linux on Wayland, with a compositor
/// that has no shadow protocol, or once it has been asked for this window.
pub fn request(window: &(impl HasWindowHandle + HasDisplayHandle)) {
    request_styled(window, Style::default());
}

/// [`request`], in `style`.
pub fn request_styled(window: &(impl HasWindowHandle + HasDisplayHandle), style: Style) {
    #[cfg(target_os = "linux")]
    wayland::request(window, style);
    #[cfg(not(target_os = "linux"))]
    let _ = (window, style);
}

/// How far the tiles reach in under the window, past its edge. The window's
/// corners are rounded (`frame::WINDOW_RADIUS`) and transparent, so without
/// shadow behind them each corner shows a notch of what's below; this is more
/// than the radius, and the window covers the rest.
const INSET: i32 = 16;

/// One tile: its width, its height and its pixels.
pub type Tile = (i32, i32, Vec<u8>);

/// The tiles for `style`: corners and sides as `(width, height, premultiplied
/// ARGB pixels)`, in the order the protocol's `attach_*` requests come in
/// (left, top left, top, top right, right, bottom right, bottom, bottom
/// left), and the offsets left, top, right, bottom.
///
/// Each pixel is the blurred edge of a rectangle the window's size, moved
/// down by `drop`: a side tile is one pixel across, which the compositor
/// stretches along the side. Every tile also reaches [`INSET`] pixels in under
/// the window, which is why a tile is that much bigger than its offset.
///
/// Public for a client that makes its own Wayland objects and attaches the
/// tiles to a surface of its own, as patina-shell does for a layer surface;
/// an app with eframe's `Frame` uses [`request`].
pub fn tiles(style: Style) -> ([Tile; 8], [i32; 4]) {
    let sigma = style.blur.clamp(0.5, MOST);
    let spread = style.spread.clamp(-MOST, MOST);
    let (sx, sy) = (
        style.offset_x.clamp(-MOST, MOST),
        style.drop.clamp(-MOST, MOST),
    );
    // How far past each edge the shadow is worth drawing: three deviations of
    // the blur from the edge of its shape, which is `spread` out, and moved
    // by the offset.
    let reach = |toward: f32| ((sigma * 3.0 + spread + toward).ceil() as i32).max(1);
    let (left, right) = (reach(-sx), reach(sx));
    let (top, bottom) = (reach(-sy), reach(sy));
    // The share of the shadow still there `d` pixels outside an edge of its
    // shape: half at the edge, none far off.
    let share = |d: f32| 0.5 * erfc((d / (sigma * std::f32::consts::SQRT_2)) as f64) as f32;
    let [red, green, blue] = style.color.map(f32::from);
    let pixel = |across_x: Option<f32>, across_y: Option<f32>| {
        let a = (style.opacity * across_x.map_or(1.0, &share) * across_y.map_or(1.0, &share))
            .clamp(0.0, 1.0);
        // B, G, R, A in memory, each colour premultiplied by the alpha.
        [
            (blue * a).round() as u8,
            (green * a).round() as u8,
            (red * a).round() as u8,
            (a * 255.0).round() as u8,
        ]
    };
    // How far outside the shadow's shape, at the window's left, right, top
    // and bottom edge, a pixel is, negative inside it. A left or top tile
    // starts `left` or `top` outside the window's edge; a right or bottom one
    // starts `INSET` inside it. The shape is `spread` bigger than the window
    // and `sx`, `sy` over, which moves its edges toward one side and away
    // from the other.
    let outside_left = |x: i32| (left - x) as f32 - 0.5 + sx - spread;
    let outside_right = |x: i32| (x - INSET) as f32 + 0.5 - sx - spread;
    let outside_top = |y: i32| (top - y) as f32 - 0.5 + sy - spread;
    let outside_bottom = |y: i32| (y - INSET) as f32 + 0.5 - sy - spread;
    let (wide_left, wide_right) = (left + INSET, right + INSET);
    let (tall_top, tall_bottom) = (top + INSET, bottom + INSET);
    let build = |w: i32, h: i32, f: &dyn Fn(i32, i32) -> [u8; 4]| {
        let mut bytes = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                bytes.extend_from_slice(&f(x, y));
            }
        }
        (w, h, bytes)
    };
    let tiles = [
        build(wide_left, 1, &|x, _| pixel(Some(outside_left(x)), None)),
        build(wide_left, tall_top, &|x, y| {
            pixel(Some(outside_left(x)), Some(outside_top(y)))
        }),
        build(1, tall_top, &|_, y| pixel(None, Some(outside_top(y)))),
        build(wide_right, tall_top, &|x, y| {
            pixel(Some(outside_right(x)), Some(outside_top(y)))
        }),
        build(wide_right, 1, &|x, _| pixel(Some(outside_right(x)), None)),
        build(wide_right, tall_bottom, &|x, y| {
            pixel(Some(outside_right(x)), Some(outside_bottom(y)))
        }),
        build(1, tall_bottom, &|_, y| pixel(None, Some(outside_bottom(y)))),
        build(wide_left, tall_bottom, &|x, y| {
            pixel(Some(outside_left(x)), Some(outside_bottom(y)))
        }),
    ];
    (tiles, [left, top, right, bottom])
}

/// The complementary error function, to about 1e-7 (Numerical Recipes'
/// Chebyshev fit).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    let poly = [
        -1.265_512_23,
        1.000_023_68,
        0.374_091_96,
        0.096_784_18,
        -0.186_288_06,
        0.278_868_07,
        -1.135_203_98,
        1.488_515_87,
        -0.822_152_23,
        0.170_872_77,
    ]
    .iter()
    .rev()
    .fold(0.0, |acc, c| c + t * acc);
    let r = t * (-z * z + poly).exp();
    if x >= 0.0 { r } else { 2.0 - r }
}

#[cfg(target_os = "linux")]
mod wayland {
    use std::collections::HashSet;
    use std::io::Write;
    use std::os::fd::{AsFd, FromRawFd, OwnedFd};
    use std::sync::Mutex;

    use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
    use wayland_backend::sys::client::{Backend, ObjectId};
    use wayland_client::{
        Connection, Dispatch, Proxy, QueueHandle,
        globals::{GlobalListContents, registry_queue_init},
        protocol::{
            wl_buffer::WlBuffer, wl_registry::WlRegistry, wl_shm::WlShm, wl_shm_pool::WlShmPool,
            wl_surface::WlSurface,
        },
    };
    use wayland_protocols_plasma::shadow::client::{
        org_kde_kwin_shadow::OrgKdeKwinShadow, org_kde_kwin_shadow_manager::OrgKdeKwinShadowManager,
    };

    use super::{Style, tiles};

    /// Windows already asked for, by their `wl_surface`.
    static DONE: Mutex<Option<HashSet<usize>>> = Mutex::new(None);

    pub(super) fn request(window: &(impl HasWindowHandle + HasDisplayHandle), style: Style) {
        let (Ok(display), Ok(surface)) = (window.display_handle(), window.window_handle()) else {
            return;
        };
        let (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(surface)) =
            (display.as_raw(), surface.as_raw())
        else {
            return;
        };
        let key = surface.surface.as_ptr() as usize;
        if !DONE.lock().unwrap().get_or_insert_default().insert(key) {
            return;
        }
        // SAFETY: the pointers come from the window's own handles, which are
        // alive as `window` is borrowed.
        if let Err(e) = unsafe { ask(display.display.as_ptr(), surface.surface.as_ptr(), style) } {
            eprintln!("patina: no window shadow: {e}");
        }
    }

    /// Nothing is kept: the shadow lives with the surface in the compositor.
    struct Client;

    /// # Safety
    /// `display` is a live `wl_display`, and `surface` a live `wl_surface` on it.
    unsafe fn ask(
        display: *mut std::ffi::c_void,
        surface: *mut std::ffi::c_void,
        style: Style,
    ) -> Result<(), String> {
        // The window's own connection, on an event queue of our own, so the
        // toolkit's events aren't touched.
        let backend = unsafe { Backend::from_foreign_display(display.cast()) };
        let conn = Connection::from_backend(backend);
        let (globals, mut queue) =
            registry_queue_init::<Client>(&conn).map_err(|e| e.to_string())?;
        let qh = queue.handle();
        let Ok(manager) = globals.bind::<OrgKdeKwinShadowManager, _, _>(&qh, 1..=2, ()) else {
            // The compositor has no shadow protocol.
            return Ok(());
        };
        let shm: WlShm = globals.bind(&qh, 1..=1, ()).map_err(|e| e.to_string())?;
        let id = unsafe { ObjectId::from_ptr(WlSurface::interface(), surface.cast()) }
            .map_err(|e| e.to_string())?;
        let surface = WlSurface::from_id(&conn, id).map_err(|e| e.to_string())?;

        let (tiles, offsets) = tiles(style);
        let total: usize = tiles.iter().map(|(_, _, b)| b.len()).sum();
        let fd = memfd(total)?;
        let mut file = std::fs::File::from(fd);
        for (_, _, bytes) in &tiles {
            file.write_all(bytes).map_err(|e| e.to_string())?;
        }
        let pool = shm.create_pool(file.as_fd(), total as i32, &qh, ());

        let shadow = manager.create(&surface, &qh, ());
        let mut at = 0;
        let buffers: Vec<WlBuffer> = tiles
            .iter()
            .map(|(w, h, bytes)| {
                let buffer = pool.create_buffer(
                    at as i32,
                    *w,
                    *h,
                    w * 4,
                    wayland_client::protocol::wl_shm::Format::Argb8888,
                    &qh,
                    (),
                );
                at += bytes.len();
                buffer
            })
            .collect();
        shadow.attach_left(&buffers[0]);
        shadow.attach_top_left(&buffers[1]);
        shadow.attach_top(&buffers[2]);
        shadow.attach_top_right(&buffers[3]);
        shadow.attach_right(&buffers[4]);
        shadow.attach_bottom_right(&buffers[5]);
        shadow.attach_bottom(&buffers[6]);
        shadow.attach_bottom_left(&buffers[7]);
        shadow.set_left_offset(offsets[0] as f64);
        shadow.set_top_offset(offsets[1] as f64);
        shadow.set_right_offset(offsets[2] as f64);
        shadow.set_bottom_offset(offsets[3] as f64);
        shadow.commit();
        // The compositor has copied the tiles by the time this returns.
        queue.roundtrip(&mut Client).map_err(|e| e.to_string())?;
        for buffer in &buffers {
            buffer.destroy();
        }
        pool.destroy();
        let _ = conn.flush();
        Ok(())
    }

    /// An anonymous file of `size` bytes, for the pool.
    fn memfd(size: usize) -> Result<OwnedFd, String> {
        let name = c"patina-shadow";
        // SAFETY: a plain syscall with a valid name.
        let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        // SAFETY: `fd` is a new descriptor nothing else owns.
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        // SAFETY: the descriptor is valid.
        if unsafe { libc::ftruncate(std::os::fd::AsRawFd::as_raw_fd(&fd), size as libc::off_t) } < 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(fd)
    }

    impl Dispatch<WlRegistry, GlobalListContents> for Client {
        fn event(
            _: &mut Self,
            _: &WlRegistry,
            _: <WlRegistry as Proxy>::Event,
            _: &GlobalListContents,
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
        }
    }

    macro_rules! ignore {
        ($($interface:ty),*) => {$(
            impl Dispatch<$interface, ()> for Client {
                fn event(
                    _: &mut Self,
                    _: &$interface,
                    _: <$interface as Proxy>::Event,
                    _: &(),
                    _: &Connection,
                    _: &QueueHandle<Self>,
                ) {
                }
            }
        )*};
    }
    ignore!(
        WlShm,
        WlShmPool,
        WlBuffer,
        OrgKdeKwinShadowManager,
        OrgKdeKwinShadow
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erfc_matches_known_values() {
        for (x, want) in [
            (0.0, 1.0),
            (1.0, 0.157_299_21),
            (-1.0, 1.842_700_79),
            (3.0, 2.209_05e-5),
        ] {
            assert!((erfc(x) - want).abs() < 1e-6, "erfc({x}) = {}", erfc(x));
        }
    }

    #[test]
    fn tiles_fit_their_offsets_and_fade_out() {
        let (tiles, [left, top, right, bottom]) = tiles(Style::default());
        assert_eq!(tiles[0].0, left + INSET);
        assert_eq!(tiles[2].1, top + INSET);
        assert_eq!(tiles[4].0, right + INSET);
        assert_eq!(tiles[6].1, bottom + INSET);
        for (w, h, bytes) in &tiles {
            assert_eq!(bytes.len(), (w * h * 4) as usize);
        }
        // The left tile is darkest at the window's edge (its last pixel) and
        // almost gone at the far side.
        let alpha = |i: usize, x: usize| tiles[i].2[x * 4 + 3];
        assert!(alpha(0, left as usize - 1) > 40);
        assert!(alpha(0, 0) < 4);
        // Under the window it is darker still, so a rounded corner has
        // shadow behind it.
        assert!(alpha(0, left as usize + INSET as usize - 1) > alpha(0, left as usize - 1));
        // The shadow sits lower: more of it under the window than over.
        assert!(bottom > top);
    }

    #[test]
    fn the_default_style_is_what_it_always_was() {
        let (_, offsets) = tiles(Style::default());
        assert_eq!(offsets, [24, 21, 24, 27]);
    }

    #[test]
    fn spread_and_sideways_offset_move_the_reach() {
        let spread = Style {
            spread: 4.0,
            ..Style::default()
        };
        assert_eq!(tiles(spread).1, [28, 25, 28, 31]);
        let right = Style {
            offset_x: 5.0,
            ..Style::default()
        };
        let (_, [left, _, right_reach, _]) = tiles(right);
        assert_eq!((left, right_reach), (19, 29));
    }

    #[test]
    fn colour_is_premultiplied_and_sizes_are_held_in_check() {
        let red = Style {
            color: [255, 0, 0],
            opacity: 1.0,
            ..Style::default()
        };
        let (red_tiles, [left, ..]) = tiles(red);
        // Under the window, where the shadow is nearly all there is, a pixel
        // is B, G, R, A: red as strong as the alpha, no blue or green.
        let at = (left as usize + INSET as usize - 1) * 4;
        let pixel = &red_tiles[0].2[at..at + 4];
        assert_eq!((pixel[0], pixel[1]), (0, 0));
        assert_eq!(pixel[2], pixel[3]);
        assert!(pixel[3] > 200);
        let huge = Style {
            blur: 1.0e6,
            spread: 1.0e6,
            ..Style::default()
        };
        assert!(tiles(huge).0.iter().all(|(w, h, _)| *w <= 400 && *h <= 400));
    }
}
