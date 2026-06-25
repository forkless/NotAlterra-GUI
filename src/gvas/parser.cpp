#include "parser.h"
#include "reader.h"
#include <tl/expected.hpp>
#include <algorithm>
#include <cstring>
#include <fstream>
#include <vector>

namespace notalterra::gvas {
namespace {

// ── internal extraction helpers ─────────────────────────────────────────────

/// Scan for a StrProperty by name, return its FString value.
/// Walks the binary looking for FName header -> type "StrProperty" -> value.
static tl::expected<std::string, std::string> extract_str_property(
    std::span<const uint8_t> data, std::string const& prop_name
) {
    auto target = reinterpret_cast<const uint8_t*>(prop_name.data());
    size_t target_len = prop_name.size();
    size_t offset = 0;
    uint32_t attempts = 0;

    while (offset + 20 <= data.size() && attempts < 100) {
        // Scan for the property name bytes
        auto it = std::search(
            data.begin() + static_cast<ptrdiff_t>(offset), data.end(),
            target, target + target_len
        );
        if (it == data.end()) {
            return tl::make_unexpected(prop_name + " not found");
        }

        size_t found = static_cast<size_t>(std::distance(data.begin(), it));

        // Validate FName length field precedes the name
        if (found < 4) { offset = found + 1; attempts++; continue; }
        auto name_len = Reader::read_u32(data, found - 4);
        if (name_len != target_len + 1) { offset = found + 1; attempts++; continue; }

        // Validate null terminator after name
        if (found + target_len >= data.size() || data[found + target_len] != 0) {
            offset = found + 1; attempts++; continue;
        }

        // Read the type name (should be "StrProperty")
        auto after_name = found + target_len + 1;
        auto [type_name, type_off] = Reader::read_fname(data, after_name);
        if (!type_name || *type_name != "StrProperty") {
            offset = found + 1; attempts++; continue;
        }

        // Skip 9 bytes of property metadata, then read the FString value
        size_t meta_off = type_off + 9;
        if (meta_off + 4 > data.size()) {
            offset = found + 1; attempts++; continue;
        }

        auto [value, _] = Reader::read_fstring(data, meta_off);
        if (value && !value->empty() && value->size() < 100) {
            return std::move(*value);
        }

        offset = found + 1;
        attempts++;
    }

    return tl::make_unexpected("no valid " + prop_name + "/StrProperty pair found");
}

/// Find a BoolProperty by name, return true/false.
static std::optional<bool> extract_bool_property(
    std::span<const uint8_t> data, std::string const& prop_name
) {
    auto target = reinterpret_cast<const uint8_t*>(prop_name.data());
    size_t target_len = prop_name.size();
    size_t offset = 0;
    uint32_t attempts = 0;

    while (offset + 20 <= data.size() && attempts < 100) {
        auto it = std::search(
            data.begin() + static_cast<ptrdiff_t>(offset), data.end(),
            target, target + target_len
        );
        if (it == data.end()) return std::nullopt;

        size_t found = static_cast<size_t>(std::distance(data.begin(), it));
        if (found < 4) { offset = found + 1; attempts++; continue; }

        auto name_len = Reader::read_u32(data, found - 4);
        if (name_len != target_len + 1) { offset = found + 1; attempts++; continue; }
        if (found + target_len >= data.size() || data[found + target_len] != 0) {
            offset = found + 1; attempts++; continue;
        }

        auto after_name = found + target_len + 1;
        auto [type_name, type_off] = Reader::read_fname(data, after_name);
        if (!type_name || *type_name != "BoolProperty") {
            offset = found + 1; attempts++; continue;
        }

        size_t val_off = type_off + 9;
        if (val_off >= data.size()) { offset = found + 1; attempts++; continue; }

        return data[val_off] != 0;
    }
    return std::nullopt;
}

/// Scan for a double value near a marker byte sequence (heuristic).
static std::optional<double> scan_double_near(
    std::span<const uint8_t> data, std::span<const uint8_t> marker
) {
    auto it = std::search(data.begin(), data.end(), marker.begin(), marker.end());
    if (it == data.end()) return std::nullopt;

    size_t pos = static_cast<size_t>(std::distance(data.begin(), it));
    size_t end = (std::min)(pos + 60, data.size());

    for (size_t off = 8; off < 50 && pos + off + 8 <= end; off++) {
        auto val = Reader::read_f64(data, pos + off);
        if (val && *val > 60.0 && *val < 10'000'000.0) {
            return val;
        }
    }
    return std::nullopt;
}

/// Find a DoubleProperty by name.
static std::optional<double> extract_double_property(
    std::span<const uint8_t> data, std::string const& prop_name
) {
    auto target = reinterpret_cast<const uint8_t*>(prop_name.data());
    size_t target_len = prop_name.size();
    size_t offset = 0;
    uint32_t attempts = 0;

    while (offset + 30 <= data.size() && attempts < 100) {
        auto it = std::search(
            data.begin() + static_cast<ptrdiff_t>(offset), data.end(),
            target, target + target_len
        );
        if (it == data.end()) return std::nullopt;

        size_t found = static_cast<size_t>(std::distance(data.begin(), it));
        if (found < 4) { offset = found + 1; attempts++; continue; }

        auto name_len = Reader::read_u32(data, found - 4);
        if (!name_len || *name_len != target_len + 1) {
            offset = found + 1; attempts++; continue;
        }
        if (found + target_len >= data.size() || data[found + target_len] != 0) {
            offset = found + 1; attempts++; continue;
        }

        auto after_name = found + target_len + 1;
        auto [type_name, type_off] = Reader::read_fname(data, after_name);
        if (!type_name || *type_name != "DoubleProperty") {
            offset = found + 1; attempts++; continue;
        }

        size_t val_off = type_off + 9;
        if (val_off + 8 > data.size()) { offset = found + 1; attempts++; continue; }

        return Reader::read_f64(data, val_off);
    }
    return std::nullopt;
}

/// Find an IntProperty by name, return u32 value.
static std::optional<uint32_t> extract_int_property(
    std::span<const uint8_t> data, std::string const& prop_name
) {
    auto target = reinterpret_cast<const uint8_t*>(prop_name.data());
    size_t target_len = prop_name.size();
    size_t offset = 0;
    uint32_t attempts = 0;

    while (offset + 20 <= data.size() && attempts < 100) {
        auto it = std::search(
            data.begin() + static_cast<ptrdiff_t>(offset), data.end(),
            target, target + target_len
        );
        if (it == data.end()) return std::nullopt;

        size_t found = static_cast<size_t>(std::distance(data.begin(), it));
        if (found < 4) { offset = found + 1; attempts++; continue; }

        auto name_len = Reader::read_u32(data, found - 4);
        if (!name_len || *name_len != target_len + 1) {
            offset = found + 1; attempts++; continue;
        }
        if (found + target_len >= data.size() || data[found + target_len] != 0) {
            offset = found + 1; attempts++; continue;
        }

        auto after_name = found + target_len + 1;
        auto [type_name, type_off] = Reader::read_fname(data, after_name);
        if (!type_name || *type_name != "IntProperty") {
            offset = found + 1; attempts++; continue;
        }

        size_t val_off = type_off + 9;
        if (val_off + 4 > data.size()) { offset = found + 1; attempts++; continue; }

        auto val = Reader::read_u32(data, val_off);
        if (val) return static_cast<uint32_t>(*val);
        offset = found + 1; attempts++;
    }
    return std::nullopt;
}

/// Read entire file into a byte vector.
static std::vector<uint8_t> read_entire_file(std::filesystem::path const& path) {
    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if (!file) throw std::system_error(errno, std::generic_category(),
        "failed to open " + path.string());

    auto size = file.tellg();
    if (size < 0) throw std::system_error(errno, std::generic_category(),
        "failed to read " + path.string());

    file.seekg(0, std::ios::beg);
    std::vector<uint8_t> buf(static_cast<size_t>(size));
    if (!file.read(reinterpret_cast<char*>(buf.data()), static_cast<std::streamsize>(buf.size()))) {
        throw std::system_error(errno, std::generic_category(),
            "failed to read " + path.string());
    }
    return buf;
}

} // anonymous namespace

// ── public API ──────────────────────────────────────────────────────────────

FullMetadata extract_full_metadata(std::filesystem::path const& file) {
    auto data = read_entire_file(file);
    std::span span(data);

    auto slot = extract_str_property(span, "SlotName");
    auto display = extract_str_property(span, "DisplayName");
    auto game = extract_str_property(span, "GameMode");
    auto level = extract_str_property(span, "LevelName");
    auto branch = extract_str_property(span, "BuildBranch");

    return FullMetadata{
        .slot_name = slot.has_value() ? std::optional(std::move(*slot)) : std::nullopt,
        .display_name = display.has_value() ? std::optional(std::move(*display)) : std::nullopt,
        .is_online = extract_bool_property(span, "bIsMultiplayerSave").value_or(false),
        .was_multiplayer = extract_bool_property(span, "bWasMultiplayerSave").value_or(false),
        .game_mode = game.has_value() ? std::optional(std::move(*game)) : std::nullopt,
        .level_name = level.has_value() ? std::optional(std::move(*level)) : std::nullopt,
        .build_number = extract_int_property(span, "BuildNumber"),
        .build_branch = branch.has_value() ? std::optional(std::move(*branch)) : std::nullopt,
        .saves_count = extract_int_property(span, "SavesCount"),
        .latest_version = extract_int_property(span, "LatestVersion"),
        .data_version = extract_int_property(span, "DataVersion"),
        .playtime_seconds = scan_double_near(span, {reinterpret_cast<const uint8_t*>("Elapsed"), 7}),
    };
}

SaveMetadata extract_metadata_from_bytes(std::span<const uint8_t> data) {
    SaveMetadata result;

    // SlotName — non-fatal if missing
    auto slot = extract_str_property(data, "SlotName");
    if (slot.has_value()) result.slot_name = std::move(*slot);
    else result.errors.push_back("SlotName not found");

    // DisplayName — non-fatal if missing
    auto display = extract_str_property(data, "DisplayName");
    if (display.has_value()) result.display_name = std::move(*display);
    else result.errors.push_back("DisplayName not found");

    result.is_online = extract_bool_property(data, "OnlineMode").value_or(false);
    result.playtime_seconds = extract_double_property(data, "PlayTime");

    return result;
}

SaveMetadata extract_metadata(std::filesystem::path const& file) {
    auto data = read_entire_file(file);
    return extract_metadata_from_bytes(data);
}

} // namespace notalterra::gvas
