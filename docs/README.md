# Documentation

Start with the root [Russian README](../README.md) or [English README](../README-EN.md)
for installation and first use. The application's Help window contains the
task-oriented reference in both languages.

## Architecture and contracts

| Guide | Use it for |
|---|---|
| [Architecture](architecture.md) | Crate responsibilities, actors, workers, ownership and event flow |
| [Core CPU](core.md) | Execution, IRQ validation, timing and processor APIs |
| [Devices](devices.md) | Port protocols, buffers, backpressure, native printing and shutdown |
| [Persistence](persistence.md) | File layouts, atomic saves, import validation and settings compatibility |
| [Assumptions](assumptions.md) | Model limitations and verification gaps |

## Desktop application

| Guide | Use it for |
|---|---|
| [App and UI](ui_app.md) | State ownership, messages, editing, document flows and platform presentation |
| [Execution and schematic](ui_execution.md) | Run modes, follow-PC, HLT, buses, lamps and timing readouts |
| [Device windows](ui_devices.md) | Tool-window lifecycle, inspection and printer setup |
| [Settings](ui_settings.md) | Drafts, live preview, focus, search and persistence |
| [Updates](updates.md) | Release discovery, verified downloads, replacement and recovery |

## Development and distribution

| Guide | Use it for |
|---|---|
| [Contributing](../CONTRIBUTING.md) | Issues, changes, verification and pull requests |
| [Testing](testing.md) | Automated checks, CI and focused test commands |
| [Manual testing](manual_testing.md) | UI, driver and native integration smoke checks |
| [Repository maintenance](maintenance.md) | Repository configuration, publishing, dependency inspection and star history |
| [Installer](installer.md) | Setup, uninstall, install ownership, rollback and distribution builds |
| [RPM and Arch packages](linux_packages.md) | Native Linux package installation, building and verification |
| [Assets](assets.md) | Artwork sources, regeneration, licensing and embedding |
| [AI Policy](../AI_POLICY.md) | Review and disclosure of AI-assisted contributions |

Implementation requirements live under `prompt/`, as specified in
[AGENTS.md](../AGENTS.md). Each guide describes current contracts; release history
belongs in the [changelogs](../CHANGELOG-EN.md).
