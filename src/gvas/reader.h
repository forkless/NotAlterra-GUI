#pragma once
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <vector>

namespace notalterra::gvas {

/// Bounds-checked binary reader over a byte slice.
/// All reads return std::nullopt on overflow, never panic.
class Reader {
public:
    explicit Reader(std::span<const uint8_t> data) : data_(data) {}

    /// Read a little-endian u32 at offset, cast to size_t.
    static std::optional<size_t> read_u32(std::span<const uint8_t> data, size_t offset) {
        if (offset + 4 > data.size()) return std::nullopt;
        return static_cast<size_t>(
            (static_cast<uint32_t>(data[offset]) << 0) |
            (static_cast<uint32_t>(data[offset + 1]) << 8) |
            (static_cast<uint32_t>(data[offset + 2]) << 16) |
            (static_cast<uint32_t>(data[offset + 3]) << 24)
        );
    }

    /// Read a little-endian i32 at offset, cast to int64_t.
    static std::optional<int64_t> read_i32(std::span<const uint8_t> data, size_t offset) {
        if (offset + 4 > data.size()) return std::nullopt;
        return static_cast<int64_t>(static_cast<int32_t>(
            (static_cast<uint32_t>(data[offset]) << 0) |
            (static_cast<uint32_t>(data[offset + 1]) << 8) |
            (static_cast<uint32_t>(data[offset + 2]) << 16) |
            (static_cast<uint32_t>(data[offset + 3]) << 24)
        ));
    }

    /// Read a little-endian f64 at offset.
    static std::optional<double> read_f64(std::span<const uint8_t> data, size_t offset) {
        if (offset + 8 > data.size()) return std::nullopt;
        uint64_t bits =
            (static_cast<uint64_t>(data[offset]) << 0) |
            (static_cast<uint64_t>(data[offset + 1]) << 8) |
            (static_cast<uint64_t>(data[offset + 2]) << 16) |
            (static_cast<uint64_t>(data[offset + 3]) << 24) |
            (static_cast<uint64_t>(data[offset + 4]) << 32) |
            (static_cast<uint64_t>(data[offset + 5]) << 40) |
            (static_cast<uint64_t>(data[offset + 6]) << 48) |
            (static_cast<uint64_t>(data[offset + 7]) << 56);
        double result;
        memcpy(&result, &bits, sizeof(result));
        return result;
    }

    /// Read an FName: <u32 length><bytes><optional null>
    /// Returns (string, new_offset) or (nullopt, offset) on failure.
    static std::pair<std::optional<std::string>, size_t> read_fname(
        std::span<const uint8_t> data, size_t offset
    ) {
        auto len = read_u32(data, offset);
        if (!len) return {std::nullopt, offset};

        size_t off = offset + 4;
        if (*len == 0 || off + *len > data.size()) {
            return {std::nullopt, off};
        }

        size_t str_len = *len;
        if (data[off + str_len - 1] == 0) str_len--;

        std::string s(reinterpret_cast<const char*>(&data[off]), str_len);
        return {std::move(s), off + *len};
    }

    /// Read an FString: <i32 length> — negative=UTF16, positive=UTF8 (incl null).
    static std::pair<std::optional<std::string>, size_t> read_fstring(
        std::span<const uint8_t> data, size_t offset
    ) {
        auto raw_len = read_i32(data, offset);
        if (!raw_len) return {std::nullopt, offset};

        size_t off = offset + 4;
        if (*raw_len == 0) return {std::string(""), off};

        bool is_utf16 = *raw_len < 0;
        size_t bytes = is_utf16
            ? (static_cast<size_t>(-*raw_len) * 2)
            : static_cast<size_t>(*raw_len);

        if (off + bytes > data.size()) return {std::nullopt, off};

        std::string value;
        if (is_utf16) {
            // Decode UTF-16 LE
            size_t code_units = bytes / 2;
            for (size_t i = 0; i < code_units; i++) {
                uint16_t cp = static_cast<uint16_t>(data[off + i * 2]) |
                             (static_cast<uint16_t>(data[off + i * 2 + 1]) << 8);
                if (cp < 0x80) {
                    value += static_cast<char>(cp);
                } else if (cp < 0x800) {
                    value += static_cast<char>(0xC0 | (cp >> 6));
                    value += static_cast<char>(0x80 | (cp & 0x3F));
                } else {
                    value += static_cast<char>(0xE0 | (cp >> 12));
                    value += static_cast<char>(0x80 | ((cp >> 6) & 0x3F));
                    value += static_cast<char>(0x80 | (cp & 0x3F));
                }
            }
        } else {
            size_t str_len = bytes;
            if (str_len > 0 && data[off + str_len - 1] == 0) str_len--;
            value.assign(reinterpret_cast<const char*>(&data[off]), str_len);
        }

        return {std::move(value), off + bytes};
    }

private:
    std::span<const uint8_t> data_;
};

} // namespace notalterra::gvas
