fn main() {
    println!("cargo::rerun-if-changed=linker-script.flash.ld");

    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("riscv64") {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/linker-script.flash.ld");
        println!("cargo::rustc-link-arg-bins=-T{script}");
        println!("cargo::rustc-link-arg-bins=--no-relax");
    }
}
