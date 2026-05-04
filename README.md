# quicklook-rs

`quicklook-rs` provides an easy to use Rust wrapper for Apple's 
[QuickLookUI API](https://developer.apple.com/documentation/quicklookui).
This is a work in progress and as such not everything may be implemented—feel
free to open an issue or pr if you'd like!

Unlike the [`objc2-quick-look-ui`](https://crates.io/crates/objc2-quick-look-ui) crate,
`quicklook-rs` doesn't just provide bindings, but rather makes QuickLookUI accessible in
idiomatic Rust. This removes the need for manually writing the required delegates and other
interop between Rust and Apple's Objective-C APIs.

## Examples 

### Simple
For a simple example and testing purposes, you can try out `examples/simple_test` 
(found within the `quicklook-rs` library folder). Simply modify the file urls in the 
`set_items` call to your liking and then execute `cargo run --example simple_test`
(ensure your executing this within the `quicklook-rs` library folder).

### Full Demo
For more robust demonstration and testing of `quicklook-rs`, you can try out
`examples/egui_demo` (found within the repository's root). For more info check out
the [README](examples/egui_demo/README.md).
