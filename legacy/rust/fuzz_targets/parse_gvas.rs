#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = notalterra::gvas::extract_metadata_from_bytes(data);
});
