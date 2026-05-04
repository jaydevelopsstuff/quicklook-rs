use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::{AnyThread, DeclaredClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSEvent, NSImage, NSPanel, NSWindowDelegate};
use objc2_foundation::{NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize};
use objc2_foundation::{NSURL, NSZeroRect};

use objc2_quick_look_ui::QLPreviewPanelDelegate as QLPreviewPanelDelegateProtocol;

use crate::PanelState;

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
            _panel: Option<&NSPanel>,
            item: Option<&NSURL>,
        ) -> NSRect {
            if let Some(item) = item {
                let state = self.ivars().state.lock().unwrap();

                let matching_item = state
                    .items
                    .iter()
                    .find(|src_item| *src_item.source == *item);

                if let Some(src_frame) = matching_item.and_then(|i| i.src_frame.as_ref()) {
                    NSRect::new(
                        NSPoint::new(src_frame.x, src_frame.y),
                        NSSize::new(src_frame.width, src_frame.height),
                    )
                } else {
                    unsafe { NSZeroRect }
                }
            } else {
                unsafe { NSZeroRect }
            }
        }

        #[unsafe(method_id(previewPanel:transitionImageForPreviewItem:contentRect:))]
        fn transition_image_for_preview_item(
            &self,
            _panel: Option<&NSPanel>,
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
