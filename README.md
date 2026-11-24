# Better Advanced Tooltips for Horizon

A clean-room Rust port of **Better Advanced Tooltips** 2101.1.0-build.5 by
**Latvian DEV** ([upstream repository](https://github.com/latvian-dev/better-advanced-tooltips),
commit `d1e922ccf46456a344e202db7de252301861412e`), released under the **MIT**
licence (see `LICENSE`). This port is published under the same MIT licence with
this source.

Better Advanced Tooltips adds extra lines to item tooltips while advanced tooltips
(F3+H) are on. This port implements the families the pinned Horizon SDK can read:

- **fuel** (F3+H + Shift): `Fuel: <ticks> t | <seconds> s | <items>x` for any item
  with a burn time, read from `item_burn_time` — the game's own fuel value, not a
  table copied from the jar;
- **components** (F3+H + Alt): one readable `id=value` line per patched data
  component on the stack (`!id` for a removed component), read from the stack's
  component patch.

The original's **tag-icon** family is not ported: the pin's `Tags` service can
answer membership for a *named* tag but cannot enumerate every tag an item belongs
to, so carrying it would require invented data. The two "remove vanilla line"
options (`remove_component_count_tooltip`, `remove_creative_tab_tooltip`) are also
not ported; see `notes.md` for the coverage table and the exact API gaps.

## Package layout

One flat SDK package (`api = "0.2"`), pinned to Horizon
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a` and horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`:

```
horizon-mod.toml   the manifest and identity
Cargo.toml         SDK path dependency only (R9); no Cargo.lock
src/lib.rs         the one impl Hooks (tooltip + client probe command)
src/runtime.rs     the pure, unit-tested rules (no mod_api type)
LICENSE            upstream's MIT licence
```

The SDK is fetched into `.horizon/sdk/` by `cargo horizon-mod` and is never packed.

## In-game proof

`tools/e2e.sh` drives a launcher-built game through `horizon-ctl`. A client probe
command (`/bat fuel <item>`, `/bat components <item>`) runs the same pure rules the
tooltip hook uses and reports the parsed line into client chat; the harness reads
`ctl messages` and asserts the exact text, with a non-fuel/unknown-id control, a
differential game built without the package, and an inverted `SELFTEST` leg that
must end at `EXIT:1`.
