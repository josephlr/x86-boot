fn main() {
    println!("cargo:rerun-if-changed=examples/bootrom/x86_64-none.json");
    println!("cargo:rerun-if-changed=examples/bootrom/layout.ld");
}
