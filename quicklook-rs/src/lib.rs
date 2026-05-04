use std::sync::{Arc, Mutex};

use objc2::{MainThreadMarker, rc::Retained, runtime::ProtocolObject};
use objc2_quick_look_ui::QLPreviewPanel;

use crate::raw::{
    qlpreviewpaneldatasource::QLPreviewPanelDataSource,
    qlpreviewpaneldelegate::QLPreviewPanelDelegate,
};

use std::path::Path;

use objc2_foundation::{NSString, NSURL};

mod raw;

pub struct QuickLookPanel {
    panel: Retained<QLPreviewPanel>,
    state: Arc<Mutex<PanelState>>,
    data_source: Retained<QLPreviewPanelDataSource>,
    delegate: Retained<QLPreviewPanelDelegate>,
}

/// A thread safe handle for sharing/modifying the quicklook preview panel state
/// from other threads.
///
/// ## Note
/// For any changes made using this handle to take effect in the Preview Pane
/// [`QuickLookPanel::reload_if_dirty`] must be called on the **main thread** after they are made.
#[derive(Clone)]
pub struct QuickLookHandle {
    state: Arc<Mutex<PanelState>>,
}

impl QuickLookPanel {
    pub fn shared() -> Option<Self> {
        let mtm = MainThreadMarker::new()?;

        let panel_state = Arc::new(Mutex::new(PanelState::default()));

        let panel: Retained<QLPreviewPanel> =
            unsafe { QLPreviewPanel::sharedPreviewPanel(mtm.clone())? };

        let data_source = QLPreviewPanelDataSource::new(panel_state.clone());
        let delegate = QLPreviewPanelDelegate::new(mtm, panel_state.clone());

        unsafe {
            panel.setDataSource(Some(ProtocolObject::from_ref(&*data_source)));
            panel.setDelegate(Some(&delegate));
        }

        Some(Self {
            panel,
            data_source,
            delegate,
            state: panel_state,
        })
    }

    /// Assigns a new set of preview items to the Preview Panel.
    ///
    /// **IMPORTANT**: You must call [`QuickLookPanel::reload_if_dirty`] for your changes
    /// to take visual effect.
    pub fn set_items(&mut self, items: Vec<PreviewItem>) {
        let mut state = self.state.lock().unwrap();

        state.items = items;
        state.dirty = true;
    }

    pub fn reload_if_dirty(&self) {
        let mut state = self.state.lock().unwrap();

        if state.dirty {
            unsafe {
                self.panel.reloadData();
            }
            state.dirty = false;
        }
    }

    pub fn refresh_current_preview_item(&self) {
        unsafe {
            self.panel.refreshCurrentPreviewItem();
        }
    }

    pub fn show(&self) {
        self.panel.makeKeyAndOrderFront(None);
    }

    pub fn hide(&self) {
        self.panel.orderOut(None);
    }

    pub fn handle(&self) -> QuickLookHandle {
        QuickLookHandle {
            state: self.state.clone(),
        }
    }
}

impl QuickLookHandle {
    /// Assigns a new set of preview items to the Preview Panel.
    ///
    /// **IMPORTANT**: You must call [`QuickLookPanel::reload_if_dirty`] after on the main thread
    /// for your changes to take visual effect.
    pub fn set_items(&mut self, items: Vec<PreviewItem>) {
        let mut state = self.state.lock().unwrap();

        state.items = items;
        state.dirty = true;
    }
}

#[derive(Default)]
pub struct PanelState {
    items: Vec<PreviewItem>,
    dirty: bool,
}

pub struct PreviewItem {
    source: Retained<NSURL>,
    src_frame: Option<SourceFrame>,
}

impl PreviewItem {
    pub fn new(source: Retained<NSURL>, src_frame: Option<SourceFrame>) -> Self {
        Self { source, src_frame }
    }

    pub fn from_file_url(path: impl AsRef<Path>, src_frame: Option<SourceFrame>) -> Option<Self> {
        Some(Self::new(
            NSURL::fileURLWithPath(&NSString::from_str(path.as_ref().to_str()?)),
            src_frame,
        ))
    }
}

/// Describes a frame on the screen where a preview item originates from.
///
/// In AppKit/Cocoa, coordinates are relative to the bottom-left corner
/// of the screen, so you must take that into account when calculating
/// your frame's `y` position.
///
/// # See Also
/// - [convertRectToScreen](https://docs.rs/objc2-app-kit/latest/objc2_app_kit/struct.NSWindow.html#method.convertRectToScreen) ([Apple Docs](https://developer.apple.com/documentation/appkit/nswindow/converttoscreen(_:))) If you're already using AppKit APIs.
#[derive(Clone, PartialEq)]
pub struct SourceFrame {
    pub x: f64,
    /// The frame's y position, relative to the bottom of the screen.
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
