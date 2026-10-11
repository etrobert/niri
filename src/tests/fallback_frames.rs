use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use wayland_client::protocol::wl_surface::WlSurface;

use super::client::{ClientId, SyncData};
use super::*;

fn map_window(f: &mut Fixture) -> (ClientId, WlSurface) {
    let id = f.add_client();
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);

    let window = f.client(id).window(&surface);
    window.attach_new_buffer();
    window.set_size(100, 100);
    window.ack_last_and_commit();
    f.double_roundtrip(id);

    (id, surface)
}

fn request_frame_after_fallback_timer(
    f: &mut Fixture,
    id: ClientId,
    surface: &WlSurface,
) -> Arc<SyncData> {
    let window = f.client(id).window(surface);
    let data = Arc::new(SyncData::default());
    window.surface.frame(&window.qh, data.clone());
    window.commit();
    f.double_roundtrip(id);

    // Let the throttle expire and the 1 s fallback timer fire.
    std::thread::sleep(Duration::from_millis(1100));
    f.double_roundtrip(id);
    data
}

#[test]
fn hidden_window_gets_fallback_frames() {
    let mut f = Fixture::new();
    f.add_output(1, (1920, 1080));
    let (id, surface) = map_window(&mut f);

    f.niri().layout.switch_workspace_down();
    f.niri_complete_animations();
    f.double_roundtrip(id);

    let data = request_frame_after_fallback_timer(&mut f, id, &surface);
    assert!(data.done.load(Ordering::Relaxed));
}

#[test]
fn no_fallback_frames_without_outputs() {
    let mut f = Fixture::new();
    let (id, surface) = map_window(&mut f);

    let data = request_frame_after_fallback_timer(&mut f, id, &surface);
    assert!(!data.done.load(Ordering::Relaxed));
}

#[test]
fn no_fallback_frames_with_monitors_off() {
    let mut f = Fixture::new();
    f.add_output(1, (1920, 1080));
    let (id, surface) = map_window(&mut f);

    f.niri().layout.switch_workspace_down();
    f.niri_complete_animations();
    f.double_roundtrip(id);

    let state = f.niri_state();
    state.niri.deactivate_monitors(&mut state.backend);

    let data = request_frame_after_fallback_timer(&mut f, id, &surface);
    assert!(!data.done.load(Ordering::Relaxed));
}
