use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    // Pretty rudimentary but can be made better

    let impl_type = if env::var("CARGO_CFG_UNIX").is_ok() || env::var("CARGO_CFG_WINDOWS").is_ok() {
        "std"
    } else if target.starts_with("thumbv") {
        "cortex_m"
    } else if target.starts_with("riscv32") {
        "riscv"
    } else {
        "none"
    };

    println!("cargo:rustc-cfg=impl_{}", impl_type);
}
