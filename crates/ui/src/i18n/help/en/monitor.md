KR580 Monitor (Ctrl+M) - virtual display device.

Two independent layers:
- Text layer: 64x20 characters, monochrome. Output via OUT to monitor port.
- Graphics layer: 256x256 pixels, 1 bit/pixel. Frame buffer filled sequentially.

Monitor window features:
- Detach into a separate borderless window, drag it by the custom title bar, or return it to the emulator window
- Pin the detached window above other windows; press again to restore normal stacking
- Split/Unified view toggle
- Byte stream filter: all data / graphics only / text only
- Clear buffer, Save image as PNG
- Raw byte stream viewer
