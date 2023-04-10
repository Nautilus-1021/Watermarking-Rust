fn main() {
    println!("cargo:rerun-if-changed=src/ressources/watermarking.gresource.xml");

    glib_build_tools::compile_resources(
        &["src/ressources"],
        "src/ressources/watermarking.gresource.xml",
        "watermarking.gresource",
    );
}
