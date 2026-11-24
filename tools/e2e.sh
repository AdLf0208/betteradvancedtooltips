#!/usr/bin/env bash
# In-game proof for Better Advanced Tooltips (client tier, first publication 0.1.0).
#
# The mod adds parsed lines to item tooltips. The pin's `item_tooltip` hook exposes
# `lines: Vec<(Text, u32)>` (readable text), and `ClientHost::modifiers` exposes the
# exact Shift/Alt bits, so this port needs no screenshot: a client probe command runs
# the very same pure rules as the tooltip hook and reports the line into client chat,
# which `horizon-ctl messages` reads back as parsed text.
#
# Legs (all on a launcher-built game that carries this package):
#   * fuel        `/bat fuel minecraft:coal` -> exact `Fuel: 1600 t | 80 s | 8x`;
#   * fuel-empty  `/bat fuel minecraft:stone` -> no Fuel line (non-fuel control);
#   * components  `/bat components minecraft:diamond_sword` -> `minecraft:custom_name=Probe Blade`;
#   * comp-empty  `/bat components no_such:item` -> no component line (unknown id refused).
# CONTROL_GAME adds a differential leg: the same Nightly WITHOUT the package knows
# no `/bat` and emits no line, so a green run cannot come from the engine.
# SELFTEST=1 inverts the fuel assertion (coal must emit NO line): the run MUST fail,
# which is the fail-capability proof. Every run's last line is `EXIT:<status>`.
set -uo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/betteradvancedtooltips
CTL=${CTL:-$PORT/target/horizon-cargo/fast/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/betteradvancedtooltips-e2e.sock}
SELFTEST=${SELFTEST:-0}

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing launcher-built game: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/bat-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }
say() { echo "== $*"; }

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" || true
    # The last line of every log: check-e2e-logs.py reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch WORLD [GAME]
    local game=${2:-$GAME}
    rm -f "$SOCK"
    "$CTL" --socket "$SOCK" launch \
        --game "$game" --world "$1" --seed 20261105 --creative --offscreen --wait \
        --game-dir "$SCRATCH/game" --resources "$RES"
    ctl wait-ready 180
}

# Send one probe command through the player's chat (a native mod command needs
# `ctl chat`, not `ctl command`). Chat history is never cleared, so diff the
# message list before and after and return only the lines this call added: that
# is what makes the "no line" control assertions non-vacuous.
probe() { # probe <full command without slash>
    local before after
    before=$(ctl messages 100 | jq -c '.chat')
    echo "== ctl chat \"/$1\"" >&2
    ctl chat "/$1"
    ctl wait 2
    after=$(ctl messages 100 | jq -c '.chat')
    echo "ctl messages before -> $before" >&2
    echo "ctl messages after  -> $after" >&2
    jq -r -n --argjson b "$before" --argjson a "$after" '
        ($b | group_by(.) | map({k: .[0], n: length})) as $B
        | ($a | group_by(.) | map({k: .[0], n: length})) as $A
        | [ $A[] | . as $x
            | (($B | map(select(.k == $x.k)) | .[0].n) // 0) as $bn
            | range(0; $x.n - $bn) | $x.k ]
        | .[]'
}

# --- modded game ---------------------------------------------------------------
say "launch (creative, offscreen, the package compiled into the game)"
launch BatE2E

say "family: fuel tooltip (F3+H + Shift)"
out=$(probe 'bat fuel minecraft:coal')
echo "$out"
if printf '%s\n' "$out" | grep -Fq 'Fuel: 1600 t | 80 s | 8x'; then
    if [ "$SELFTEST" = 1 ]; then
        fail "selftest: coal emitted the fuel line; the inverted assertion (must be absent) fails as required"
    fi
    pass "parsed fuel line present: 'Fuel: 1600 t | 80 s | 8x'"
else
    if [ "$SELFTEST" = 1 ]; then
        pass "selftest: coal emitted no fuel line, the inverted assertion holds"
    else
        fail "coal emitted no parsed fuel line (expected 'Fuel: 1600 t | 80 s | 8x')"
    fi
fi

say "control: a vanilla item with no burn time emits no fuel line"
out=$(probe 'bat fuel minecraft:stone')
echo "$out"
if printf '%s\n' "$out" | grep -Fq 'Fuel:'; then
    fail "a non-fuel vanilla item emitted a fuel line"
else
    pass "non-fuel control emitted no fuel line"
fi

say "family: component tooltip (F3+H + Alt)"
out=$(probe 'bat components minecraft:diamond_sword')
echo "$out"
if printf '%s\n' "$out" | grep -Fq 'minecraft:custom_name=Probe Blade'; then
    pass "parsed component line present: 'minecraft:custom_name=Probe Blade'"
else
    if [ "$SELFTEST" = 1 ]; then
        pass "selftest: the component line is absent, as inverted"
    else
        fail "a patched component emitted no parsed line (expected 'minecraft:custom_name=Probe Blade')"
    fi
fi

say "control: an unknown item id is refused and emits no component line"
out=$(probe 'bat components no_such:item')
echo "$out"
if printf '%s\n' "$out" | grep -Eq 'minecraft:custom_name=|![a-z_]+:'; then
    fail "an unknown item id emitted a component line"
else
    pass "unknown-id control emitted no component line"
fi

# --- differential leg ----------------------------------------------------------
if [ -n "$CONTROL_GAME" ]; then
    say "differential: the same Nightly built WITHOUT this package"
    launch BatControl "$CONTROL_GAME"
    out=$(probe 'bat fuel minecraft:coal')
    echo "$out"
    if printf '%s\n' "$out" | grep -Fq 'Fuel:'; then
        fail "the unmodded game emitted the fuel line — the assertion is vacuous"
    else
        pass "unmodded control emitted no fuel line"
    fi
fi

pass "all tooltip families asserted from parsed in-game text"
