fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set("FileVersion", "1.0.0");
        res.set("ProductName", "opencode-proxy");
        res.set("FileDescription", "OpenCode Zen API proxy with dashboard");
        res.compile().unwrap();
    }
}
