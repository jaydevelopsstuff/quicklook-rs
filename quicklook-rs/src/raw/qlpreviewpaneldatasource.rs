use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::{AnyThread, DeclaredClass, define_class, msg_send};
use objc2_app_kit::NSPanel;
use objc2_foundation::NSURL;
use objc2_foundation::{NSObject, NSObjectProtocol};

use objc2_quick_look_ui::QLPreviewPanelDataSource as QLPreviewPanelDataSourceProtocol;

use crate::PanelState;

#[derive(Clone)]
pub struct Ivars {
    state: Arc<Mutex<PanelState>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = Ivars]
    pub struct QLPreviewPanelDataSource;

    unsafe impl QLPreviewPanelDataSourceProtocol for QLPreviewPanelDataSource {
        #[unsafe(method(numberOfPreviewItemsInPreviewPanel:))]
        fn instance_number_of_preview_items_in_preview_panel(&self, _panel: &NSPanel) -> usize {
            self.ivars().state.lock().unwrap().items.len()
        }

        #[unsafe(method_id(previewPanel:previewItemAtIndex:))]
        fn preview_item_at_index(&self, _panel: &NSPanel, index: usize) -> Retained<NSURL> {
            self.ivars().state.lock().unwrap().items[index]
                .source
                .clone()
        }
    }

    unsafe impl NSObjectProtocol for QLPreviewPanelDataSource {}
);

impl QLPreviewPanelDataSource {
    pub fn new(state: Arc<Mutex<PanelState>>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(Ivars { state });

        unsafe { msg_send![super(this), init] }
    }
}
