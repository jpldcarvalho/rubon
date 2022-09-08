
fn main() {
    prost_build::compile_protos(&["src/overlay/overlay.proto"], &["src"]).unwrap();
}
