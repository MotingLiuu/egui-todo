use objc2::MainThreadMarker;
use objc2_app_kit::{NSView, NSWindowButton, NSWindowTitleVisibility};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// Hide native chrome without changing the window's full-size content style.
pub fn set_minimal_mode(frame: &eframe::Frame, minimal: bool) {
    let Some(_main_thread) = MainThreadMarker::new() else {
        eprintln!("Cannot update macOS window chrome outside the main thread");
        return;
    };
    let Ok(handle) = frame.window_handle() else {
        eprintln!("Cannot access the native window handle");
        return;
    };
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return;
    };

    // SAFETY: eframe supplies a live AppKit NSView. The borrowed window handle
    // stays alive for this call, which runs on the main thread; no pointer is stored.
    let view = unsafe { &*handle.ns_view.as_ptr().cast::<NSView>() };
    let Some(window) = view.window() else {
        eprintln!("The AppKit view is not attached to a window");
        return;
    };

    for button in [
        NSWindowButton::CloseButton,
        NSWindowButton::MiniaturizeButton,
        NSWindowButton::ZoomButton,
    ] {
        if let Some(button) = window.standardWindowButton(button) {
            button.setHidden(minimal);
        }
    }
    window.setTitleVisibility(if minimal {
        NSWindowTitleVisibility::Hidden
    } else {
        NSWindowTitleVisibility::Visible
    });
    window.setTitlebarAppearsTransparent(true);
}
