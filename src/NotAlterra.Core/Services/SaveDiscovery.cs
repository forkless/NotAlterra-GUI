// Save-folder discovery.
// Checks current user's default save locations — no cross-profile scanning.
// Ported from src/discovery.rs

namespace NotAlterra.Services;

public static class SaveDiscovery
{
    public record DiscoveredFolder(string Label, string Path);

    /// Known save-root patterns (relative from user profile).
    private static readonly (string Label, string Rel)[] KnownPatterns =
    [
        ("Steam (LocalLow)",       @"AppData\LocalLow\Unknown Worlds\Subnautica2"),
        ("Steam (LocalLow, alt)",  @"AppData\LocalLow\Unknown Worlds\Subnautica 2"),
        ("AppData Local",          @"AppData\Local\Subnautica2\Saved\SaveGames"),
        ("AppData Local (alt)",    @"AppData\Local\Subnautica 2\Saved\SaveGames"),
        ("Saved Games",            @"Saved Games\Subnautica2"),
        ("Saved Games (alt)",      @"Saved Games\Subnautica 2"),
        ("Documents",              @"Documents\Subnautica2"),
        ("Epic / Steam custom",    @"AppData\LocalLow\Subnautica2"),
    ];

    // ── helpers ──────────────────────────────────────────────────────────

    private static bool HasSaveFiles(string dir)
    {
        if (!Directory.Exists(dir)) return false;
        return Directory.EnumerateFiles(dir, "*.sav").Any()
            || Directory.EnumerateFiles(dir, "*.save").Any();
    }

    private static string HomeDir =>
        Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

    private static string LocalAppData =>
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);

    // ── quick_discover ───────────────────────────────────────────────────

    /// Quick check of current user's default save locations.
    /// No scanning of other profiles. Returns first valid path or null.
    public static string? QuickDiscover()
    {
        // Primary: %LOCALAPPDATA%\Subnautica2\Saved\SaveGames
        var primary = Path.Combine(LocalAppData, "Subnautica2", "Saved", "SaveGames");
        if (HasSaveFiles(primary)) return primary;

        // Fallback: check home directory patterns
        foreach (var (_, rel) in KnownPatterns)
        {
            var candidate = Path.Combine(HomeDir, rel);
            if (HasSaveFiles(candidate)) return candidate;
        }

        return null;
    }

    // ── discover_save_folders ────────────────────────────────────────────

    /// Search all known locations. Returns deduplicated, ranked list.
    public static List<DiscoveredFolder> DiscoverAll()
    {
        var found = new List<DiscoveredFolder>();
        var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        // 1. Fast path: %LOCALAPPDATA%\Subnautica2\Saved\SaveGames
        var primary = Path.Combine(LocalAppData, "Subnautica2", "Saved", "SaveGames");
        if (HasSaveFiles(primary) && seen.Add(primary))
            found.Add(new DiscoveredFolder("AppData Local", primary));

        // 2. Current user profile — remaining patterns
        foreach (var (label, rel) in KnownPatterns)
        {
            var candidate = Path.Combine(HomeDir, rel);
            if (HasSaveFiles(candidate) && seen.Add(candidate))
                found.Add(new DiscoveredFolder(label, candidate));
        }

        // 3. Xbox / Game Pass wildcard scan
        var pkgRoot = Path.Combine(HomeDir, @"AppData\Local\Packages");
        if (Directory.Exists(pkgRoot))
        {
            foreach (var pkg in Directory.EnumerateDirectories(pkgRoot, "*Subnautica2*"))
            {
                var wgs = Path.Combine(pkg, "SystemAppData", "wgs");
                if (HasSaveFiles(wgs) && seen.Add(wgs))
                    found.Add(new DiscoveredFolder("Xbox / Game Pass", wgs));
            }
        }

        return found;
    }
}
