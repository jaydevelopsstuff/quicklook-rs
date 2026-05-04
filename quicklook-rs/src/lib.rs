/*!
Interact with Apple's [QuickLookUI API](https://developer.apple.com/documentation/quicklookui)
in Rust without the tedious interop.

## Basic Usage
In reality most of these methods would likely be called in response to user inputs
in different part of the application cycle, but this gives a good picture of the API.
```rust
use quicklook_rs::{PreviewItem, QuickLookPanel, SourceFrame};

// ...
// On the main thread and after a running application has been established

let mut panel = QuickLookPanel::shared().unwrap();

// Assigning items
panel.set_items(vec![
    // Without a source frame (preview pane will have a fade in/out animation)
    PreviewItem::from_file_url("/test/example-text.txt", None).unwrap(),
    // With a source frame (preview pane will have zoom in/out animation based on the frame)
    PreviewItem::from_file_url("/test/example-img.jpeg", Some(SourceFrame {
        // Dummy values
        x: 64.,
        y: 64.,
        width: 64.,
        height: 64.,
    })).unwrap(),
    PreviewItem::from_url_string("https://google.com", None).unwrap(),
]);

// Displaying the panel
panel.show();

// Adding items on the fly (you could also use set_items)
panel.push_item(PreviewItem::from_file_url("/test/example-img2.jpeg", None).unwrap());

// Reloading to trigger changes taking effect if the panel is already open
panel.reload_if_dirty();

// Hiding the panel
panel.hide();
```
*/

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

/// The main representation for the QuickLook Preview Panel.
/// Only one instance per application should be created and used
/// on the **main thread only**.
///
/// # See Also
/// - [`QuickLookPanel::handle`] for accessing/modifying pane
/// state from other threads
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
    /// Creates a new instance of the QuickLookPanel which has shared access
    /// to the underlying panel. You should only ever really call this once
    /// for each application that is running.
    ///
    /// This will return [`None`] if called from a thread other than the main
    /// thread, or if the sharedPreviewPanel instance failed to be
    /// retrieved/created.
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

    /// Appends a new preview item after the last one.
    ///
    /// **IMPORTANT**: You must call [`QuickLookPanel::reload_if_dirty`] after for your changes
    /// to take visual effect.
    pub fn push_item(&self, item: PreviewItem) {
        let mut state = self.state.lock().unwrap();

        state.items.push(item);
        state.dirty = true;
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

    /// Requests that the preview panel reload its data
    /// from the data source, if it is marked dirty.
    ///
    /// This must be manually called on the main thread after
    /// any changes are made to the preview items.
    pub fn reload_if_dirty(&self) {
        let mut state = self.state.lock().unwrap();

        if state.dirty {
            unsafe {
                self.panel.reloadData();
            }
            state.dirty = false;
        }
    }

    /// Requests that the preview panel recompute the preview
    /// of the current preview item.
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
    /// Appends a new preview item after the last one.
    ///
    /// **IMPORTANT**: You must call [`QuickLookPanel::reload_if_dirty`] after on the main thread
    /// for your changes to take visual effect.
    pub fn push_item(&self, item: PreviewItem) {
        let mut state = self.state.lock().unwrap();

        state.items.push(item);
        state.dirty = true;
    }

    /// Assigns a new set of preview items to the Preview Panel.
    ///
    /// **IMPORTANT**: You must call [`QuickLookPanel::reload_if_dirty`] after on the main thread
    /// for your changes to take visual effect.
    pub fn set_items(&self, items: Vec<PreviewItem>) {
        let mut state = self.state.lock().unwrap();

        state.items = items;
        state.dirty = true;
    }
}

/// Stores the preview panel state.
#[derive(Default)]
pub struct PanelState {
    items: Vec<PreviewItem>,
    dirty: bool,
}

/// An item that can be shown in the preview pane.
///
/// If no `src_frame` is specified, the preview pane will use a
/// fade in/out animation rather than a zoom in/out animation.
#[derive(Clone)]
pub struct PreviewItem {
    source: Retained<NSURL>,
    src_frame: Option<SourceFrame>,
}

impl PreviewItem {
    pub fn new(source: Retained<NSURL>, src_frame: Option<SourceFrame>) -> Self {
        Self { source, src_frame }
    }

    /// Creates a new [`PreviewItem`] using the given file path and optional source frame.
    ///
    /// This will return [`None`] if the path is not valid unicode.
    pub fn from_file_url(path: impl AsRef<Path>, src_frame: Option<SourceFrame>) -> Option<Self> {
        Some(Self::new(
            NSURL::fileURLWithPath(&NSString::from_str(path.as_ref().to_str()?)),
            src_frame,
        ))
    }

    /// Creates a new [`PreviewItem`] using the given URL string and optional source frame.
    ///
    /// This will return [`None`] if the URL is malformed.
    pub fn from_url_string(
        url_string: impl AsRef<str>,
        src_frame: Option<SourceFrame>,
    ) -> Option<Self> {
        Some(Self::new(
            NSURL::URLWithString(&NSString::from_str(url_string.as_ref()))?,
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
#[derive(Debug, Clone, PartialEq)]
pub struct SourceFrame {
    /// The frame's x position, relative to the left of the screen.
    pub x: f64,
    /// The frame's y position, relative to the bottom of the screen.
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
