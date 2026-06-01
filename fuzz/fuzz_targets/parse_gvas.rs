#[fuzz]
mod target {
    #[fuzz]
    fn parse_gvas(data: &[u8]) {
        let _ = notalterra::gvas::extract_metadata_from_bytes(data);
    }
}
