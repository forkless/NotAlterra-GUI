#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Exercises extract_full_metadata — covers IntProperty, DoubleProperty,
    // additional StrProperty (GameMode, LevelName, BuildBranch) and
    // BoolProperty (bIsMultiplayerSave, bWasMultiplayerSave) code paths
    // not reached by the existing parse_gvas target.
    let tmp = std::env::temp_dir();
    let path = tmp.join("notalterra_fuzz_full_meta.sav");
    let _ = std::fs::write(&path, data);
    let _ = notalterra::gvas::extract_full_metadata(&path);
    let _ = std::fs::remove_file(&path);
});
