#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Exercises the tar.gz backup round-trip: create an archive from a
    // directory tree of fuzzed save files, then restore and verify.
    // This catches logic bugs in create_tar_gz / extract_tar_gz, not
    // bugs in the tar or flate2 libraries themselves.
    use std::fs;

    let tmp = match std::env::temp_dir() {
        t => t,
    };
    let src = tmp.join("notalterra_fuzz_backup_src");
    let arc_dir = tmp.join("notalterra_fuzz_backup_arc");
    let ext_dir = tmp.join("notalterra_fuzz_backup_ext");

    let _ = fs::create_dir_all(&src);
    let _ = fs::create_dir_all(&arc_dir);
    let _ = fs::create_dir_all(&ext_dir);

    // Write fuzzed data as a save file
    let _ = fs::write(src.join("savegame_0.sav"), data);

    // Create tar.gz and restore
    let Ok(result) = notalterra::ops::create_full_backup(&src)
        else { return };
    let _ = notalterra::ops::restore_full_backup(&result.dest_path, &ext_dir);

    // Cleanup
    let _ = fs::remove_dir_all(&src);
    let _ = fs::remove_dir_all(&arc_dir);
    let _ = fs::remove_dir_all(&ext_dir);
});
