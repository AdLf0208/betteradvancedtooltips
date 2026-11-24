# Better Advanced Tooltips — port notes

- **Class / tier**: `client-ui` / `client` (ledger row and `horizon/docs/modpacks.md`
  §"Classes"/"Tiers"). The approach is a **client tooltip mod**: no content carry,
  no machines, no deep entities — only the readable tooltip-line hook and client
  host reads.
- **Licence**: **MIT**, rank 0 (`META-INF/neoforge.mods.toml` in the cached jar and
  the upstream repo's `LICENSE`, "Copyright (c) 2026 Latvian DEV"). Publication is
  the orchestrator's call; first publication version **0.1.0**.
- **Pin**: `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins
  `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, game 26.3, `api = "0.2"`, R9: SDK
  path dependency only, no `[workspace]`, no packed `Cargo.lock`.
- **Upstream**: latvian.dev/better-advanced-tooltips, commit
  `d1e922ccf46456a344e202db7de252301861412e`, version 2101.1.0-build.5. See
  `FINDINGS.md` for the full inventory and behaviour extraction.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

## Jar inventory (before any family was chosen)

37,653 B, SHA-256 `0f76d46f2f18eb67e67c7974170d7b27deb66473396461c8dc50d6949de1449b`,
64 entries: eight client tooltip/config classes and two mixins, two metadata files,
a logo, and `assets/betteradvancedtooltips/` with `font/icons.json`, `font/tags.json`,
`lang/en_us.json` and 30 PNG glyphs. **No** blocks, items, models, recipes or loot
tables. The artefact, not the mod's name, decided the families below.

## Client gate (binding, done before code)

The brief requires a stamped tooltip hook exposing a **readable** value, not just
a rendered image. Measured from `ports/_shared/sdk/0a875da/surface.json`:

```
hook:item_tooltip   side=client contract=listen
  arguments: stack: &'a ItemStack, advanced: bool, creative: bool,
             lines: &'a mut Vec<(text::Text, u32)>
  doc: "Add, change, reorder or remove lines. ... Asked each frame the tooltip
        shows ... Client-side."
method:prelude::ClientInputClientHost::modifiers
  -> ClientModifiers { bits: u32 }     # shift 1, control 2, alt 4, super 8
method:prelude::ItemsHost::item_burn_time(ItemKey) -> Option<i32>
handle:item::item_stack::Components::as_nbt -> &Compound
handle:NetworkClientHost::chat_line(Text) -> Result<(), TextFormError>
```

So the gate **passes**: a parsed tooltip line is readable, the mod's Shift/Alt
gating is exactly reproducible, and both families use real game reads rather than
invented data. `chat_line` puts the probe's parsed line into client chat, which
`horizon-ctl messages` reads — an oracle, not a screenshot.

## Red-first TDD (verbatim)

First `cargo test --offline` on the stubbed `runtime.rs`:

```
running 8 tests
...
test result: FAILED. 2 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out
failures:
    runtime::tests::component_lines_read_id_and_value_or_removal
    runtime::tests::fuel_coal_is_ticks_seconds_and_items
    runtime::tests::fuel_fractional_values_keep_java_short_decimals
    runtime::tests::gating_alt_is_components_and_bare_advanced_is_nothing
    runtime::tests::gating_shift_is_fuel_and_alt_wins_over_shift
    runtime::tests::probe_parses_both_families_and_refuses_other_lines
error: test failed, to rerun pass `--lib`
```

Saved as `ports/_shared/evidence/betteradvancedtooltips/sdk-test-red.log`.
A second red pinned the whole-line SNBT parse (`sdk-test-red3.log`):

```
assertion `left == right` failed
  left: Components("{id:")
 right: Components("{id: \"minecraft:stone\", count: 1, components: {\"minecraft:damage\": 5}}")
test result: FAILED. 0 passed; 1 failed; 8 filtered out
```

After implementation: **9 passed, 0 failed** (`sdk-test.log`).

## Coverage table — every family in the jar

| Family (jar evidence) | State | Assertion / reason |
|---|---|---|
| **fuel tooltip** (`BATClientEventHandler.java:113-133`) | **tested** | unit: 1600 → `Fuel: 1600 t \| 80 s \| 8x`, fractional and non-fuel cases; in-game: `/bat fuel minecraft:coal` → exact parsed line in `ctl messages`, `minecraft:stone` control emits none, `SELFTEST=1` inverts to `EXIT:1` |
| **component tooltip** (`:59-111`) | **tested** | unit: `id=value` and `!id`; in-game: `/bat components {…components:{minecraft:damage:5}}` → `minecraft:damage=5`, a patch-free stack control emits none, `SELFTEST=1` inverts |
| **prototype components with Shift+Alt** (`:91-110`) | not ported | needs every default component of the item; the pin exposes one default by id, not an enumeration of the prototype. No invented list. |
| **tag tooltip** (`:135-191`, `TagInstance`, `TooltipTagType`) | `blocked(no-item-tag-enumeration)` | `Tags` in the pin has `is_in_tag(ItemKey, TagKey)` and `items(TagKey)` — membership of a *named* tag, never "all tags of this item". Grep evidence below. |
| **remove component count tooltip** (default true) | not ported | vanilla line; no stable identity in the finished `lines` list to remove and assert |
| **remove creative tab tooltip** (default false) | not ported | same; a screen-mixin behaviour with no readable identity |
| **font icons / custom glyphs** (`assets/.../font/icons.json`, 30 PNGs) | not ported | rendered images, not readable text; the client gate forbids dressing a screenshot up as proof |

Tag-gap grep evidence (pin surface, `hook:*`/`handle:*`/`trait:Tags` only):

```
trait:Tags :: is_in_tag(ItemKey, TagKey) / items(TagKey) / blocks / fluids / key(&Identifier)
# no handle or trait method taking an ItemKey/ItemStack and returning its tag set
```

## Verification (final, re-read from the artefacts)

- **Package**: `ports/betteradvancedtooltips/betteradvancedtooltips.hmod`,
  **12,052 B**, SHA-256 `ba7ed7bc9a498d930a60732f072b2c14cca9eded75e4834c81792c2816929a38`,
  8 entries (`sdk-check.log`). No packed `Cargo.lock`, R9 tables only.
- **Tests**: `cargo test --offline` → **9 passed, 0 failed** (`sdk-test.log`);
  the two red legs are preserved in `sdk-test-red.log` / `sdk-test-red3.log`.
- **Clippy / fmt**: `cargo clippy --all-targets -- -D warnings` → 0 warnings
  (`sdk-clippy.log`); `cargo fmt --check` clean (`sdk-fmt.log`).
- **`check`**: `cargo horizon-mod check` → `native-mod, format native-1/0a875da…`,
  **EXIT 0** (`sdk-check.log`).
- **Both `fits`**: pin `0a875da  made for` **EXIT 0** (`sdk-fits.log`); other build
  `0b14f8a  fits` **EXIT 0** (`sdk-fits-other.log`).
- **Package gate**: `python3 ports/tools/verify-sdk-package.py betteradvancedtooltips`
  → **1/1** (runtime byte-identical, package matches tree, R9, tests, clippy, check,
  fits, skills gate).
- **In-game** (launcher-built game with the package compiled in, `--offscreen`,
  `horizon-ctl`, scratch home): transcript in
  `ports/_shared/evidence/betteradvancedtooltips-e2e/e2e.log` —
  `ctl chat "/bat fuel minecraft:coal"` → `ctl messages` `["Fuel: 1600 t | 80 s | 8x"]`;
  `minecraft:stone` control → no new line;
  `ctl chat "/bat components minecraft:diamond_sword"` → `["minecraft:custom_name=Probe Blade"]`;
  unknown-id control → no new line; differential game WITHOUT the package →
  `Unknown command`, no fuel line. `PASS … EXIT:0`.
  `SELFTEST=1` (`e2e-selftest.log`) inverts the fuel assertion and ends
  `FAIL: selftest: coal emitted the fuel line … / EXIT:1`.
  `python3 ports/tools/check-e2e-logs.py … --strict` → `logs=2 ok=2 missing-exit=0 bad-exit=0`.
- A `fits` red was found and fixed, not waived: the analyzer read this crate's
  `Probe::Components` enum variant as an SDK `Components` literal. Renaming the
  variants to `FuelProbe`/`ComponentsProbe` made **both** builds green.

## Gaps and deviations

1. **Tag family blocked**, not faked — see the coverage row and grep evidence.
2. **Upstream text quirk kept**: `BATClientEventHandler.java:129` slices the
   seconds string by the smelted string's length. Reproduced in `runtime::fuel_line`
   so the observable text matches; recorded in `FINDINGS.md`.
3. **Glyph prefixes are not in the parsed oracle**: the mod's lines start with a
   custom-font glyph. The hook still appends `Text::literal` of the readable part;
   the icon itself is a render-only asset this port does not ship.
4. **Probe command**: `/bat …` is a harness device the mod registers as a client
   command. It calls the same `runtime` functions as the tooltip hook and reports
   the result through `chat_line`; it is not a gameplay feature.
5. **SDK quirk found while building the probe**: `ItemStack::set_components` stores
   `minecraft:damage` in a dedicated field and removes it from the patch
   (`item_stack.rs:387-399`), and `item_from_snbt` decodes a stack whose
   `components()` is `None`. A parsed component line therefore needs a non-damage
   patch; the probe writes `minecraft:custom_name`. The tooltip hook itself reads
   whatever patch the game supplies, so this affects only the oracle, not the port.

## Next slice

**Family**: the tag family re-check, or the prototype-component branch, at the
next SDK pin (whichever the repin audit shows is newly expressible).
**Gap line**: `tag tooltip → blocked(no-item-tag-enumeration)` in the coverage table.
**Why it fits the tier**: still `client-ui`/`client` — a tooltip line, no content
or machines. **In-game assertion**: once a pin exposes an item→tags query, add
`/bat tags <item>` and assert the parsed `#<tag>` line for an item with a known tag
(e.g. `minecraft:oak_log`) while a tag-free item emits none, with `SELFTEST=1`
ending `EXIT:1`. Until then this row stays blocked and no tag list is invented.
