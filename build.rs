fn main() {
    // Embed icon on Windows (via windres for cross-compilation support)
    #[cfg(target_os = "windows")]
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let rc_path = "resources/app.rc";
        let res_path = format!("{}/app.res", out_dir);
        let status = std::process::Command::new("x86_64-w64-mingw32-windres")
            .args([rc_path, "-O", "coff", "-o", &res_path])
            .status();
        if let Ok(s) = status {
            if s.success() {
                println!("cargo:rustc-link-arg={}", res_path);
                return;
            }
        }
        // Fallback: try plain windres
        let status = std::process::Command::new("windres")
            .args([rc_path, "-O", "coff", "-o", &res_path])
            .status();
        if let Ok(s) = status {
            if s.success() {
                println!("cargo:rustc-link-arg={}", res_path);
            }
        }
    }
}
