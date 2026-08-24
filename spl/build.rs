fn main() {
    println!("cargo::rerun-if-changed=linker-script.spl.ld");

    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("riscv64") {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/linker-script.spl.ld");
        println!("cargo::rustc-link-arg-bins=-T{script}");
    }
}
