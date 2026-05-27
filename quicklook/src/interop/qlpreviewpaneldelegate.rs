use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::{AnyThread, DeclaredClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSApplication, NSEvent, NSImage, NSPanel, NSWindowDelegate};
use objc2_foundation::{NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize};
use objc2_foundation::{NSURL, NSZeroRect};

use objc2_quick_look_ui::{
    QLPreviewPanel, QLPreviewPanelDelegate as QLPreviewPanelDelegateProtocol,
};

use crate::{PanelState, SourceFrame};

#[derive(Clone)]
pub struct Ivars {
    state: Arc<Mutex<PanelState>>,
}

define_class!(
    #[thread_kind = MainThreadOnly]
    #[unsafe(super(NSObject))]
    #[ivars = Ivars]
    pub struct QLPreviewPanelDelegate;

    unsafe impl QLPreviewPanelDelegateProtocol for QLPreviewPanelDelegate {
        #[unsafe(method(previewPanel:handleEvent:))]
        fn handle_event(&self, _panel: &NSPanel, _event: &NSEvent) -> bool {
            false
        }

        #[unsafe(method(previewPanel:sourceFrameOnScreenForPreviewItem:))]
        fn source_frame_on_screen_for_preview_item(
            &self,
            panel: Option<&QLPreviewPanel>,
            item: Option<&NSURL>,
        ) -> NSRect {
            if let (Some(item), Some(_panel)) = (item, panel) {
                let state = self.ivars().state.lock().unwrap();

                let matching_item = state
                    .items
                    .iter()
                    .find(|src_item| *src_item.source == *item);

                match matching_item.and_then(|i| i.src_frame.as_ref()) {
                    Some(SourceFrame::Screen(frame)) => NSRect::new(
                        NSPoint::new(frame.x, frame.y),
                        NSSize::new(frame.width, frame.height),
                    ),
                    Some(SourceFrame::Window(window_number, frame)) => {
                        if let Some(window) = NSApplication::sharedApplication(unsafe {
                            MainThreadMarker::new_unchecked()
                        })
                        .windowWithWindowNumber(*window_number)
                        {
                            window.convertRectToScreen(NSRect::new(
                                NSPoint::new(frame.x, frame.y),
                                NSSize::new(frame.width, frame.height),
                            ))
                        } else {
                            unsafe { NSZeroRect }
                        }
                    }
                    None => unsafe { NSZeroRect },
                }
            } else {
                unsafe { NSZeroRect }
            }
        }

        #[unsafe(method_id(previewPanel:transitionImageForPreviewItem:contentRect:))]
        fn transition_image_for_preview_item(
            &self,
            _panel: Option<&QLPreviewPanel>,
            item: Option<&NSURL>,
            content_rect: *mut NSRect,
        ) -> Option<Retained<NSImage>> {
            item.and_then(|item| NSImage::initWithContentsOfURL(NSImage::alloc(), item))
        }
    }

    unsafe impl NSObjectProtocol for QLPreviewPanelDelegate {}
    unsafe impl NSWindowDelegate for QLPreviewPanelDelegate {}
);

impl QLPreviewPanelDelegate {
    pub fn new(mtm: MainThreadMarker, state: Arc<Mutex<PanelState>>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(Ivars { state });

        unsafe { msg_send![super(this), init] }
    }
}
