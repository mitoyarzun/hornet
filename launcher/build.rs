//! Embed the program icon in the Windows executable.

fn main() {
    println!("cargo:rerun-if-changed=../packaging/windows/hornet.ico");
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("../packaging/windows/hornet.ico");
        resource
            .compile()
            .expect("could not embed the Windows icon");
    }
}
