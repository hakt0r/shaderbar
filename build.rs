fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=src/hello.c");
    // check if target/patches/glium-0.34.0 is present
    let patched_exists = std::path::Path::new("target/patches/glium-0.34.0").exists();
    if patched_exists {
        println!("cargo:rustc-link-search=target/patches/glium-0.34.0");
    } else {
        // run patch-package
        let output = std::process::Command::new("cargo")
            .arg("patch-crate")
            .output()
            .expect("failed to execute process");
        println!("patch-crate output: {:?}", output);
    }
}
