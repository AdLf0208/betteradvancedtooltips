//! Pure rules for Better Advanced Tooltips — no `mod_api` type, no world access.
//!
//! Everything here is the port's own logic, exercised by `cargo test` without staging
//! the game. The glue in `lib.rs` only reads the game (burn time, modifier keys, the
//! component patch) and hands the numbers to these functions.

/// Which extra lines the mod adds for a tooltip with these flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// F3+H is off, or no modifier is held: no extra lines.
    None,
    /// F3+H + Shift: the fuel line (upstream also adds tag lines here; see notes.md).
    Fuel,
    /// F3+H + Alt: the data-component lines (Shift is ignored when Alt is down).
    Components,
}

/// Pick the family from the tooltip's advanced flag and the window's modifier bits.
///
/// Upstream `BATClientEventHandler.onItemTooltip` (lines 45, 59, 112): return unless
/// `flags.isAdvanced()`; Alt takes the component branch; else Shift takes the fuel/tag
/// branch. The pin exposes Shift/Alt through `ClientHost::modifiers`, so the gating is
/// exact rather than approximated.
pub fn mode_for(advanced: bool, shift: bool, alt: bool) -> Mode {
    if !advanced {
        return Mode::None;
    }
    if alt {
        return Mode::Components;
    }
    if shift {
        return Mode::Fuel;
    }
    Mode::None
}

/// The fuel line upstream appends for a burnable stack (`BATClientEventHandler:113-132`).
///
/// `burn_ticks` is `ItemStack.getBurnTime` in ticks. The text is
/// `Fuel: <ticks> t | <seconds> s | <items>x`, where the seconds are
/// `ticks / 20` and the smelted count is `ticks / 200`, each printed with Java
/// float-string formatting (an integral float keeps one `.0` internally, which the
/// upstream code then strips). `None` for a stack that is not fuel.
pub fn fuel_line(burn_ticks: i32) -> Option<String> {
    if burn_ticks <= 0 {
        return None;
    }
    // Java's `String.valueOf(float)` keeps one `.0` for an integral value; the
    // upstream code later strips it. Rust prints `80` for `80.0f32`, so add it back
    // first to reproduce upstream's exact string operations.
    fn java_float(value: f32) -> String {
        let text = format!("{value}");
        if text.contains('.') {
            text
        } else {
            format!("{text}.0")
        }
    }
    let seconds = java_float(burn_ticks as f32 / 20.0);
    let smelted = java_float(burn_ticks as f32 / 200.0);

    let seconds_shown = seconds.strip_suffix(".0").unwrap_or(&seconds);
    // Upstream uses `s.substring(0, i.length() - 2)` here (BATClientEventHandler:129),
    // i.e. a slice of the *seconds* string sized by the *smelted* string. Keep it: it
    // is the observable text, and changing it would break parity for other burn times.
    let smelted_shown = if smelted.ends_with(".0") {
        let cut = smelted.len() - 2;
        seconds.get(..cut).unwrap_or(&seconds)
    } else {
        &smelted
    };
    Some(format!(
        "Fuel: {burn_ticks} t | {seconds_shown} s | {smelted_shown}x"
    ))
}

/// The readable part of one data-component line (`BATClientEventHandler:60-110`).
///
/// Upstream prefixes a font glyph and a colour; the glyph is an image, not text, so
/// the parsed value this port exposes is `id=value`, or `!id` for a removed component
/// (upstream's `!` marker), which is what an oracle can assert.
pub fn component_line(id: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => format!("{id}={value}"),
        None => format!("!{id}"),
    }
}

/// The probe command a harness sends: `/bat fuel <item>` or `/bat components <item>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    FuelProbe(String),
    ComponentsProbe(String),
    Unknown,
}

/// Parse the bare command text (no leading slash) into a probe.
pub fn parse_probe(line: &str) -> Probe {
    let Some(rest) = line.strip_prefix("bat ") else {
        return Probe::Unknown;
    };
    if let Some(item) = rest.strip_prefix("fuel ") {
        return Probe::FuelProbe(item.to_owned());
    }
    if let Some(item) = rest.strip_prefix("components ") {
        return Probe::ComponentsProbe(item.to_owned());
    }
    Probe::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuel_coal_is_ticks_seconds_and_items() {
        assert_eq!(fuel_line(1600).as_deref(), Some("Fuel: 1600 t | 80 s | 8x"));
    }

    #[test]
    fn fuel_fractional_values_keep_java_short_decimals() {
        assert_eq!(fuel_line(100).as_deref(), Some("Fuel: 100 t | 5 s | 0.5x"));
        assert_eq!(
            fuel_line(150).as_deref(),
            Some("Fuel: 150 t | 7.5 s | 0.75x")
        );
        assert_eq!(
            fuel_line(2400).as_deref(),
            Some("Fuel: 2400 t | 120 s | 12x")
        );
    }

    #[test]
    fn fuel_non_positive_is_not_a_line() {
        assert_eq!(fuel_line(0), None);
        assert_eq!(fuel_line(-1), None);
    }

    #[test]
    fn gating_requires_advanced_tooltips() {
        assert_eq!(mode_for(false, true, false), Mode::None);
        assert_eq!(mode_for(false, false, true), Mode::None);
    }

    #[test]
    fn gating_shift_is_fuel_and_alt_wins_over_shift() {
        assert_eq!(mode_for(true, true, false), Mode::Fuel);
        assert_eq!(mode_for(true, true, true), Mode::Components);
    }

    #[test]
    fn gating_alt_is_components_and_bare_advanced_is_nothing() {
        assert_eq!(mode_for(true, false, true), Mode::Components);
        assert_eq!(mode_for(true, false, false), Mode::None);
    }

    #[test]
    fn component_lines_read_id_and_value_or_removal() {
        assert_eq!(
            component_line("minecraft:damage", Some("5")),
            "minecraft:damage=5"
        );
        assert_eq!(component_line("minecraft:lore", None), "!minecraft:lore");
    }

    #[test]
    fn probe_parses_both_families_and_refuses_other_lines() {
        assert_eq!(
            parse_probe("bat fuel minecraft:coal"),
            Probe::FuelProbe("minecraft:coal".into())
        );
        assert_eq!(
            parse_probe("bat components minecraft:stone"),
            Probe::ComponentsProbe("minecraft:stone".into())
        );
        assert_eq!(parse_probe("bat"), Probe::Unknown);
        assert_eq!(parse_probe("give minecraft:coal"), Probe::Unknown);
    }

    #[test]
    fn probe_keeps_a_namespaced_item_id() {
        let line = "bat components minecraft:diamond_sword";
        assert_eq!(
            parse_probe(line),
            Probe::ComponentsProbe("minecraft:diamond_sword".to_owned())
        );
    }
}
