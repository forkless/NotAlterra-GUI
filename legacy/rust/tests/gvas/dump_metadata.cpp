#include <gvas/parser.h>
#include <iostream>
#include <filesystem>

namespace fs = std::filesystem;

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: dump_metadata <file.sav>" << std::endl;
        return 1;
    }

    auto path = fs::path(argv[1]);
    if (!fs::exists(path)) {
        std::cerr << "File not found: " << path << std::endl;
        return 1;
    }

    std::cout << "File: " << path.filename() << std::endl;
    std::cout << "Size: " << fs::file_size(path) << " bytes" << std::endl;
    std::cout << "----------------------------------------" << std::endl;

    try {
        auto meta = notalterra::gvas::extract_full_metadata(path);

        auto show = [](auto const& label, auto const& val) {
            std::cout << label << ": ";
            if (val) std::cout << *val;
            else std::cout << "(not found)";
            std::cout << std::endl;
        };

        show("SlotName",      meta.slot_name);
        show("DisplayName",   meta.display_name);
        show("GameMode",      meta.game_mode);
        show("LevelName",     meta.level_name);
        show("BuildBranch",   meta.build_branch);
        show("bIsMultiplayerSave", meta.is_online ? "true" : "false");
        show("bWasMultiplayerSave", meta.was_multiplayer ? "true" : "false");

        auto show_num = [](auto const& label, auto const& val) {
            std::cout << label << ": ";
            if (val) std::cout << *val;
            else std::cout << "(not found)";
            std::cout << std::endl;
        };

        show_num("BuildNumber",  meta.build_number);
        show_num("SavesCount",   meta.saves_count);
        show_num("LatestVersion", meta.latest_version);
        show_num("DataVersion",  meta.data_version);
        show("PlaytimeSeconds",  meta.playtime_seconds);

    } catch (std::exception const& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }

    return 0;
}
