//! Vector icons used by the UI.
//!
//! Each handle is built once from compile-time bytes and cloned cheaply
//! by callers. iced's `svg::Handle` is reference-counted internally.

use std::sync::LazyLock;

use iced::widget::image as iced_image;
use iced::widget::svg;

macro_rules! action_icon_bytes {
    ($name:literal) => {
        include_bytes!(concat!("../../../../assets/icons/actions/", $name, ".svg"))
    };
}

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
static PANEL_DETACH: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("panel-detach").as_slice()));
static PANEL_ATTACH: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("panel-attach").as_slice()));
static PIN: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("pin").as_slice()));
static GLOBE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("globe").as_slice()));
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
static SEARCH: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("search").as_slice()));
static CHEVRON_DOWN: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("chevron-down").as_slice()));
static CHEVRON_RIGHT: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("chevron-right").as_slice()));
static EXPAND_ALL: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("expand-all").as_slice()));
static COLLAPSE_ALL: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("collapse-all").as_slice()));
static SQUARE_SPLIT_VERTICAL: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(action_icon_bytes!("square-split-vertical").as_slice())
});
static SQUARE_MERGE_VERTICAL: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(action_icon_bytes!("square-merge-vertical").as_slice())
});
static BINARY: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("binary").as_slice()));
static STACK: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("stack").as_slice()));
static LINE_SQUIGGLE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("line-squiggle").as_slice()));
static TEXT_CURSOR: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("text-cursor").as_slice()));
static TYPE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("type").as_slice()));
static BRUSH_CLEANING: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("brush-cleaning").as_slice()));
static IMAGE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("image").as_slice()));
static HARD_DRIVE_DOWNLOAD: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(action_icon_bytes!("hard-drive-download").as_slice())
});
static HARD_DRIVE_X: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("hard-drive-x").as_slice()));
static HARD_DRIVE_UPLOAD: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("hard-drive-upload").as_slice()));
static BUG: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("bug").as_slice()));
static TRASH_2: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("trash-2").as_slice()));
static FILE_PLUS_CORNER: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("file-plus-corner").as_slice()));
static BOOK_MARKED: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("book-marked").as_slice()));
static INFO: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("info").as_slice()));
static GITHUB: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("github").as_slice()));

/// 128px PNG variant of the application icon – large enough that the
/// About-dialog plate (64×64 px logical) downscales without visible
/// pixelation on HiDPI displays.
static APP_ICON: LazyLock<iced_image::Handle> = LazyLock::new(|| {
    iced_image::Handle::from_bytes(
        include_bytes!("../../../../assets/icons/icon-128.png").as_slice(),
    )
});

pub(super) fn play() -> svg::Handle {
    PLAY.clone()
}

pub(super) fn pause() -> svg::Handle {
    PAUSE.clone()
}

pub(super) fn step_forward() -> svg::Handle {
    STEP_FORWARD.clone()
}

pub(super) fn redo_dot() -> svg::Handle {
    REDO_DOT.clone()
}

/// Counter-clockwise circular arrow – the running-state replacement
/// for `step_forward()`. While a program runs, the second action button
/// becomes a "restart" affordance that resets registers and re-runs
/// from `0x0000`.
pub(super) fn refresh_ccw() -> svg::Handle {
    REFRESH_CCW.clone()
}

pub(super) fn reset_ram() -> svg::Handle {
    RESET_RAM.clone()
}

pub(super) fn reset_registers() -> svg::Handle {
    RESET_REGISTERS.clone()
}

pub(super) fn chevrons_left() -> svg::Handle {
    CHEVRONS_LEFT.clone()
}

pub(super) fn chevrons_right() -> svg::Handle {
    CHEVRONS_RIGHT.clone()
}

pub(super) fn file() -> svg::Handle {
    FILE.clone()
}

pub(super) fn folder_open() -> svg::Handle {
    FOLDER_OPEN.clone()
}

pub(super) fn save() -> svg::Handle {
    SAVE.clone()
}

pub(super) fn save_as() -> svg::Handle {
    SAVE_AS.clone()
}

pub(super) fn file_down() -> svg::Handle {
    FILE_DOWN.clone()
}

pub(super) fn file_up() -> svg::Handle {
    FILE_UP.clone()
}

pub(super) fn window_minimize() -> svg::Handle {
    WINDOW_MINIMIZE.clone()
}

pub(super) fn window_maximize() -> svg::Handle {
    WINDOW_MAXIMIZE.clone()
}

pub(super) fn window_restore() -> svg::Handle {
    WINDOW_RESTORE.clone()
}

pub(super) fn window_close() -> svg::Handle {
    WINDOW_CLOSE.clone()
}

pub(super) fn panel_detach() -> svg::Handle {
    PANEL_DETACH.clone()
}

pub(super) fn panel_attach() -> svg::Handle {
    PANEL_ATTACH.clone()
}

pub(super) fn pin() -> svg::Handle {
    PIN.clone()
}

pub(super) fn globe() -> svg::Handle {
    GLOBE.clone()
}

pub(super) fn cpu() -> svg::Handle {
    CPU.clone()
}

/// Octagonal stop sign with a counter-clockwise reset arc – used next
/// to the clear-HLT-flag menu entry. The arc shape matches `reset_registers`
/// to signal "reset", the stop sign carries the "HLT was raised" half.
pub(super) fn clear_halt() -> svg::Handle {
    CLEAR_HALT.clone()
}

pub(super) fn device_monitor() -> svg::Handle {
    DEVICE_MONITOR.clone()
}

pub(super) fn device_floppy() -> svg::Handle {
    DEVICE_FLOPPY.clone()
}

/// Cassette/disk-pack silhouette authored with `fill="currentColor"`
/// (no stroke). iced's resvg backend honours `currentColor` for both
/// stroke and fill, so the same `svg::Style { color: ... }` pipeline
/// tints it.
pub(super) fn device_hdd() -> svg::Handle {
    DEVICE_HDD.clone()
}

pub(super) fn device_network() -> svg::Handle {
    DEVICE_NETWORK.clone()
}

pub(super) fn device_printer() -> svg::Handle {
    DEVICE_PRINTER.clone()
}

pub(super) fn search() -> svg::Handle {
    SEARCH.clone()
}

pub(super) fn chevron_down() -> svg::Handle {
    CHEVRON_DOWN.clone()
}

pub(super) fn chevron_right() -> svg::Handle {
    CHEVRON_RIGHT.clone()
}

pub(super) fn expand_all() -> svg::Handle {
    EXPAND_ALL.clone()
}

pub(super) fn collapse_all() -> svg::Handle {
    COLLAPSE_ALL.clone()
}

pub(super) fn square_split_vertical() -> svg::Handle {
    SQUARE_SPLIT_VERTICAL.clone()
}

pub(super) fn square_merge_vertical() -> svg::Handle {
    SQUARE_MERGE_VERTICAL.clone()
}

pub(super) fn binary() -> svg::Handle {
    BINARY.clone()
}

pub(super) fn stack() -> svg::Handle {
    STACK.clone()
}

pub(super) fn line_squiggle() -> svg::Handle {
    LINE_SQUIGGLE.clone()
}

pub(super) fn text_cursor() -> svg::Handle {
    TEXT_CURSOR.clone()
}

pub(super) fn type_icon() -> svg::Handle {
    TYPE.clone()
}

pub(super) fn brush_cleaning() -> svg::Handle {
    BRUSH_CLEANING.clone()
}

pub(super) fn image() -> svg::Handle {
    IMAGE.clone()
}

pub(super) fn hard_drive_download() -> svg::Handle {
    HARD_DRIVE_DOWNLOAD.clone()
}

pub(super) fn hard_drive_x() -> svg::Handle {
    HARD_DRIVE_X.clone()
}

pub(super) fn hard_drive_upload() -> svg::Handle {
    HARD_DRIVE_UPLOAD.clone()
}

pub(super) fn file_plus_corner() -> svg::Handle {
    FILE_PLUS_CORNER.clone()
}

pub(super) fn trash_2() -> svg::Handle {
    TRASH_2.clone()
}

pub(super) fn bug() -> svg::Handle {
    BUG.clone()
}

pub(super) fn book_marked() -> svg::Handle {
    BOOK_MARKED.clone()
}

pub(super) fn info() -> svg::Handle {
    INFO.clone()
}

pub(super) fn github() -> svg::Handle {
    GITHUB.clone()
}

pub(super) fn app_icon() -> iced_image::Handle {
    APP_ICON.clone()
}
