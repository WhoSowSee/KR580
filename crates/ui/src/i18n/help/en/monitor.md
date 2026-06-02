KR580 Monitor (Ctrl+M) - virtual display device.

Two independent layers:
- Text layer: 64x25 characters, monochrome. Output via OUT to monitor port.
- Graphics layer: 512x256 pixels, 1 bit/pixel. Frame buffer filled sequentially.

Monitor window features:
- Split/Unified view toggle
- Byte stream filter: all data / graphics only / text only
- Clear buffer, Save image as PNG
- Raw byte stream viewer