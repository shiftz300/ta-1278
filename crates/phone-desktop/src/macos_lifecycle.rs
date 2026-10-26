//! Route AppKit Quit (menu, Cmd-Q and Dock) through the same fallible save
//! path as closing the window. Winit's default terminate: bypasses key input.
use objc2::{ClassType, DeclaredClass, declare_class, msg_send_id, mutability, rc::Retained};
use objc2_app_kit::NSApplication;
use objc2_foundation::{MainThreadMarker, NSObjectProtocol};
use std::sync::atomic::{AtomicBool, Ordering};

static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

declare_class!(
    struct PhoneApplication;

    // SAFETY: NSApplication is main-thread-only. We add no ivars and only
    // override its documented terminate: action with the matching signature.
    unsafe impl ClassType for PhoneApplication {
        type Super = NSApplication;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "TA1278Application";
    }
    impl DeclaredClass for PhoneApplication {}

    unsafe impl PhoneApplication {
        #[method(terminate:)]
        fn request_quit(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            // Do not call the save path inside AppKit's event dispatch. The
            // regular winit loop consumes this once, and may refuse to exit.
            QUIT_REQUESTED.store(true, Ordering::Relaxed);
        }
    }
);

pub fn initialize() -> Result<(), &'static str> {
    let _main = MainThreadMarker::new().ok_or("AppKit must start on the main thread")?;
    // SAFETY: First sharedApplication call, on the main thread. Winit supports
    // a custom NSApplication and retains control of its own delegate/windows.
    let app: Retained<NSApplication> =
        unsafe { msg_send_id![PhoneApplication::class(), sharedApplication] };
    if !app.is_kind_of::<PhoneApplication>() {
        return Err("AppKit was initialized before the safe quit handler");
    }
    Ok(())
}

pub fn take_quit_request() -> bool {
    QUIT_REQUESTED.swap(false, Ordering::Relaxed)
}
