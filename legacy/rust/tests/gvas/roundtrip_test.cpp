#include <gtest/gtest.h>
#include <gvas/parser.h>
#include <gvas/types.h>
#include <cstring>
#include <vector>

using namespace notalterra::gvas;

// ── derive_slot_from_filename ───────────────────────────────────────────────

TEST(SlotNameTest, StandardSav) {
    auto slot = derive_slot_from_filename("savegame_0.sav");
    ASSERT_TRUE(slot.has_value());
    EXPECT_EQ(*slot, "savegame_0");
}

TEST(SlotNameTest, StandardBak) {
    auto slot = derive_slot_from_filename("savegame_1.bak");
    ASSERT_TRUE(slot.has_value());
    EXPECT_EQ(*slot, "savegame_1");
}

TEST(SlotNameTest, BackupIndexBak) {
    auto slot = derive_slot_from_filename("savegame_0_3.bak");
    ASSERT_TRUE(slot.has_value());
    EXPECT_EQ(*slot, "savegame_0");
}

TEST(SlotNameTest, RandomFile) {
    auto slot = derive_slot_from_filename("notasave.txt");
    EXPECT_FALSE(slot.has_value());
}

TEST(SlotNameTest, NoExtension) {
    auto slot = derive_slot_from_filename("savegame_12");
    ASSERT_TRUE(slot.has_value());
    EXPECT_EQ(*slot, "savegame_12");
}

// ── corruption_check ────────────────────────────────────────────────────────

TEST(CorruptionCheckTest, Match) {
    auto result = corruption_check("savegame_0.sav", std::optional<std::string>("savegame_0"));
    EXPECT_FALSE(result.has_value());
}

TEST(CorruptionCheckTest, Mismatch) {
    auto result = corruption_check("savegame_0.sav", std::optional<std::string>("savegame_3"));
    ASSERT_TRUE(result.has_value());
    EXPECT_TRUE(result->find("mismatch") != std::string::npos ||
                result->find("Mismatch") != std::string::npos ||
                result->find("mismatch") != std::string::npos);
}

TEST(CorruptionCheckTest, NoSlotFromFilename) {
    auto result = corruption_check("random.txt", std::optional<std::string>("savegame_0"));
    EXPECT_FALSE(result.has_value());
}

TEST(CorruptionCheckTest, NulloptSlotName) {
    auto result = corruption_check("savegame_0.sav", std::nullopt);
    EXPECT_FALSE(result.has_value());
}

// ── extract_metadata_from_bytes (synthetic GVAS-like data) ──────────────────

/// Build a minimal GVAS fragment containing a StrProperty.
/// This is a synthetic FName-encoded property for testing the scanner.
static std::vector<uint8_t> make_str_property(
    std::string const& name,
    std::string const& type,
    std::string const& value
) {
    std::vector<uint8_t> buf;

    // FName: name (u32 length + bytes + null)
    uint32_t name_len = static_cast<uint32_t>(name.size() + 1); // +1 for null
    buf.push_back(static_cast<uint8_t>(name_len >> 0));
    buf.push_back(static_cast<uint8_t>(name_len >> 8));
    buf.push_back(static_cast<uint8_t>(name_len >> 16));
    buf.push_back(static_cast<uint8_t>(name_len >> 24));
    buf.insert(buf.end(), name.begin(), name.end());
    buf.push_back(0); // null terminator

    // FName: type
    uint32_t type_len = static_cast<uint32_t>(type.size() + 1);
    buf.push_back(static_cast<uint8_t>(type_len >> 0));
    buf.push_back(static_cast<uint8_t>(type_len >> 8));
    buf.push_back(static_cast<uint8_t>(type_len >> 16));
    buf.push_back(static_cast<uint8_t>(type_len >> 24));
    buf.insert(buf.end(), type.begin(), type.end());
    buf.push_back(0);

    // 9 bytes of property metadata (skipped by parser)
    for (int i = 0; i < 9; i++) buf.push_back(0);

    // FString value (positive length = UTF-8)
    int32_t val_len = static_cast<int32_t>(value.size() + 1); // +1 for null
    buf.push_back(static_cast<uint8_t>(val_len >> 0));
    buf.push_back(static_cast<uint8_t>(val_len >> 8));
    buf.push_back(static_cast<uint8_t>(val_len >> 16));
    buf.push_back(static_cast<uint8_t>(val_len >> 24));
    buf.insert(buf.end(), value.begin(), value.end());
    buf.push_back(0); // null terminator

    return buf;
}

TEST(MetadataTest, ExtractSlotName) {
    auto data = make_str_property("SlotName", "StrProperty", "savegame_0");
    auto meta = extract_metadata_from_bytes(data);

    ASSERT_TRUE(meta.slot_name.has_value());
    EXPECT_EQ(*meta.slot_name, "savegame_0");
}

TEST(MetadataTest, ExtractDisplayName) {
    auto data = make_str_property("DisplayName", "StrProperty", "My Cool Save");
    auto meta = extract_metadata_from_bytes(data);

    ASSERT_TRUE(meta.display_name.has_value());
    EXPECT_EQ(*meta.display_name, "My Cool Save");
}

TEST(MetadataTest, ExtractMultipleProperties) {
    auto slot = make_str_property("SlotName", "StrProperty", "savegame_3");
    auto display = make_str_property("DisplayName", "StrProperty", "Test World");

    std::vector<uint8_t> combined;
    combined.insert(combined.end(), slot.begin(), slot.end());
    combined.insert(combined.end(), display.begin(), display.end());

    auto meta = extract_metadata_from_bytes(combined);

    ASSERT_TRUE(meta.slot_name.has_value());
    EXPECT_EQ(*meta.slot_name, "savegame_3");
    ASSERT_TRUE(meta.display_name.has_value());
    EXPECT_EQ(*meta.display_name, "Test World");
}

TEST(MetadataTest, EmptyData) {
    std::vector<uint8_t> empty;
    auto meta = extract_metadata_from_bytes(empty);

    EXPECT_FALSE(meta.slot_name.has_value());
    EXPECT_FALSE(meta.display_name.has_value());
    EXPECT_FALSE(meta.errors.empty());
}

TEST(MetadataTest, GarbageDataNoCrash) {
    // Random bytes — should not crash
    std::vector<uint8_t> garbage(256);
    for (size_t i = 0; i < garbage.size(); i++) {
        garbage[i] = static_cast<uint8_t>(i * 17 + 43);
    }
    auto meta = extract_metadata_from_bytes(garbage);
    // Should gracefully return empty metadata
    EXPECT_FALSE(meta.slot_name.has_value());
    EXPECT_FALSE(meta.display_name.has_value());
}
