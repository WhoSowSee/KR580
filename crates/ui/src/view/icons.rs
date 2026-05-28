//! Vector icons used by the UI.
//!
//! Each handle is built once from the bytes embedded at compile time
//! (`include_bytes!`) and then cloned cheaply by callers. iced's
//! `svg::Handle` is reference-counted internally, so the cost of a
//! `clone()` is bounded; the `LazyLock` guard exists so the first paint
//! does not allocate the handle inside the layout pass.

use std::sync::LazyLock;

use iced::widget::svg;

/// Bytes for an action-panel icon, located under `assets/icons/actions/`.
macro_rules! action_icon_bytes {
    ($name:literal) => {
        include_bytes!(concat!("../../../../assets/icons/actions/", $name, ".svg"))
    };
}

/// Bytes for a device-panel icon, located under `assets/icons/devices/`.
/// Used by the schematic's bottom row of peripheral chips (monitor, floppy,
/// HDD, network adapter, printer) — same `currentColor`-driven authoring
/// convention as the action icons, so the same `svg::Style` tinting helper
/// works for both families.
macro_rules! device_icon_bytes {
    ($name:literal) => {
        include_bytes!(concat!("../../../../assets/icons/devices/", $name, ".svg"))
    };
}

static PLAY: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("play").as_slice()));
static PAUSE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("pause").as_slice()));
static STEP_FORWARD: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("step-forward").as_slice()));
static REDO_DOT: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("redo-dot").as_slice()));
static RESET_RAM: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("reset-ram").as_slice()));
static REFRESH_CCW: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("refresh-ccw").as_slice()));
static RESET_REGISTERS: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("reset-registers").as_slice()));
static CHEVRONS_LEFT: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("chevrons-left").as_slice()));
static CHEVRONS_RIGHT: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("chevrons-right").as_slice()));
static FILE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("file").as_slice()));
static FOLDER_OPEN: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("folder-open").as_slice()));
static SAVE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("save").as_slice()));
static SAVE_AS: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("save-as").as_slice()));
static FILE_DOWN: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("file-down").as_slice()));
static FILE_UP: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("file-up").as_slice()));
static WINDOW_MINIMIZE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-minimize").as_slice()));
static WINDOW_MAXIMIZE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-maximize").as_slice()));
static WINDOW_RESTORE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-restore").as_slice()));
static WINDOW_CLOSE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-close").as_slice()));
static CPU: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("cpu").as_slice()));
static CLEAR_HALT: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("clear-halt").as_slice()));
static DEVICE_MONITOR: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(device_icon_bytes!("monitor").as_slice()));
static DEVICE_FLOPPY: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(device_icon_bytes!("floppy").as_slice()));
static DEVICE_HDD: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(device_icon_bytes!("hdd").as_slice()));
static DEVICE_NETWORK: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(device_icon_bytes!("network").as_slice()));
static DEVICE_PRINTER: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(device_icon_bytes!("printer").as_slice()));

/// Lucide `play` — solid right-pointing triangle. Used for "run program".
pub(super) fn play() -> svg::Handle {
    PLAY.clone()
}

/// Lucide `pause` — twin vertical bars. Used for the "stop / pause" toggle
/// state of the run button: when the user has armed the run state we swap
/// the play glyph for this one to mirror the reference KR-580 emulator.
pub(super) fn pause() -> svg::Handle {
    PAUSE.clone()
}

/// Lucide `step-forward` — triangle with a vertical stop bar. Used for
/// "execute one instruction".
pub(super) fn step_forward() -> svg::Handle {
    STEP_FORWARD.clone()
}

/// Lucide `redo-dot` — a curved arrow with a dot under it. Used for
/// "execute one machine tact".
pub(super) fn redo_dot() -> svg::Handle {
    REDO_DOT.clone()
}

/// Lucide `refresh-ccw` — counter-clockwise circular arrow pair. Used as
/// the running-state replacement for the "step instruction" glyph: while
/// a program is actually executing on the CPU, the second action button
/// becomes a "restart" affordance that resets the registers/flags and
/// re-runs the program from `0x0000`.
pub(super) fn refresh_ccw() -> svg::Handle {
    REFRESH_CCW.clone()
}

/// Custom KR-580 glyph: a memory module silhouette with a circular
/// reset arrow above it. Used for "reset RAM".
pub(super) fn reset_ram() -> svg::Handle {
    RESET_RAM.clone()
}

/// Custom KR-580 glyph: stacked register cells with a circular reset
/// arrow on the right. Used for "reset registers".
pub(super) fn reset_registers() -> svg::Handle {
    RESET_REGISTERS.clone()
}

/// Lucide `chevrons-left` — twin left-pointing chevrons. Used as the
/// speed stepper's left-side button.
pub(super) fn chevrons_left() -> svg::Handle {
    CHEVRONS_LEFT.clone()
}

/// Lucide `chevrons-right` — twin right-pointing chevrons. Used as the
/// glyph for the "apply" (Enter) buttons next to the memory cell and
/// register value editors, and as the speed stepper's right-side button.
pub(super) fn chevrons_right() -> svg::Handle {
    CHEVRONS_RIGHT.clone()
}

/// Lucide `file` — blank document silhouette with a folded-corner tab.
/// Used as the glyph next to "Новый файл" in the File dropdown.
pub(super) fn file() -> svg::Handle {
    FILE.clone()
}

/// Document silhouette with a small upward arrow under the folded-corner
/// tab — a "save" cassette with an "open / load" hint. Used next to
/// "Открыть" in the File dropdown so the row reads as "pull a file in".
pub(super) fn folder_open() -> svg::Handle {
    FOLDER_OPEN.clone()
}

/// Save-cassette silhouette with a small downward arrow at the bottom.
/// Used next to "Сохранить" in the File dropdown so the row reads as
/// "push the current state to a file".
pub(super) fn save() -> svg::Handle {
    SAVE.clone()
}

/// Save-cassette silhouette with a pencil overlay — the "save as" twin
/// of `save()`. Used next to "Сохранить как" so the user can tell the
/// two save rows apart at a glance.
pub(super) fn save_as() -> svg::Handle {
    SAVE_AS.clone()
}

/// Lucide `file-down` — document silhouette with a downward arrow inside.
/// Used next to "Импорт" in the File dropdown: data flows *into* the
/// emulator, so the arrow points down into the page body.
pub(super) fn file_down() -> svg::Handle {
    FILE_DOWN.clone()
}

/// Lucide `file-up` — document silhouette with an upward arrow inside.
/// Used next to "Экспорт" in the File dropdown: data flows *out* of the
/// emulator into a file on disk, so the arrow points up out of the page.
pub(super) fn file_up() -> svg::Handle {
    FILE_UP.clone()
}

/// Lucide `minus` — single horizontal stroke. Used for the "minimise"
/// caption button in our custom title bar; matches the convention every
/// desktop window manager uses for the leftmost caption glyph.
pub(super) fn window_minimize() -> svg::Handle {
    WINDOW_MINIMIZE.clone()
}

/// Lucide-style empty square — used as the "maximise" caption button
/// glyph when the window is currently restored. Reads as "fill the
/// screen" the same way the native Windows caption does.
pub(super) fn window_maximize() -> svg::Handle {
    WINDOW_MAXIMIZE.clone()
}

/// Two overlapping squares — used as the "restore" caption glyph when
/// the window is already maximised. Mirrors the native restore icon
/// so the caption tells the user "you can shrink me back".
pub(super) fn window_restore() -> svg::Handle {
    WINDOW_RESTORE.clone()
}

/// Lucide `x` — diagonal cross. Used for the "close" caption button;
/// the surrounding button styles tint the hover surface red so the
/// destructive action has the same affordance as the native chrome.
pub(super) fn window_close() -> svg::Handle {
    WINDOW_CLOSE.clone()
}

/// Lucide `cpu` — square IC silhouette with eight pin stubs. Sits at
/// the very left of the menu bar as a tiny brand mark, replacing the
/// "Эмулятор KR580VM80A" wordmark we removed: the user wanted a
/// quieter title bar, but losing every signifier of "this is a CPU
/// emulator" leaves the bar visually anonymous, and the IC glyph
/// reads as that signifier in a single 16 px square.
pub(super) fn cpu() -> svg::Handle {
    CPU.clone()
}

/// Custom KR-580 glyph: an octagonal stop sign on a post (the
/// universal "halt" affordance) paired with a counter-clockwise
/// reset arc on the right (the same arc shape `reset_registers`
/// uses for its reset semantics). Used as the menu glyph next to
/// "Сбросить флаг HLT" — the row that snaps `cpu.halted` back to
/// `false` without rewinding PC or wiping registers, which is a
/// strictly weaker reset than `Очистить регистры` and deserves a
/// distinct icon. The stop sign carries the "HLT was raised" half
/// of the meaning, the arc carries the "we're undoing it" half.
pub(super) fn clear_halt() -> svg::Handle {
    CLEAR_HALT.clone()
}

/// Lucide `monitor` — display silhouette on a stand. Used as the
/// peripheral-row glyph for the monitor chip on the schematic plate.
pub(super) fn device_monitor() -> svg::Handle {
    DEVICE_MONITOR.clone()
}

/// Lucide `hard-drive` — flat disk silhouette with two indicator dots.
/// Repurposed as the floppy-drive (FDD) chip on the peripheral row,
/// matching the visual idiom the user picked for the device strip.
pub(super) fn device_floppy() -> svg::Handle {
    DEVICE_FLOPPY.clone()
}

/// Material-style cassette/disk-pack silhouette: a cassette enclosure with
/// a disc cut-out, used as the hard-drive (HDD) chip on the peripheral
/// row. Authored with `fill="currentColor"` (no stroke), so the same
/// `svg::Style { color: ... }` tint pipeline still works because iced's
/// resvg backend honours `currentColor` for both `stroke` and `fill`.
pub(super) fn device_hdd() -> svg::Handle {
    DEVICE_HDD.clone()
}

/// Custom "globe on a stand" glyph: a meridian-cut sphere mounted on a
/// short pillar with two side feet. Used as the network-adapter chip on
/// the peripheral row — reads as "world / network" in the same idiom as
/// the rest of the Lucide-flavoured set.
pub(super) fn device_network() -> svg::Handle {
    DEVICE_NETWORK.clone()
}

/// Lucide `printer` — desktop printer silhouette with a paper tray.
/// Used as the printer chip on the peripheral row.
pub(super) fn device_printer() -> svg::Handle {
    DEVICE_PRINTER.clone()
}
