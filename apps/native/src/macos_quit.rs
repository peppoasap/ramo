//! Route this application's macOS Quit menu through egui's guarded window close.
//! Winit otherwise terminates the event loop for Cmd-Q without a cancellable
//! viewport close event. No application delegate or global OS setting is replaced.
use eframe::egui;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSApplication, NSMenu};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use std::sync::atomic::{AtomicBool, Ordering};
struct State {
    requested: AtomicBool,
    context: egui::Context,
}
define_class!(
    // SAFETY: NSObject has no additional subclassing requirements. This class
    // stays on the UI thread and has no custom Drop implementation.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = State]
    struct RamoNativeQuitTarget;
    // SAFETY: NSObjectProtocol imposes no additional requirements.
    unsafe impl NSObjectProtocol for RamoNativeQuitTarget {}
    impl RamoNativeQuitTarget {
        // SAFETY: Cocoa action signature: void, self, selector, nullable sender.
        #[unsafe(method(ramoRequestQuit:))]
        fn request_quit(&self,_sender:Option<&AnyObject>) {
            self.ivars().requested.store(true,Ordering::Release);
            self.ivars().context.request_repaint();
        }
    }
);
pub struct Bridge {
    target: Retained<RamoNativeQuitTarget>,
}
fn wire(menu: &NSMenu, target: &RamoNativeQuitTarget) -> bool {
    for item in menu.itemArray().iter() {
        if item.action() == Some(sel!(terminate:)) {
            // SAFETY: The retained target below implements this exact selector;
            // Cocoa keeps only a weak target reference, so Bridge owns it for the
            // complete App lifetime. Other app menu actions are untouched.
            unsafe {
                item.setTarget(Some(target));
                item.setAction(Some(sel!(ramoRequestQuit:)));
            }
            return true;
        }
        if let Some(submenu) = item.submenu()
            && wire(&submenu, target)
        {
            return true;
        }
    }
    false
}
impl Bridge {
    pub fn install(context: &egui::Context) -> Option<Self> {
        let mtm = MainThreadMarker::new()?;
        let menu = NSApplication::sharedApplication(mtm).mainMenu()?;
        // SAFETY: NSObject init on a freshly allocated class with initialized ivars.
        let allocated = RamoNativeQuitTarget::alloc(mtm).set_ivars(State {
            requested: AtomicBool::new(false),
            context: context.clone(),
        });
        let target: Retained<RamoNativeQuitTarget> = unsafe { msg_send![super(allocated), init] };
        wire(&menu, &target).then_some(Self { target })
    }
    pub fn take_request(&self) -> bool {
        self.target.ivars().requested.swap(false, Ordering::AcqRel)
    }
}
