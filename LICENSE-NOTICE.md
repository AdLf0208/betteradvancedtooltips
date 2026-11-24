# Licence provenance

- Mod: `betteradvancedtooltips` (Better Advanced Tooltips) by **Latvian DEV** —
  <https://github.com/latvian-dev/better-advanced-tooltips>
- Licence: **MIT**, read from the mod's own jar
  `ports/_shared/licence-cache/better-advanced-tooltips-2101.1.0-build.5.jar`,
  whose `META-INF/neoforge.mods.toml` declares `license = "MIT"`, and from the
  repository's `LICENSE` ("MIT License, Copyright (c) 2026 Latvian DEV"). Rank 0:
  publishable with this source; the orchestrator handles publication.
- Upstream version analysed: **2101.1.0-build.5**, repository commit
  `d1e922ccf46456a344e202db7de252301861412e` (Minecraft Java 1.21.1, NeoForge).
- This port is a **clean-room Rust rewrite** written from the mod's observable
  behaviour, its public source repository and its jar's data files. No Java source
  was copied or translated, and no jar was decompiled.
- Nothing is carried: the mod's jar holds only client tooltip code and font icons
  (no blocks, items, models or recipes), so the package has no `assets/` or `data/`
  directory. The icon family is not ported (see `notes.md`).
- The complete corresponding source of this package is this directory: `src/`,
  `horizon-mod.toml`, `Cargo.toml` and the `LICENSE` text beside this notice.
