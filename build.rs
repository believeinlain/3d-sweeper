#[cfg(windows)]
use winres::WindowsResource;

fn main() {
    #[cfg(windows)]
    WindowsResource::new()
        .set_icon("embed/icon_01.ico")
        .compile()
        .unwrap();
}
