# Upstream analysis — Better Advanced Tooltips

- **Jar**: `ports/_shared/licence-cache/better-advanced-tooltips-2101.1.0-build.5.jar`,
  37,653 B, SHA-256 `0f76d46f2f18eb67e67c7974170d7b27deb66473396461c8dc50d6949de1449b`,
  64 entries.
- **Repository**: <https://github.com/latvian-dev/better-advanced-tooltips>, commit
  `d1e922ccf46456a344e202db7de252301861412e` (shallow clone at
  `ports/_shared/upstream/betteradvancedtooltips/`).
- **Licence**: MIT from `META-INF/neoforge.mods.toml` and the repo's `LICENSE`.

## Jar inventory (families, not guessed from the name)

The 64 entries are: 8 client classes (`BATClientEventHandler`, `BATConfig`,
`BATIcons`, `BATMod`, `ItemTagIconsEvent`, `TagInstance`, `TooltipTagType`,
`CreativeModeInventoryScreenMixin`, `ItemStackMixin`), two mixin/neoforge metadata
files, a logo, and `assets/betteradvancedtooltips/` with `font/icons.json`,
`font/tags.json`, `lang/en_us.json` and 30 small PNG glyphs (24 `icons/*` plus
8 `icons/tag/*`). No blocks, items, models, recipes or loot tables: this is a
pure client tooltip mod, as the ledger's `client-ui`/`client` row says.

The jar's own description: *"Adds tag, component and other tooltips to items in
F3+H mode"*.

## Behaviour extracted (source files read)

`BATClientEventHandler.java:35-193` — `onItemTooltip`, `EventPriority.LOW`:

1. Return unless `flags.isAdvanced()` (F3+H), the local level exists and the
   stack is non-empty (`:37-53`).
2. **Alt branch** (`:59-111`, `componentTooltip` default `true`): for each entry
   of `stack.getComponentsPatch()`, append a line with a font glyph, `!` when the
   patch removes the component, then `<id>` (yellow), then `=<value>` when a value
   is present. With Shift as well (`:91-110`) it also walks the stack's prototype
   for components not in the patch (grey). A codec error appends
   `<id> errored, see log` in dark red.
3. **Shift branch** (`:112-191`):
   - **fuel** (`:113-133`, `fuelTooltip` default `true`): if `getBurnTime` > 0,
     append `🔥 Fuel: <ticks> t | <ticks/20> s | <ticks/200>x`, all values yellow
     with a gold prefix, seconds and smelted count printed as Java floats.
   - **tags** (`:135-191`, `tagTooltip` default `true`): gather the tags of the
     item, its block (BlockItem), fluid (BucketItem), entity type (spawn egg),
     enchantment, instrument, painting variant and banner pattern into a map keyed
     by tag id, post `ItemTagIconsEvent`, then append one sorted line per tag:
     a glyph, `#<tag>` in dark grey, then one registry-type glyph per registry.
4. `TagInstance.java:20-32` and `TooltipTagType.java:19-53` define those lines'
   text: `#<namespaced tag>` and one-letter glyphs for banner_pattern (A), block
   (B), enchantment (C), entity_type (E), fluid (F), item (I), instrument (N),
   painting_variant (P).
5. `BATConfig.java:21-27` defaults: `remove_creative_tab_tooltip=false`,
   `remove_component_count_tooltip=true`, `fuel_tooltip=true`, `tag_tooltip=true`,
   `component_tooltip=true`.

## Where the rewrite differs

- **Tag family is not ported.** The pin's `Tags` service offers
  `is_in_tag(ItemKey, TagKey)` and `items(TagKey)` (members of a *named* tag), but
  no "all tags of this item" query. Enumerating tags would mean hard-coding a tag
  list, i.e. inventing data. Recorded as `blocked(no-item-tag-enumeration)` with the
  grep evidence in `notes.md`.
- **Remove-line options are not ported.** They edit vanilla lines produced by
  `CreativeModeInventoryScreenMixin`/`ItemStackMixin`; the readable `item_tooltip`
  hook sees the finished list but has no stable identity for "component count" or
  "creative tab" lines, so removing them cannot be asserted without guessing.
- **Font glyphs are images, not text.** The parsed oracle asserts the readable text
  (`Fuel: …`, `id=value`), never a screenshot; this is the client gate's ruling.
- **Upstream text quirk kept**: `BATClientEventHandler.java:129` sizes the smelted
  slice with `i.length()` but takes it from `s` (the seconds string). The port
  reproduces that observable text rather than "fixing" a harmless quirk.
