#pragma once
#include <string>
#include <optional>
#include <vector>

namespace notalterra::gvas {

/// Full metadata extracted from a UE5 GVAS save file.
struct FullMetadata {
    std::optional<std::string> slot_name;
    std::optional<std::string> display_name;
    bool is_online = false;
    bool was_multiplayer = false;
    std::optional<std::string> game_mode;
    std::optional<std::string> level_name;
    std::optional<uint32_t> build_number;
    std::optional<std::string> build_branch;
    std::optional<uint32_t> saves_count;
    std::optional<uint32_t> latest_version;
    std::optional<uint32_t> data_version;
    std::optional<double> playtime_seconds;
};

/// User-facing metadata for the restore picker.
struct SaveMetadata {
    std::optional<std::string> slot_name;
    std::optional<std::string> display_name;
    bool is_online = false;
    std::optional<double> playtime_seconds;
    std::vector<std::string> errors;
};

/// Slot name derived from filename, e.g. "savegame_0"
inline std::optional<std::string> derive_slot_from_filename(std::string const& filename) {
    // Matches "savegame_N" prefix where N is digits
    // Covers: savegame_0.sav, savegame_0.bak, savegame_0_1.bak
    auto pos = filename.find("savegame_");
    if (pos == std::string::npos) return std::nullopt;

    auto rest = filename.substr(pos + 9); // after "savegame_"
    if (rest.empty() || !std::isdigit(rest[0])) return std::nullopt;

    std::string slot = "savegame_";
    for (char c : rest) {
        if (std::isdigit(c)) slot += c;
        else break;
    }
    return slot;
}

/// Check for corruption by comparing metadata vs filename convention.
/// Returns a reason string if corruption is suspected, nullopt otherwise.
inline std::optional<std::string> corruption_check(
    std::string const& filename,
    std::optional<std::string> const& slot_name
) {
    auto expected = derive_slot_from_filename(filename);
    if (!expected) return std::nullopt;
    if (slot_name && *slot_name != *expected) {
        return "Slot name mismatch: file says \"" + *slot_name
            + "\", expected \"" + *expected + "\"";
    }
    return std::nullopt;
}

} // namespace notalterra::gvas
