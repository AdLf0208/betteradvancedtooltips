//! Better Advanced Tooltips for Horizon — the one `impl Hooks` at the crate root.
//!
//! The rules live in [`runtime`] and have no `mod_api` type. This file only reads the
//! game (advanced flag, Shift/Alt bits, burn time, the stack's component patch) and
//! writes the parsed lines back into the tooltip or, for the probe command, the
//! client chat so a harness can assert them.
//!
//! Upstream `BATClientEventHandler.onItemTooltip` only acts in F3+H mode: Alt adds
//! one line per patched component; Shift adds the fuel line and the tag lines. The
//! pin exposes `advanced`/`creative` on the hook and the exact Shift/Alt bits on the
//! client host, so that gating is preserved. The tag family is not ported: the SDK's
//! `Tags` service answers membership for a named tag but cannot enumerate every tag
//! an item belongs to (see notes.md).

pub mod runtime;

use mod_api::{
    self, ClientHost, ClientModifiers, Command, CommandEvent, Compound, Flow, Hooks, Identifier,
    ItemStack, ItemTooltipEvent, RegisterClientCommandsEvent, Tag, Text,
};

/// GLFW modifier bits, as `ClientModifiers` documents them.
const SHIFT: u32 = 1;
const ALT: u32 = 4;

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// A client command the probe can type. Registering it makes chat complete it;
    /// the `command` hook below answers it before the game's own dispatcher runs.
    fn register_client_commands(
        &self,
        _host: &mut dyn ClientHost,
        event: &mut RegisterClientCommandsEvent<'_>,
    ) {
        event
            .commands
            .push(Command::new("bat <mode:word> <item:word>"));
    }

    /// `/bat fuel <item>` and `/bat components <snbt>` report the exact line the
    /// tooltip hook would append, into this client's chat, so `horizon-ctl messages`
    /// can assert parsed text. A plain string is not a rendered image.
    fn command(&self, host: &mut dyn ClientHost, event: &mut CommandEvent<'_>) -> Flow {
        let lines = match runtime::parse_probe(event.command) {
            runtime::Probe::FuelProbe(item) => probe_fuel(host, &item),
            runtime::Probe::ComponentsProbe(snbt) => probe_components(host, &snbt),
            runtime::Probe::Unknown => return Flow::Continue,
        };
        for line in lines {
            let _ = host.chat_line(Text::literal(line));
        }
        Flow::Cancel
    }

    /// The mod's real behaviour: append the family's lines to the finished tooltip.
    fn item_tooltip(&self, host: &dyn ClientHost, event: &mut ItemTooltipEvent<'_>) {
        let modifiers: ClientModifiers = host.modifiers();
        let shift = modifiers.bits & SHIFT != 0;
        let alt = modifiers.bits & ALT != 0;
        match runtime::mode_for(event.advanced, shift, alt) {
            runtime::Mode::None => {}
            runtime::Mode::Fuel => {
                let ticks = host.item_burn_time(event.stack.item()).unwrap_or(0);
                if let Some(line) = runtime::fuel_line(ticks) {
                    event.lines.push((Text::literal(line), u32::MAX));
                }
            }
            runtime::Mode::Components => {
                for line in component_lines(event.stack) {
                    event.lines.push((Text::literal(line), u32::MAX));
                }
            }
        }
    }
}

/// Resolve an item id to a burn-time probe. An unresolvable id yields no line, the
/// same as a non-fuel item, so the harness can pair it with a burnable control.
fn probe_fuel(host: &dyn ClientHost, item: &str) -> Vec<String> {
    let Ok(id) = Identifier::parse(item) else {
        return Vec::new();
    };
    let Some(key) = host.item_key(&id) else {
        return Vec::new();
    };
    runtime::fuel_line(host.item_burn_time(key).unwrap_or(0))
        .into_iter()
        .collect()
}

/// Report the patched components of a stack with one known patch, so the
/// component family has the same parsed in-game oracle as the fuel family.
///
/// `minecraft:damage` lives in the stack's dedicated damage field, not its
/// `Components` patch (SDK `item_stack.rs:387-399`), so the probe writes a
/// `minecraft:custom_name` string instead: that produces a genuine patch that
/// `ItemStack::components()` exposes to the very same `component_lines` the
/// tooltip hook calls.
fn probe_components(host: &dyn ClientHost, item: &str) -> Vec<String> {
    let Ok(id) = Identifier::parse(item) else {
        return Vec::new();
    };
    let Some(key) = host.item_key(&id) else {
        return Vec::new();
    };
    let Some(mut stack) = ItemStack::new(key, 1) else {
        return Vec::new();
    };
    let patch: Compound = [(
        "minecraft:custom_name".into(),
        Tag::String("Probe Blade".into()),
    )]
    .into_iter()
    .collect();
    if stack.set_components(patch).is_err() {
        return Vec::new();
    }
    component_lines(&stack)
}

/// The readable part of each patched component on a stack, in the component id's
/// canonical order (`Components::as_nbt` is a `BTreeMap`, like upstream's sorted map).
fn component_lines(stack: &ItemStack) -> Vec<String> {
    let Some(components) = stack.components() else {
        return Vec::new();
    };
    components
        .as_nbt()
        .iter()
        .map(|(id, value)| runtime::component_line(&id.to_string_lossy(), Some(&tag_text(value))))
        .collect()
}

/// A component value as plain text. The NBT leaves are the readable ones; a nested
/// compound/list is reported by its kind, as upstream's pretty component would show
/// a structured value rather than a single scalar.
fn tag_text(tag: &Tag) -> String {
    match tag {
        Tag::Byte(value) => value.to_string(),
        Tag::Short(value) => value.to_string(),
        Tag::Int(value) => value.to_string(),
        Tag::Long(value) => value.to_string(),
        Tag::Float(bits) => f32::from_bits(*bits).to_string(),
        Tag::Double(bits) => f64::from_bits(*bits).to_string(),
        Tag::String(value) => value.to_string_lossy(),
        Tag::End => "end".to_owned(),
        Tag::ByteArray(_) => "byte_array".to_owned(),
        Tag::List(_) => "list".to_owned(),
        Tag::Compound(_) => "compound".to_owned(),
        Tag::IntArray(_) => "int_array".to_owned(),
        Tag::LongArray(_) => "long_array".to_owned(),
    }
}
