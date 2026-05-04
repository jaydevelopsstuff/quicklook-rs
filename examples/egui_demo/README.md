# egui demo

This is a very simple demo of how quicklook-rs could be used with images.

### Usage
- Run `cargo run`
- Select images using the file select dialog and then press the show preview pane button.
- Whilst keeping the preview pane open, open the file select dialog and choose different files. Observe how the preview pane updates while still open.

### Issues
- Currently images are set to a fixed size (150x150). I'd prefer to set a max height and then preserve aspect ratio but things got janky when I tried to measure the image sizes for the src_frame later. If someone more familiar with egui can resolve this, please open a pr!
