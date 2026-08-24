fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("windows-msvc") {
        println!("cargo::rustc-link-arg-bins=/STACK:16777216");
    } else if target.contains("windows") {
        println!("cargo::rustc-link-arg-bins=-Wl,--stack,16777216");
    }
}
