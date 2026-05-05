# quicklook

`quicklook` provides an easy to use Rust wrapper for Apple's 
[QuickLookUI API](https://developer.apple.com/documentation/quicklookui).

Unlike the [`objc2-quick-look-ui`](https://crates.io/crates/objc2-quick-look-ui) crate,
`quicklook` doesn't just provide bindings, but rather makes QuickLookUI accessible in
idiomatic Rust. This removes the need for manually writing the required delegates and other
interop between Rust and Apple's Objective-C APIs.

This is a work in progress and as such not everything may be thoroughly implemented—feel
free to open issues/prs.

## Basic Usage

In reality most of these methods would likely be called in response to user inputs
in different part of the application cycle, but this gives a good picture of the API.

```rust
use quicklook::{PreviewItem, QuickLookPanel, SourceFrame};

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

## Examples 

### Simple
For a simple example and testing purposes, you can try out `examples/simple_test` 
(found within the `quicklook` library folder). Simply modify the file urls in the 
`set_items` call to your liking and then execute `cargo run --example simple_test`
(ensure you're executing this within the `quicklook` library folder).

### Full Demo
For more robust demonstration and testing of `quicklook`, you can try out
`examples/egui_demo` (found within the repository's root). For more info check out
its [README](examples/egui_demo/README.md).

## Scope
For now, this is just a wrapper for Apple's QuickLookUI API, but I'm open to
implementing similar features for different platforms if they prove relevant.
Open a feature request issue if you're interested.
