//! Apple Pencil's own gestures on iPad (docs/ui-design.md, "Phones,
//! tablets and pens"): the double-tap (2nd generation, Pro, USB-C) and the
//! squeeze (Pro). The webview sees the Pencil's strokes, hover, pressure
//! and tilt as pointer events, but not these, so a `UIPencilInteraction`
//! on the webview hears them and sends them to the editor as a `pencil`
//! event, with the action the user picked for it in Settings → Apple
//! Pencil. The editor does that action (src/lib/pen.ts).

use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol, ProtocolObject};
use objc2::{class, define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_ui_kit::{UIPencilInteraction, UIPencilInteractionDelegate, UIPencilInteractionPhase, UIPencilInteractionSqueeze, UIPencilPreferredAction, UIView};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

/// What the editor is sent: which gesture, and the action chosen in Settings.
#[derive(Clone, Serialize)]
struct PencilGesture {
    gesture: &'static str,
    action: &'static str,
}

fn action_name(action: UIPencilPreferredAction) -> &'static str {
    match action {
        UIPencilPreferredAction::SwitchEraser => "switchEraser",
        UIPencilPreferredAction::SwitchPrevious => "switchPrevious",
        UIPencilPreferredAction::ShowColorPalette => "showColorPalette",
        UIPencilPreferredAction::ShowInkAttributes => "showInkAttributes",
        UIPencilPreferredAction::ShowContextualPalette => "showContextualPalette",
        // Ignore, and the system shortcut, which iPadOS runs itself.
        _ => "ignore",
    }
}

struct Ivars {
    app: AppHandle,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements, and the delegate
    // has no Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "StylusPencilDelegate"]
    #[ivars = Ivars]
    struct PencilDelegate;

    unsafe impl NSObjectProtocol for PencilDelegate {}

    unsafe impl UIPencilInteractionDelegate for PencilDelegate {
        // The tap method every iPadOS version calls (the newer
        // `pencilInteraction:didReceiveTap:` adds only the hover pose).
        #[unsafe(method(pencilInteractionDidTap:))]
        fn did_tap(&self, _interaction: &UIPencilInteraction) {
            self.send("tap", UIPencilInteraction::preferredTapAction(self.mtm()));
        }

        // Apple Pencil Pro, iPadOS 17.5 and later. A squeeze has phases;
        // it counts once, when it ends.
        #[unsafe(method(pencilInteraction:didReceiveSqueeze:))]
        fn did_squeeze(&self, _interaction: &UIPencilInteraction, squeeze: &UIPencilInteractionSqueeze) {
            if squeeze.phase() == UIPencilInteractionPhase::Ended {
                self.send("squeeze", UIPencilInteraction::preferredSqueezeAction(self.mtm()));
            }
        }
    }
);

impl PencilDelegate {
    fn new(mtm: MainThreadMarker, app: AppHandle) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(Ivars { app });
        // SAFETY: NSObject's plain init.
        unsafe { msg_send![super(this), init] }
    }

    fn send(&self, gesture: &'static str, action: UIPencilPreferredAction) {
        let event = PencilGesture { gesture, action: action_name(action) };
        if let Err(e) = self.ivars().app.emit("pencil", event) {
            log::warn!("couldn't send the Pencil's {gesture}: {e}");
        }
    }
}

/// Listens for the Pencil's gestures on the window's webview.
pub fn attach(window: &WebviewWindow) {
    let app = window.app_handle().clone();
    let attached = window.with_webview(move |webview| {
        // with_webview runs this on the main thread.
        let Some(mtm) = MainThreadMarker::new() else { return };
        // SAFETY: on iOS, `inner` is the WKWebView, a UIView, alive while
        // the window is.
        let view: &UIView = unsafe { &*(webview.inner() as *const UIView) };
        let delegate = PencilDelegate::new(mtm, app);
        let interaction = UIPencilInteraction::initWithDelegate(UIPencilInteraction::alloc(mtm), ProtocolObject::from_ref(&*delegate));
        view.addInteraction(ProtocolObject::from_ref(&*interaction));
        // The interaction holds its delegate weakly, and the app has one
        // window for its whole life, so the delegate is kept for good.
        std::mem::forget(delegate);
    });
    if let Err(e) = attached {
        log::warn!("couldn't listen for the Pencil's double-tap: {e}");
    }
}

/// Whether Settings → Apple Pencil → "Only Draw with Apple Pencil" is on
/// (iPadOS 17.5 and later; `None` before that), so fingers never draw.
#[tauri::command]
pub fn pencil_only_drawing() -> Option<bool> {
    // A command without `async` runs on the main thread.
    let mtm = MainThreadMarker::new()?;
    // SAFETY: respondsToSelector: on a class object.
    let known: bool = unsafe { msg_send![class!(UIPencilInteraction), respondsToSelector: sel!(prefersPencilOnlyDrawing)] };
    known.then(|| UIPencilInteraction::prefersPencilOnlyDrawing(mtm))
}
