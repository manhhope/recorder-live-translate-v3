// screencapturekit nối thư viện Swift; test và binary của crate cần tìm runtime hệ thống (/usr/lib/swift, macOS 12+).
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    }
}
