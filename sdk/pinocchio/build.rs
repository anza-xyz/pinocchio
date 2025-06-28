fn main() {
    // Minimum compiler version required for Box::new_uninit_slice / assume_init path.
    const NEEDS_VERSION: &str = "1.82.0";

    // Parse current rustc version.
    let version_meta = rustc_version::version().expect("rustc version unavailable");
    let needs = rustc_version::Version::parse(NEEDS_VERSION).unwrap();

    if version_meta >= needs {
        println!("cargo:rustc-cfg=has_box_new_uninit_slice");
        // Inform the `unexpected_cfgs` lint that this cfg is intentional.
        println!("cargo:rustc-check-cfg=cfg(has_box_new_uninit_slice)");
    }
}
