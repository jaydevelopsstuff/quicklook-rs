# quicklook-rs

`quicklook-rs` provides an easy to use Rust wrapper for Apple's 
[QuickLookUI API](https://developer.apple.com/documentation/quicklookui).


Unlike the [`objc2-quick-look-ui`](https://crates.io/crates/objc2-quick-look-ui) crate,
`quicklook-rs` doesn't just provide bindings, but rather makes QuickLookUI accessible in
idiomatic Rust. This removes the need for manually writing the required delegates and other
interop between Rust and Apple's Objective-C APIs.
