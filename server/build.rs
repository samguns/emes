use cmake::Config;

fn main() {
    let dst = Config::new("./qwen2_vl_cpp").profile("Release").build();
    println!("cargo:rustc-link-arg=-L{}/lib", dst.display());
}
