# Settings

Settings open from the menu or Ctrl+,. `PreferencesState` holds confirmed
preferences and the optional `SettingsDialog` draft. Storage format and
compatibility are defined in [Persistence](persistence.md#settings); filesystem
locations depend on [installation mode](installer.md#modes).

## Categories and search

| Category | Controls |
|---|---|
| General | Language, startup speed, Follow PC, operand highlighting, title-bar file name and file associations |
| External Devices | Floppy image, HDD directory, printer defaults/dialog mode, monitor layout and network startup endpoints |
| Appearance | Colour scheme |
| Shortcuts | Configurable action bindings and shortcut reset |

Search matches localized category names, setting labels/hints, theme names and
shortcut actions. A category or parent-label match retains its complete control;
an individual theme/action match narrows that compound control. Scroll edge
hints are hidden while searching.

The content pane keeps its mounted scroll state with a hidden native scrollbar.
Wheel lines use a smaller step; pixel deltas clamp exactly at both ends.
Upper/lower fades appear only when content remains in that direction. Those
overlays stay inside the outer border and do not change layout.

## Live preview and saving

Language, theme, shortcut map and active speed preview immediately. The startup
speed preference and the speed currently selected in the main window have
separate baselines. Cancelling an unrelated settings edit does not replace the
session speed with its saved startup default.

Monitor layout also has separate saved and session values. Preview changes the
open monitor; Cancel restores its pre-open view. Save stores the chosen startup
layout. A session split/merge toggle is not persisted by saving an unrelated
preference.

Device paths and endpoints remain draft values until Save. Printer setup for the
global target also uses a draft; its session counterpart is described in
[Device windows](ui_devices.md#printer-buffer).

Save keeps the dialog open and submits its captured draft to the FIFO file
worker. Save, Reset and Cancel are disabled during storage; other draft controls
remain editable. Success advances the cancel baseline to the submitted values,
preserving later edits and scroll position. A failed write retains the previous
baseline and reports an error.

Each background settings operation reads the latest file, applies only its owned
change and writes atomically. Printer-profile and path updates cannot overwrite
one another with stale whole-file copies. Event handlers use the confirmed cache.

A short animated notice appears after successful Save or Reset. Repeated actions
continue from the current presentation; dismissal and expiry remove only the
notice. No success notice is emitted before storage completes.

Cancel, backdrop click and Esc roll back unsaved live previews to their latest
confirmed baselines. Esc first dismisses an open selector or Reset confirmation.
Reset requires confirmation, persists defaults, and advances the baseline only
after success.

## Focus and keyboard

Settings owns its input while open. Focus starts logically on the first content
control without a keyboard outline. The first Tab/Shift+Tab enables that outline;
Enter and pointer activation hide it before applying the action.

| Input | Action |
|---|---|
| Ctrl+Tab / Ctrl+Shift+Tab | Cycle Search, Sidebar, Content and Footer |
| Tab / Shift+Tab | Traverse enabled items inside the current section, wrapping at its ends |
| Up / Down in Sidebar | Move its category cursor without selecting a category |
| Enter in Sidebar | Apply the cursor's category |
| Up / Down in language selector | Move its option highlight without applying it |
| Enter in selector | Apply the highlighted option and close |
| Left / Right on speed controls | Move through the speed choices |
| Esc | Close selector, confirmation, then Settings in that order |

Entering Search focuses its real input. Leaving an input for a non-input section
removes its caret. Search is a single Tab stop; moving between sections requires
Ctrl+Tab. Navigation never falls through to RAM/register controls underneath.

The footer normally traverses Reset, Cancel and Save. Shortcuts adds Reset
shortcuts. Reset confirmation starts on Cancel with a filled logical selection;
keyboard traversal switches it to an outline.

## Shortcut bindings

Bindings are live-previewed in menus and tooltips. Assigning an existing chord
removes its previous assignment, except for the two context-scoped RAM actions:
pattern search in address/value fields and replacement in a selected RAM cell.
They share Ctrl+Enter by default but can be rebound independently.

Physical key positions preserve letter actions across keyboard layouts. Focused
inputs retain native editing behavior when they consume the key. Printable
rebound chords must not insert their letter into an underlying text field.

Reset shortcuts previews the default map for this draft. Save confirms it;
Cancel restores the last confirmed map. The complete current assignments are
visible in Settings and the in-app shortcut reference.

## Device defaults and associations

The floppy image, HDD directory, printer and network endpoints are startup
defaults. Device-window attachment, endpoint and printer changes are session
overrides and do not implicitly save these fields.

Printer settings retain the validated driver configuration, not just its name.
Clear returns to the OS default target. Driver access stays off the UI thread;
the complete setup lifecycle is in
[Device windows](ui_devices.md#printer-setup-and-properties).

Windows file associations show Add when either handler is missing and Remove
when both exist. While Settings is open, Windows registration checks refresh
that cached state unless a change is pending. Unix does not poll registration.
Linux/macOS expose Set as default; confined Snap hides the action.

An association operation is independent of the settings save and keeps one
pending flag even if Settings closes. Failure cannot report success. Linux
removal restores an owned previous default; macOS removal is unsupported.
See [desktop integration](installer.md#desktop-integration) for platform and
ownership details.
