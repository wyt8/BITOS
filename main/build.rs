fn main() {
    println!("cargo:rustc-link-arg=-T./main/src/riscv64.ld");
}