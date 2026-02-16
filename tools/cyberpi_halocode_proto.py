#!/usr/bin/env python3
"""
CyberPi HalocodeProtocol test - uses the REAL f3/f4 protocol discovered
from the makeblock pip package. Sends Python expressions inside f3 frames,
CyberPi evaluates them and returns results.

Protocol format (HalocodePackData):
  f3 [hdr_check] [datalen_lo] [datalen_hi] [type] [mode] [idx_lo] [idx_hi] [data...] [checksum] f4

  hdr_check = (0xf3 + datalen_lo + (datalen_hi << 8)) & 0xff   -- wait, let me re-check
  Actually from source: hdr_check = (((datalen>>8)&0xff) + (datalen&0xff) + 0xf3) & 0xff
  datalen = len(data) + 4  (includes type, mode, idx_lo, idx_hi)
  checksum = (type + mode + idx_lo + idx_hi + sum(data)) & 0xff

  For TYPE_SCRIPT (0x28), data = [script_len_lo, script_len_hi, *script_bytes]

Response format:
  f3 [hdr] [len_lo] [len_hi] 0x28 ... [data...] [checksum] f4
  Response data[3:] contains Python repr like "{'ret': 42}"
"""

import serial
import time
import sys
import threading

PORT = "/dev/ttyUSB0"
BAUD = 115200

# Protocol constants
TYPE_SCRIPT = 0x28
TYPE_SUBSCRIBE = 0x29
TYPE_ONLINE = 0x0d
MODE_RUN_WITHOUT_RESPONSE = 0x00
MODE_RUN_WITH_RESPONSE = 0x01


def build_script_frame(script: str, idx: int = 1, mode: int = MODE_RUN_WITH_RESPONSE) -> bytearray:
    """Build a HalocodePackData frame containing a Python script."""
    script_bytes = script.encode('utf-8')
    script_len = len(script_bytes)

    # data = [script_len_lo, script_len_hi, *script_bytes]
    data = [script_len & 0xff, (script_len >> 8) & 0xff]
    data.extend(script_bytes)

    typ = TYPE_SCRIPT
    idx_lo = idx & 0xff
    idx_hi = (idx >> 8) & 0xff

    # datalen includes type + mode + idx_lo + idx_hi + data
    datalen = len(data) + 4

    # Build frame
    buf = bytearray()
    buf.append(0xf3)  # header

    # header check byte
    hdr_check = (((datalen >> 8) & 0xff) + (datalen & 0xff) + 0xf3) & 0xff
    buf.append(hdr_check)

    buf.append(datalen & 0xff)
    buf.append((datalen >> 8) & 0xff)

    buf.append(typ)
    buf.append(mode)
    buf.append(idx_lo)
    buf.append(idx_hi)

    for b in data:
        buf.append(b)

    # checksum
    cksum = typ + mode + idx_hi + idx_lo
    for b in data:
        cksum += b
    cksum &= 0xff
    buf.append(cksum)

    buf.append(0xf4)  # footer
    return buf


def build_online_mode_frame() -> bytearray:
    """Build the goto_online_mode frame (TYPE_ONLINE, no mode byte)."""
    # From source: [0xf3, 0xf6, 0x03, 0x0, 0x0d, 0x0, 0x01, 0x0e, 0xf4]
    return bytearray([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x01, 0x0e, 0xf4])


def build_offline_mode_frame() -> bytearray:
    """Build the goto_offline_mode frame."""
    return bytearray([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x00, 0x0e, 0xf4])


class F3Parser:
    """Parse incoming f3/f4 frames byte-by-byte (mirrors HalocodeProtocol.on_parse)."""

    def __init__(self):
        self._buffer = []
        self._is_receiving = False
        self._datalen = 0
        self.ready = False
        self.frames = []  # collected complete frames

    def feed(self, byte: int):
        self._buffer.append(byte)

        # Detect header: check if last 4 bytes form valid header
        if len(self._buffer) > 3:
            b = self._buffer
            # hdr_check validation: (b[-1] + b[-2] + b[-4]) & 0xff == b[-3]
            # and b[-4] == 0xf3
            if b[-4] == 0xf3 and ((b[-1] + b[-2] + b[-4]) & 0xff) == b[-3]:
                # Valid header found, reset buffer to just the header
                self._buffer = [0xf3, b[-3], b[-2], b[-1]]
                self._datalen = self._buffer[2] + (self._buffer[3] << 8)
                self._is_receiving = True

        if self._is_receiving:
            # Complete frame: datalen + 6 (4 header + checksum + footer)
            if len(self._buffer) - 6 == self._datalen and self._buffer[0] == 0xf3:
                self.ready = True
                frame = list(self._buffer)
                self.frames.append(frame)

                # Extract type
                if len(frame) > 4:
                    frame_type = frame[4]
                    if frame_type == TYPE_SCRIPT:
                        self._parse_script_response(frame)
                    elif frame_type == TYPE_SUBSCRIBE:
                        self._parse_subscribe_response(frame)
                    elif frame_type == TYPE_ONLINE:
                        print(f"  [ONLINE] mode response")
                    else:
                        print(f"  [TYPE 0x{frame_type:02x}] unknown")

                self._buffer = []
                self._datalen = 0
                self._is_receiving = False

    def _parse_script_response(self, frame):
        """Parse TYPE_SCRIPT response - data[3:] contains Python repr."""
        # Frame: f3 hdr len_lo len_hi type mode idx_lo idx_hi [data...] cksum f4
        # data starts at index 8 (after idx_hi), ends at -2 (before cksum, f4)
        data = frame[7:-2]  # idx_hi is at 7, but let me recount...
        # Actually: 0=f3, 1=hdr, 2=len_lo, 3=len_hi, 4=type, 5=mode, 6=idx_lo, 7=idx_hi
        # data starts at 8, ends at -2
        data = frame[8:-2]
        if len(data) > 3:
            try:
                result_str = bytes(data[3:]).decode('utf-8', errors='replace')
                print(f"  [SCRIPT RESPONSE] raw={result_str}")
                try:
                    result = eval(result_str)
                    if isinstance(result, dict) and 'ret' in result:
                        print(f"  [VALUE] ret = {result['ret']}")
                except:
                    print(f"  [EVAL FAILED] couldn't parse: {result_str}")
            except:
                print(f"  [DECODE FAILED] data={data}")
        else:
            print(f"  [SCRIPT RESPONSE] short data: {data}")

        idx = frame[6] + (frame[7] << 8)
        print(f"  [IDX] {idx}")

    def _parse_subscribe_response(self, frame):
        """Parse TYPE_SUBSCRIBE response."""
        data = frame[5:-2]  # subscribe has different layout
        if len(data) > 3:
            try:
                result_str = bytes(data[3:]).decode('utf-8', errors='replace')
                print(f"  [SUBSCRIBE RESPONSE] {result_str}")
            except:
                print(f"  [SUBSCRIBE] raw data: {[hex(b) for b in data]}")


def hex_dump(data, prefix=""):
    """Pretty-print bytes."""
    return prefix + " ".join(f"{b:02x}" for b in data)


def main():
    print("=" * 70)
    print("CyberPi HalocodeProtocol Test")
    print("=" * 70)

    # Detect port
    port = PORT
    if len(sys.argv) > 1:
        port = sys.argv[1]

    print(f"\nOpening {port} @ {BAUD}...")
    ser = serial.Serial(port, BAUD, timeout=0.01)

    # Critical: don't toggle DTR/RTS (would reset ESP32)
    ser.dtr = False
    ser.rts = False

    parser = F3Parser()

    def read_all(timeout=2.0, label=""):
        """Read all available data for timeout seconds, feeding to parser."""
        raw = bytearray()
        start = time.time()
        while time.time() - start < timeout:
            chunk = ser.read(256)
            if chunk:
                raw.extend(chunk)
                for b in chunk:
                    parser.feed(b)
            else:
                time.sleep(0.01)
        if raw:
            # Show both raw hex and any ASCII
            ascii_str = raw.decode('ascii', errors='replace').replace('\r', '\\r').replace('\n', '\\n')
            print(f"  [{label}] {len(raw)} bytes raw")
            if len(raw) <= 200:
                print(f"  HEX: {hex_dump(raw)}")
            else:
                print(f"  HEX (first 100): {hex_dump(raw[:100])}")
            if any(32 <= b < 127 for b in raw):
                print(f"  ASCII: {ascii_str[:200]}")
        else:
            print(f"  [{label}] 0 bytes")
        return raw

    # Drain any boot messages
    print("\n--- Phase 0: Drain boot messages (2s) ---")
    read_all(2.0, "drain")

    # Phase 1: Send online mode frame
    print("\n--- Phase 1: Send goto_online_mode ---")
    online_frame = build_online_mode_frame()
    print(f"  TX: {hex_dump(online_frame)}")
    ser.write(online_frame)
    ser.flush()
    read_all(2.0, "online_mode")

    # Phase 2: Broadcast - send cyberpi.get_bri() to test connectivity
    print("\n--- Phase 2: Broadcast (cyberpi.get_bri) ---")
    for attempt in range(5):
        frame = build_script_frame("cyberpi.get_bri()", idx=attempt + 1, mode=MODE_RUN_WITH_RESPONSE)
        print(f"  TX[{attempt}]: {hex_dump(frame)}")
        ser.write(frame)
        ser.flush()
        read_all(1.0, f"broadcast_{attempt}")

        if parser.ready:
            print(f"  >>> Protocol ready! (after {attempt + 1} attempts)")
            break

    if not parser.ready:
        print("\n  WARNING: No f3 response received yet. Trying different approach...")

        # Try: first send online_mode, wait, then broadcast
        print("\n--- Phase 2b: Reset and retry ---")
        # Toggle RTS briefly to reset
        print("  Toggling RTS to reset CyberPi...")
        ser.rts = True
        time.sleep(0.1)
        ser.rts = False
        time.sleep(2.0)
        read_all(2.0, "post_reset_drain")

        # Send online mode
        print("  Sending online mode...")
        ser.write(build_online_mode_frame())
        ser.flush()
        read_all(1.5, "online_retry")

        # Broadcast again
        for attempt in range(5):
            frame = build_script_frame("cyberpi.get_bri()", idx=attempt + 10, mode=MODE_RUN_WITH_RESPONSE)
            print(f"  TX[{attempt}]: {hex_dump(frame)}")
            ser.write(frame)
            ser.flush()
            read_all(1.0, f"broadcast_retry_{attempt}")
            if parser.ready:
                print(f"  >>> Protocol ready after reset!")
                break

    if not parser.ready:
        print("\n  WARNING: Still no f3 responses. Trying subscribe approach...")

        # Try subscribe pattern: subscribe.add_item(key, func, params)
        print("\n--- Phase 2c: Subscribe approach ---")
        subscribe_script = "subscribe.add_item({0}, cyberpi.get_bri, ())"
        frame = build_script_frame(subscribe_script, idx=20, mode=MODE_RUN_WITH_RESPONSE)
        # Use TYPE_SCRIPT for subscribe too (that's what the library does)
        print(f"  TX: {hex_dump(frame)}")
        ser.write(frame)
        ser.flush()
        read_all(2.0, "subscribe")

    # Phase 3: If we got responses, try reading all sensors
    print("\n--- Phase 3: Sensor reads ---")
    sensors = [
        ("cyberpi.get_bri()", "brightness"),
        ("cyberpi.get_loudness('maximum')", "loudness"),
        ("cyberpi.get_battery()", "battery"),
        ("cyberpi.get_roll()", "roll"),
        ("cyberpi.get_pitch()", "pitch"),
        ("cyberpi.get_yaw()", "yaw"),
        ("cyberpi.get_acc('x')", "accel_x"),
        ("cyberpi.get_acc('y')", "accel_y"),
        ("cyberpi.get_acc('z')", "accel_z"),
        ("cyberpi.get_gyro('x')", "gyro_x"),
        ("cyberpi.get_gyro('y')", "gyro_y"),
        ("cyberpi.get_gyro('z')", "gyro_z"),
        ("cyberpi.get_firmware_version()", "firmware"),
        ("cyberpi.get_name()", "name"),
    ]

    for i, (script, label) in enumerate(sensors):
        idx = 100 + i
        frame = build_script_frame(script, idx=idx, mode=MODE_RUN_WITH_RESPONSE)
        print(f"\n  [{label}] TX: {hex_dump(frame[:20])}... ({len(frame)} bytes)")
        ser.write(frame)
        ser.flush()
        read_all(1.0, label)

    # Phase 4: Try sending a display command (fire-and-forget)
    print("\n--- Phase 4: Display test (no response expected) ---")
    display_script = 'cyberpi.console.println("Hello from Rust!")'
    frame = build_script_frame(display_script, idx=200, mode=MODE_RUN_WITHOUT_RESPONSE)
    print(f"  TX: {hex_dump(frame[:30])}... ({len(frame)} bytes)")
    ser.write(frame)
    ser.flush()
    read_all(1.0, "display")

    # Phase 5: Motor test (if mbot2 shield is connected)
    print("\n--- Phase 5: Motor test (tiny pulse) ---")
    motor_script = 'mbot2.drive_speed(10, 10)'
    frame = build_script_frame(motor_script, idx=201, mode=MODE_RUN_WITHOUT_RESPONSE)
    print(f"  TX: {hex_dump(frame[:30])}... ({len(frame)} bytes)")
    ser.write(frame)
    ser.flush()
    time.sleep(0.3)
    # Stop motors immediately
    stop_script = 'mbot2.drive_speed(0, 0)'
    frame = build_script_frame(stop_script, idx=202, mode=MODE_RUN_WITHOUT_RESPONSE)
    ser.write(frame)
    ser.flush()
    read_all(0.5, "motor_stop")

    # Summary
    print("\n" + "=" * 70)
    print(f"RESULTS: {len(parser.frames)} f3 frames received")
    print(f"Protocol ready: {parser.ready}")
    for i, frame in enumerate(parser.frames):
        print(f"  Frame {i}: {hex_dump(frame[:40])}{'...' if len(frame) > 40 else ''} ({len(frame)} bytes)")
    print("=" * 70)

    ser.close()


if __name__ == "__main__":
    main()
