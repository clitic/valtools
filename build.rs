fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rerun-if-changed=icons/logo.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/logo.ico");
        res.compile().expect("Failed to embed icon resource");
    }
}
