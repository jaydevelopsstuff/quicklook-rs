use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;
use quicklook_rs::{PreviewItem, QuickLookPanel, SourceFrame};

fn main() {
    let mtm = MainThreadMarker::new().unwrap();

    let app = NSApplication::sharedApplication(mtm);

    let mut panel = QuickLookPanel::shared().unwrap();

    // Add some test file paths that exist on your file system
    panel.set_items(vec![
        PreviewItem::from_file_url("<path-to-your-file-1>", None).unwrap(),
        PreviewItem::from_file_url(
            "<path-to-your-file-2>",
            Some(SourceFrame {
                x: 64.,
                y: 64.,
                width: 64.,
                height: 64.,
            }),
        )
        .unwrap(),
        PreviewItem::from_url_string("https://google.com", None).unwrap(),
    ]);

    panel.show();

    app.run();
}
