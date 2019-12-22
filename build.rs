//extern crate embed_resource;

fn main() {
//    embed_resource::compile("./src/version.rc");
    println!("cargo:rustc-link-search=native={}", "C:/ProjectsSplit/Make/G Code Insert Pause/tmp" /*out_dir*/);
    println!("cargo:rustc-link-lib=dylib={}", "version" /*prefix*/);
}
