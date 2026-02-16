#!/usr/bin/env python3
"""
CyberPi f3 Protocol over Serial - DEFAULT MODE (no upload mode!)

KEY INSIGHT: Every previous serial f3 test entered upload/REPL mode first.
But f3 is CyberPiOS's NATIVE protocol for Live mode communication.
mBlock uses f3 over serial in DEFAULT mode to read sensors and control motors.

This script:
1. Resets CyberPi via RTS toggle
2. Stays in DEFAULT mode (never sends "mode upload")
3. Captures and parses initial f3 frames from CyberPi
4. Responds with f3 f5/f6 handshake
5. Tries systematic f3 commands for sensor reads
"""

import serial
import time
import struct
import sys

SERIAL_PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_serial_f3_default.log"
LOG = []

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

def parse_f3_frames(data):
    """Extract f3...f4 frames from raw data."""
    frames = []
    i = 0
    while i < len(data):
        if data[i] == 0xf3:
            # Find matching f4
            for j in range(i + 1, len(data)):
                if data[j] == 0xf4:
                    frames.append(data[i:j+1])
                    i = j + 1
                    break
            else:
                i += 1
        else:
            i += 1
    return frames

def hex_dump(data):
    return data.hex(' ') if data else "(empty)"

def main():
    log(f"=== CyberPi f3 Serial DEFAULT Mode - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")
    log("CRITICAL: This test stays in DEFAULT mode (no 'mode upload')")
    log("This is how mBlock Live mode communicates with CyberPi.\n")

    # ==========================================
    # PHASE 1: Reset CyberPi
    # ==========================================
    log("=== PHASE 1: Reset CyberPi via RTS ===")
    ser = serial.Serial()
    ser.port = SERIAL_PORT
    ser.baudrate = BAUD
    ser.timeout = 0.1
    ser.dtr = False
    ser.rts = False
    ser.open()
    ser.dtr = False
    ser.rts = False
    time.sleep(0.1)

    # RTS toggle = ESP32 reset
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    log("Reset pulse sent. Waiting for boot (4s)...")
    time.sleep(4.0)

    # ==========================================
    # PHASE 2: Capture ALL boot data
    # ==========================================
    log("\n=== PHASE 2: Capture boot data ===")
    boot_data = b""
    deadline = time.time() + 3.0
    while time.time() < deadline:
        if ser.in_waiting:
            chunk = ser.read(ser.in_waiting)
            boot_data += chunk
            log(f"  RX [{len(chunk)}]: {hex_dump(chunk)}")
        time.sleep(0.05)

    log(f"\nTotal boot data: {len(boot_data)} bytes")
    if boot_data:
        # Try to decode text portions
        text = boot_data.decode('utf-8', errors='replace')
        printable = ''.join(c if c.isprintable() or c in '\n\r\t' else '.' for c in text)
        for line in printable.split('\n'):
            if line.strip():
                log(f"  TXT: {line.strip()}")

        # Parse f3 frames
        frames = parse_f3_frames(boot_data)
        log(f"\n  Found {len(frames)} f3 frames in boot data:")
        for i, frame in enumerate(frames):
            log(f"    [{i}] {hex_dump(frame)}")
            if len(frame) >= 3:
                log(f"         Type: 0x{frame[1]:02x}, Payload: {hex_dump(frame[2:-1])}")

    # ==========================================
    # PHASE 3: f5 Handshake (same as BLE)
    # ==========================================
    log("\n=== PHASE 3: f5 Handshake over Serial ===")

    def send_f3(data, label, wait=1.0):
        """Send f3 frame, return response bytes."""
        ser.reset_input_buffer()
        ser.write(data)
        time.sleep(wait)
        resp = b""
        while ser.in_waiting:
            resp += ser.read(ser.in_waiting)
            time.sleep(0.05)
        return resp

    # The handshake that worked over BLE
    handshake = bytes([0xf3, 0xf5, 0x02, 0x00, 0x08, 0xc0, 0xc8, 0xf4])
    resp = send_f3(handshake, "f5 handshake", 2.0)
    log(f"  TX: {hex_dump(handshake)}")
    log(f"  RX ({len(resp)} bytes): {hex_dump(resp)}")
    if resp:
        frames = parse_f3_frames(resp)
        for f in frames:
            log(f"    Frame: {hex_dump(f)}")
        log("  *** f5 HANDSHAKE RESPONDED OVER SERIAL! ***")
    else:
        log("  No response to f5 handshake")

    # f6 config
    config = bytes([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4])
    resp = send_f3(config, "f6 config", 1.0)
    log(f"\n  TX: {hex_dump(config)}")
    log(f"  RX ({len(resp)} bytes): {hex_dump(resp)}")
    if resp:
        frames = parse_f3_frames(resp)
        for f in frames:
            log(f"    Frame: {hex_dump(f)}")
        log("  *** f6 CONFIG RESPONDED OVER SERIAL! ***")

    # ==========================================
    # PHASE 4: Sensor read commands (f3 protocol)
    # ==========================================
    log("\n=== PHASE 4: Sensor read commands ===")
    log("Trying systematic f3 commands for sensor data...\n")

    # Based on Makeblock protocol knowledge:
    # The old Makeblock protocol (FF 55) used device IDs for sensors.
    # CyberPi's f3 protocol likely has different command structure.
    # Let's try various approaches:

    # Approach 1: Simple command types (0x01-0x20) with minimal payloads
    log("--- Approach 1: Command types 0x01-0x20 ---")
    for cmd in range(0x01, 0x21):
        for payload in [bytes([0x00]), bytes([0x01]), bytes([0x00, 0x00]),
                        bytes([0x01, 0x00]), bytes([0x02, 0x00])]:
            frame = bytes([0xf3, cmd]) + payload + bytes([0xf4])
            resp = send_f3(frame, f"cmd 0x{cmd:02x}", 0.2)
            if resp:
                log(f"  0x{cmd:02x} payload={payload.hex()}: RX {hex_dump(resp)}")
                frames = parse_f3_frames(resp)
                for f in frames:
                    log(f"    Frame: {hex_dump(f)}")

    # Approach 2: High command types (0xf0-0xff) - where f5/f6 live
    log("\n--- Approach 2: Command types 0xf0-0xff ---")
    for cmd in range(0xf0, 0x100):
        if cmd in (0xf5, 0xf6):
            continue  # Already tested
        for payload in [bytes([0x00]), bytes([0x01]), bytes([0x02, 0x00, 0x08]),
                        bytes([0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d])]:
            frame = bytes([0xf3, cmd]) + payload + bytes([0xf4])
            resp = send_f3(frame, f"cmd 0x{cmd:02x}", 0.2)
            if resp:
                log(f"  0x{cmd:02x} payload={payload.hex()}: RX {hex_dump(resp)}")
                frames = parse_f3_frames(resp)
                for f in frames:
                    log(f"    Frame: {hex_dump(f)}")

    # Approach 3: Mid-range commands (0x30-0x50, 0x60-0x80)
    log("\n--- Approach 3: Mid-range commands ---")
    for cmd in [0x30, 0x31, 0x32, 0x40, 0x41, 0x42, 0x50, 0x51,
                0x60, 0x61, 0x62, 0x70, 0x71, 0x72, 0x80, 0x81]:
        frame = bytes([0xf3, cmd, 0x00, 0xf4])
        resp = send_f3(frame, f"cmd 0x{cmd:02x}", 0.2)
        if resp:
            log(f"  0x{cmd:02x}: RX {hex_dump(resp)}")

    # Approach 4: Makeblock device ID style (device_id, port, slot format)
    log("\n--- Approach 4: Device ID style queries ---")
    # CyberPi sensors: gyro=0x06, sound=0x07, light=0x03, etc.
    for device_id in [0x01, 0x02, 0x03, 0x06, 0x07, 0x08, 0x09, 0x0a,
                      0x10, 0x11, 0x12, 0x20, 0x21, 0x30, 0x3c, 0x3d]:
        for cmd in [0x01, 0x02, 0x03, 0x04, 0x05]:
            frame = bytes([0xf3, cmd, device_id, 0x00, 0xf4])
            resp = send_f3(frame, f"dev {device_id:02x} cmd {cmd:02x}", 0.15)
            if resp:
                log(f"  cmd=0x{cmd:02x} dev=0x{device_id:02x}: RX {hex_dump(resp)}")

    # Approach 5: Subscribe-style commands (maybe f3 supports data streaming)
    log("\n--- Approach 5: Subscribe/stream commands ---")
    subscribe_frames = [
        # Maybe a "start streaming" command
        bytes([0xf3, 0xf7, 0x01, 0xf4]),
        bytes([0xf3, 0xf7, 0x01, 0x00, 0xf4]),
        bytes([0xf3, 0xf8, 0x01, 0xf4]),
        bytes([0xf3, 0xf9, 0x01, 0xf4]),
        # Full sweep of common patterns
        bytes([0xf3, 0x0a, 0x01, 0x00, 0xf4]),  # Read type 0x0a
        bytes([0xf3, 0x0b, 0x01, 0x00, 0xf4]),
        bytes([0xf3, 0x0c, 0x01, 0x00, 0xf4]),
        bytes([0xf3, 0x0d, 0x01, 0x00, 0xf4]),
    ]
    for frame in subscribe_frames:
        resp = send_f3(frame, f"subscribe: {hex_dump(frame)}", 0.5)
        if resp:
            log(f"  {hex_dump(frame)}: RX {hex_dump(resp)}")

    # ==========================================
    # PHASE 5: Try second handshake + different f5 parameters
    # ==========================================
    log("\n=== PHASE 5: Re-handshake with different params ===")

    # Maybe the handshake bytes encode what data we want
    # Original: f3 f5 02 00 08 c0 c8 f4
    # Bytes:          ^  ^  ^  ^  ^
    #                 |  |  |  |  checksum?
    #                 |  |  |  flags?
    #                 |  |  data type?
    #                 |  sub-command?
    #                 count/version?

    variants = [
        # Try requesting different "data types"
        bytes([0xf3, 0xf5, 0x02, 0x00, 0x01, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x02, 0x00, 0x02, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x02, 0x00, 0x04, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x02, 0x00, 0x10, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x02, 0x00, 0x20, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x02, 0x00, 0xff, 0xc0, 0xc8, 0xf4]),
        # Try different "sub-commands"
        bytes([0xf3, 0xf5, 0x02, 0x01, 0x08, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x02, 0x02, 0x08, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x02, 0x03, 0x08, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x02, 0x04, 0x08, 0xc0, 0xc8, 0xf4]),
        # Different version bytes
        bytes([0xf3, 0xf5, 0x01, 0x00, 0x08, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x03, 0x00, 0x08, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf5, 0x04, 0x00, 0x08, 0xc0, 0xc8, 0xf4]),
    ]
    for v in variants:
        resp = send_f3(v, f"f5 variant", 0.3)
        if resp:
            log(f"  TX: {hex_dump(v)}")
            log(f"  RX: {hex_dump(resp)}")

    # ==========================================
    # PHASE 6: Full sweep 0x00-0xff (definitive)
    # ==========================================
    log("\n=== PHASE 6: Full command type sweep 0x00-0xff ===")
    responders = {}
    for cmd in range(0x00, 0x100):
        frame = bytes([0xf3, cmd, 0x00, 0xf4])
        resp = send_f3(frame, f"sweep 0x{cmd:02x}", 0.12)
        if resp:
            responders[cmd] = resp
            log(f"  0x{cmd:02x}: RX [{len(resp)}] {hex_dump(resp[:32])}")

    log(f"\n  Responding command types: {len(responders)}")
    for cmd, resp in sorted(responders.items()):
        log(f"    0x{cmd:02x}: {hex_dump(resp[:40])}")

    # ==========================================
    # PHASE 7: Listen for unsolicited data
    # ==========================================
    log("\n=== PHASE 7: Listen for unsolicited data (5s) ===")
    ser.reset_input_buffer()
    all_rx = b""
    deadline = time.time() + 5.0
    while time.time() < deadline:
        if ser.in_waiting:
            chunk = ser.read(ser.in_waiting)
            all_rx += chunk
            log(f"  RX: {hex_dump(chunk)}")
        time.sleep(0.05)
    if not all_rx:
        log("  No unsolicited data")

    # ==========================================
    # PHASE 8: Try raw text commands in default mode
    # ==========================================
    log("\n=== PHASE 8: Text commands in DEFAULT mode ===")
    text_cmds = [
        b"\r\n",
        b"help\r\n",
        b"status\r\n",
        b"version\r\n",
        b"sensor\r\n",
        b"read\r\n",
        b"live\r\n",
        b"mode live\r\n",
    ]
    for cmd in text_cmds:
        ser.reset_input_buffer()
        ser.write(cmd)
        time.sleep(1.0)
        resp = b""
        while ser.in_waiting:
            resp += ser.read(ser.in_waiting)
            time.sleep(0.05)
        if resp:
            log(f"  '{cmd.strip().decode()}': [{len(resp)}] {hex_dump(resp[:64])}")
            text = resp.decode('utf-8', errors='replace')
            printable = ''.join(c if c.isprintable() or c in '\n\r' else '.' for c in text)
            if printable.strip():
                log(f"    TXT: {printable.strip()[:100]}")
        else:
            log(f"  '{cmd.strip().decode()}': no response")

    # ==========================================
    # PHASE 9: Post-text, check f3 still works
    # ==========================================
    log("\n=== PHASE 9: f3 after text commands ===")
    resp = send_f3(handshake, "f5 re-handshake", 1.0)
    log(f"  TX: {hex_dump(handshake)}")
    log(f"  RX ({len(resp)} bytes): {hex_dump(resp)}")

    ser.close()

    # ==========================================
    # SUMMARY
    # ==========================================
    log("\n" + "="*60)
    log("SUMMARY")
    log("="*60)
    log(f"Boot data: {len(boot_data)} bytes")
    log(f"f3 frames in boot: {len(parse_f3_frames(boot_data))}")
    log(f"f5 handshake over serial: {'YES' if any(0xf5 in responders for _ in [1]) else 'Check above'}")
    log(f"Responding command types: {len(responders)}")
    if responders:
        log(f"  Types: {', '.join(f'0x{k:02x}' for k in sorted(responders.keys()))}")
    log(f"Log saved: {LOGFILE}")
    log("="*60)
    save_log()

if __name__ == "__main__":
    main()
