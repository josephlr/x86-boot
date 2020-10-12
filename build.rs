fn main() {
    println!("cargo:rerun-if-changed=examples/bootrom.json");
    println!("cargo:rerun-if-changed=examples/bootrom.ld");
}
