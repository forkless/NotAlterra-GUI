fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let rc_path = "resources/app.rc";
    let res_path = format!("{}/app.res", out_dir);

    // Try cross-compile windres first, then native windres
    let compiled = || -> bool {
        for windres in &["x86_64-w64-mingw32-windres", "windres"] {
            if std::process::Command::new(windres)
                .args(["-I", ".", rc_path, "-O", "coff", "-o", &res_path])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                return true;
            }
        }
        false
    };

    if compiled() {
        println!("cargo:rustc-link-arg={}", res_path);
    }
}
