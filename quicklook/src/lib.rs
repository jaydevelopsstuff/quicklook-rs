/*!
Interact with Apple's [QuickLookUI API](https://developer.apple.com/documentation/quicklookui)
in Rust without the tedious interop.

## Basic Usage
In reality most of these methods would likely be called in response to user inputs
in different part of the application cycle, but this gives a good picture of the API.
```rust,no_run
use quicklook::{PreviewItem, QuickLookPanel, SourceFrame};

// ...
// On the main thread and after a running application has been established

let mut panel = QuickLookPanel::shared().unwrap();

// Assigning items
panel.set_items(vec![
    // Without a source frame (preview pane will have a fade in/out animation)
    PreviewItem::from_file_url("/test/example-text.txt", None).unwrap(),
    // With a source frame (preview pane will have zoom in/out animation based on the frame)
    PreviewItem::from_file_url("/test/example-img.jpeg", Some(SourceFrame::screen(64., 64., 64., 64.))).unwrap(),
    PreviewItem::from_url_string("https://google.com", None).unwrap(),
]);

// Displaying the panel
panel.show();

// Adding items on the fly / manually mutating the list of items
panel.with_items_mut(|items| {
    items.push(PreviewItem::from_file_url("/test/example-img2.jpeg", None).unwrap());
});

// Reloading to trigger changes taking effect if the panel is already open
panel.reload_if_dirty();

// Hiding the panel
panel.hide();
```
*/

use std::sync::{Arc, Mutex};

use objc2::{MainThreadMarker, rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::NSView;
use objc2_quick_look_ui::QLPreviewPanel;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use crate::interop::{
    qlpreviewpaneldatasource::QLPreviewPanelDataSource,
    qlpreviewpaneldelegate::QLPreviewPanelDelegate,
};

use std::path::Path;

use objc2_foundation::{NSInteger, NSString, NSURL};

mod interop;

/// The main representation and manager for the QuickLook Preview Panel.
/// Only one instance per application can be created and used on the
/// **main thread only**.
///
/// # See Also
/// - [`QuickLookPanel::handle`] for accessing/modifying pane state from
/// other threads
pub struct QuickLookPanel {
    panel: Retained<QLPreviewPanel>,
    state: Arc<Mutex<PanelState>>,
    _data_source: Retained<QLPreviewPanelDataSource>,
    _delegate: Retained<QLPreviewPanelDelegate>,
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
    /// thread, or if the `sharedPreviewPanel` instance failed to be
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
            _data_source: data_source,
            _delegate: delegate,
            state: panel_state,
        })
    }

    /// Assigns a new set of preview items to the Preview Panel.
    ///
    /// **IMPORTANT**: If you change the URLs or order of the items you MUST
    /// call [`QuickLookPanel::reload_if_dirty`] after for your changes to take
    /// visual effect. However, if you are only updating the source frames of
    /// pre-existing items you can safely avoid reloading.
    pub fn set_items(&self, items: Vec<PreviewItem>) {
        let mut state = self.state.lock().unwrap();

        state.items = items;
        state.dirty = true;
    }

    /// Provides mutable access to the current list of preview items through a closure.
    ///
    /// **IMPORTANT**: If you change the URLs or order of the items you MUST
    /// call [`QuickLookPanel::reload_if_dirty`] after for your changes to take
    /// visual effect. However, if you are only updating the source frames of
    /// pre-existing items you can safely avoid reloading.
    pub fn with_items_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Vec<PreviewItem>) -> R,
    {
        let mut state = self.state.lock().unwrap();

        let res = f(&mut state.items);
        state.dirty = true;
        res
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

    /// This method shows the pane if it is currently hidden, and vice versa
    /// if its already shown.
    pub fn toggle_visible(&self) {
        if self.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Moves the preview pane to the front of the screen list and focuses
    /// it.
    pub fn show(&self) {
        self.panel.makeKeyAndOrderFront(None);
    }

    /// Removes the window from the screen list, hiding it.
    pub fn hide(&self) {
        self.panel.orderOut(None);
    }

    /// Whether the preview pane is currently visible on the screen.
    pub fn is_visible(&self) -> bool {
        self.panel.isVisible()
    }

    /// Returns a clone of the preview item currently being
    /// viewed in the preview pane.
    ///
    /// May return [`None`] if items haven't been added to the pane yet,
    /// or if the preview pane and the data source have somehow lost sync
    /// (very unlikely).
    pub fn current_preview_item(&self) -> Option<PreviewItem> {
        let index = self.current_preview_item_index();
        if index < 0 {
            None
        } else {
            self.state
                .lock()
                .unwrap()
                .items
                .get(index as usize)
                .cloned()
        }
    }

    /// The index of the preview item currently being viewed
    pub fn current_preview_item_index(&self) -> NSInteger {
        unsafe { self.panel.currentPreviewItemIndex() }
    }

    /// Setter for the current preview item index
    pub fn set_current_preview_item_index(&self, index: NSInteger) {
        unsafe {
            self.panel.setCurrentPreviewItemIndex(index);
        }
    }

    /// Acquire a thread safe handle for updating preview pane data.
    ///
    /// Any main-thread only methods like refreshing, showing, hiding, etc.
    /// which hook into the AppKit APIs are not available through the handle.
    pub fn handle(&self) -> QuickLookHandle {
        QuickLookHandle {
            state: self.state.clone(),
        }
    }
}

impl QuickLookHandle {
    /// Assigns a new set of preview items to the Preview Panel.
    ///
    /// **IMPORTANT**: If you change the URLs or order of the items you MUST
    /// call [`QuickLookPanel::reload_if_dirty`] after on the main thread for your
    /// changes to take visual effect. However, if you are only updating the source
    /// frames of pre-existing items you can safely avoid reloading.
    pub fn set_items(&self, items: Vec<PreviewItem>) {
        let mut state = self.state.lock().unwrap();

        state.items = items;
        state.dirty = true;
    }

    /// Provides mutable access to the current list of preview items through a closure.
    ///
    /// **IMPORTANT**: If you change the URLs or order of the items you MUST
    /// call [`QuickLookPanel::reload_if_dirty`] after on the main thread for your
    /// changes to take visual effect. However, if you are only updating the source
    /// frames of pre-existing items you can safely avoid reloading.
    pub fn with_items_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Vec<PreviewItem>) -> R,
    {
        let mut state = self.state.lock().unwrap();

        let res = f(&mut state.items);
        state.dirty = true;
        res
    }
}

/// Stores the preview panel state.
#[derive(Default, Debug)]
struct PanelState {
    items: Vec<PreviewItem>,
    dirty: bool,
}

/// An item that can be shown in the preview pane.
///
/// If no `src_frame` is specified, the preview pane will use a
/// fade in/out animation rather than a zoom in/out animation when
/// it is opened and closed.
#[derive(Clone, Debug)]
pub struct PreviewItem {
    source: Retained<NSURL>,
    src_frame: Option<SourceFrame>,
}

impl PreviewItem {
    /// Raw constructor for a preview item. Primarily for use cases where you need
    /// full control over the creation of [`objc2_foundation::NSURL`].
    ///
    /// ## See also
    /// - [`PreviewItem::from_file_url`] and [`PreviewItem::from_url_string`] are preferable
    /// for most use cases
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

    /// Attempts to get the absolute url string from the stored NSURL this PreviewItem
    /// holds.
    pub fn absolute_url_string(&self) -> Option<String> {
        self.source.absoluteString().map(|s| s.to_string())
    }

    /// Returns a reference to this item's source frame, if it has one.
    pub fn source_frame(&self) -> Option<&SourceFrame> {
        self.src_frame.as_ref()
    }
}

/// Describes a source frame where a preview item originates from.
///
/// This is used when the preview pane is opened or close, where if a source frame is specified for the currently
/// viewed item the pane will animate the pane through scaling it in or out to make it appear as if the preview pane
/// is spawning from or "coming out of" the source frame.
#[derive(Debug, PartialEq, Clone)]
pub enum SourceFrame {
    /// A source frame with coordinates relative to the screen/monitor. This is intended for
    /// users with unusual use-cases or that prefer more fine-grained control over coordinates.
    ///
    /// ## Notes
    /// If your coordinates are for a frame within a window, you will need to manually recalculate and update
    /// the source frame coordinates everytime that screen is moved. For this reason most of the
    /// time you should probably should use [`SourceFrame::Window`] instead, which handles that for you.
    Screen(SourceFrameRect),
    /// A source frame with coordinates relative to a specified window. This is preferred over [`SourceFrame::Screen`]
    /// in most use cases.
    ///
    /// Behind the scenes this makes use of `NSWindow`'s
    /// [convertRectToScreen](https://developer.apple.com/documentation/appkit/nswindow/converttoscreen(_:)) method.
    ///
    /// ## Notes
    /// It is important to note that while this type of source frame guarantees the source frame coordinates will
    /// remain valid when the window is moved, it does NOT guarantee the source frame coordinates will remain valid
    /// if the window is resized, or in other cases like when a user scrolls within the window. See
    /// [`SourceFrameRect`]'s documentation for more details to help ensure source frame coordinates remain
    /// valid.
    Window(NSInteger, SourceFrameRect),
}

impl SourceFrame {
    /// Creates a [`SourceFrame::Screen`] based on the input coordinates and dimensions. `x` and `y`
    /// coordinates are relative to the bottom left corner of the screen.
    ///
    /// ## See Also
    /// - [`SourceFrame::Screen`] for more details
    pub fn screen(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::Screen(SourceFrameRect {
            x,
            y,
            width,
            height,
        })
    }

    /// Attempts to create a [`SourceFrame::Window`] based on the input coordinates, dimensions and provided window.
    /// `x` and `y` coordinates are relative to the bottom left corner of the window.
    ///
    /// This will return [`None`] if the underlying window handle cannot be accessed, is not a [`RawWindowHandle::AppKit`]
    /// handle, or if the `NSView` provided by the raw window handle does not return a parent window.
    /// The 2nd case will only happen if you're trying to use this crate on the wrong platform and the other cases
    /// are unlikely to occur with correct use.
    ///
    /// # See Also
    /// - [`SourceFrame::Window`] for more details
    pub fn window<W: HasWindowHandle>(
        window: &W,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Option<Self> {
        if let Ok(RawWindowHandle::AppKit(handle)) = window.window_handle().map(|h| h.as_raw()) {
            let raw_view_ptr = handle.ns_view.as_ptr() as *mut NSView;

            // SAFETY: The `raw_window_handle::HasWindowHandle` trait guarantees at execution time of this function
            // that `raw_view_ptr` will be a valid pointer to `NSView`.
            let view = unsafe { Retained::retain(raw_view_ptr)? };
            let window = view.window()?;

            Some(Self::Window(
                window.windowNumber(),
                SourceFrameRect {
                    x,
                    y,
                    width,
                    height,
                },
            ))
        } else {
            None
        }
    }

    /// Returns a reference to the [`SourceFrameRect`] for this source frame
    pub fn rect(&self) -> &SourceFrameRect {
        match self {
            Self::Screen(d) => d,
            Self::Window(_w, d) => d,
        }
    }
}

/// Describes a frame with coordinates relative either to the entire screen or a window
/// where the preview item originates from.
///
/// In AppKit/Cocoa, coordinates are relative to the bottom-left corner
/// of the window or screen, so you must take that into account when calculating
/// your frame's `y` position.
///
/// ## Notes
/// An important consideration is how often you must update the position of your preview items,
/// even if their content doesn't change. For example with a window relative frame, if a container of a preview item is
/// scrollable, that preview item's previously set source frame position may become invalid when the user scrolls.
/// Another example is when a window is resized and your `y` coordinate is relative to the top of
/// the window—because AppKit coordinates are relative to the bottom of the window, your old `y`
/// coordinate is now invalid. To avoid these issues you should ensure that you automatically recalculate
/// your preview item(s) source frame positions and call [`QuickLookPanel::set_items`] or [`QuickLookPanel::with_items_mut`]
/// (or the equivalent methods within [`QuickLookHandle`]) anytime a change occurs that might invalidate
/// items' source frame `x`/`y` coordinates. The same principal goes for `width` and `height`, but those
/// are less likely to become invalid in most use cases.
///
/// ## See Also
/// - [`SourceFrame`] for more details on the differences between creating a frame relative to the screen vs window
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SourceFrameRect {
    /// The frame's x position, relative to the left of the window or screen
    pub x: f64,
    /// The frame's y position, relative to the bottom of the window or screen
    pub y: f64,
    /// The frame's width
    pub width: f64,
    /// The frame's height
    pub height: f64,
}
