use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;
use quicklook_rs::{PreviewItem, QuickLookPanel};

fn main() {
    let mtm = MainThreadMarker::new().unwrap();

    let app = NSApplication::sharedApplication(mtm);

    let mut panel = QuickLookPanel::shared().unwrap();

    panel.set_items(vec![
        PreviewItem::from_file_url("<path-to-your-file-1>", None).unwrap(),
        PreviewItem::from_file_url("<path-to-your-file-2>", None).unwrap(),
    ]);

    panel.show();

    app.run();
}
