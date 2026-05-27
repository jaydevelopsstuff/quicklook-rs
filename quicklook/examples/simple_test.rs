use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;
use quicklook::{PreviewItem, QuickLookPanel, SourceFrame};

fn main() {
    let mtm = MainThreadMarker::new().unwrap();

    let app = NSApplication::sharedApplication(mtm);

    let panel = QuickLookPanel::shared().unwrap();

    // Add some test file paths that exist on your file system
    panel.set_items(vec![
        PreviewItem::from_file_url("<path-to-your-file-1>", None).unwrap(),
        PreviewItem::from_file_url(
            "<path-to-your-file-2>",
            Some(SourceFrame::screen(64., 64., 64., 64.)),
        )
        .unwrap(),
        PreviewItem::from_url_string("https://google.com", None).unwrap(),
    ]);

    panel.with_items_mut(|items| {
        items.push(PreviewItem::from_file_url("/test/example-img2.jpeg", None).unwrap());
    });

    panel.show();

    app.run();
}
