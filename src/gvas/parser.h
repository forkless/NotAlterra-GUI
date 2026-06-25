#pragma once
#include "types.h"
#include <filesystem>
#include <span>
#include <string>

namespace notalterra::gvas {

/// Parse a .sav or .bak file and return all known GVAS metadata.
/// Returns metadata on success. On I/O error, throws std::system_error.
FullMetadata extract_full_metadata(std::filesystem::path const& file);

/// Parse GVAS metadata from an in-memory byte slice. Useful for fuzzing.
SaveMetadata extract_metadata_from_bytes(std::span<const uint8_t> data);

/// Parse a .sav or .bak file and return user-facing metadata.
/// Errors are non-fatal and collected in SaveMetadata::errors.
SaveMetadata extract_metadata(std::filesystem::path const& file);

} // namespace notalterra::gvas
