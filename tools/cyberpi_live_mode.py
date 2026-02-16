#!/usr/bin/env python3
"""
CyberPi LIVE MODE Protocol Test

BREAKTHROUGH: mBlock Extension API docs reveal:
  - Upload mode: f3 0d 00 03 f4
  - Live/Debug mode: f3 0d 00 00 f4
  - After entering Live mode, FF 55 protocol is used for sensor reads!

The FF 55 protocol format:
  Request:  ff 55 [length] [index] [action] [module] [data...]
  Response: ff 55 [index] [type] [data...]
  Action 1 = GET (read sensor), Action 2 = RUN (actuator)
  Response types: 1=byte, 2=float(4B LE), 3=short(2B LE), 4=string

Known Makeblock module IDs:
  0x01=ultrasonic, 0x03=light, 0x06=gyro, 0x07=sound,
  0x08=rgbled, 0x0a=dc_motor, 0x0b=servo, 0x22=buzzer
"""

import serial
import time
import struct

SERIAL_PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_live_mode.log"
LOG = []

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

def hex_dump(data):
    return data.hex(' ') if data else "(empty)"

def parse_ff55_response(data):
    """Parse FF 55 response frames from raw data."""
    results = []
    i = 0
    while i < len(data) - 3:
        if data[i] == 0xff and data[i+1] == 0x55:
            idx = data[i+2]
            dtype = data[i+3]
            if dtype == 1 and i + 4 < len(data):  # byte
                results.append((idx, 'byte', data[i+4]))
                i += 5
            elif dtype == 2 and i + 7 < len(data):  # float (4 bytes LE)
                val = struct.unpack('<f', data[i+4:i+8])[0]
                results.append((idx, 'float', val))
                i += 8
            elif dtype == 3 and i + 5 < len(data):  # short (2 bytes LE)
                val = struct.unpack('<H', data[i+4:i+6])[0]
                results.append((idx, 'short', val))
                i += 6
            elif dtype == 4:  # string (null-terminated)
                end = data.find(0x00, i+4)
                if end < 0:
                    end = len(data)
                text = data[i+4:end].decode('utf-8', errors='replace')
                results.append((idx, 'string', text))
                i = end + 1
            else:
                results.append((idx, f'type_{dtype}', data[i+4:min(i+12, len(data))].hex(' ')))
                i += 4
        else:
            i += 1
    return results

def make_ff55_request(index, action, module, port=0, extra=b""):
    """Build an FF 55 request frame."""
    data = bytes([port]) + extra
    length = 2 + len(data)  # action + module + data
    frame = bytes([0xff, 0x55, length, index, action, module]) + data
    return frame

def main():
    log(f"=== CyberPi LIVE MODE Test - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")
    log("Strategy: Enter LIVE mode via f3 0d 00 00 f4, then use FF 55 sensor reads\n")

    # Open serial
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

    # Reset CyberPi
    log("=== Reset CyberPi ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    log("Reset pulse sent. Waiting for boot (5s)...")
    time.sleep(5.0)

    # Drain boot data
    boot = b""
    while ser.in_waiting:
        boot += ser.read(ser.in_waiting)
        time.sleep(0.05)
    log(f"Boot data: {len(boot)} bytes drained")

    def send_and_read(data, label, wait=2.0):
        """Send data, wait, read response."""
        ser.reset_input_buffer()
        ser.write(data)
        time.sleep(wait)
        resp = b""
        while ser.in_waiting:
            resp += ser.read(ser.in_waiting)
            time.sleep(0.05)
        log(f"\n  [{label}] TX: {hex_dump(data)}")
        log(f"  [{label}] RX ({len(resp)} bytes): {hex_dump(resp)}")
        if resp:
            text = resp.decode('utf-8', errors='replace')
            printable = ''.join(c if c.isprintable() or c in '\n\r' else '.' for c in text)
            if printable.strip():
                log(f"  TXT: {printable.strip()[:200]}")
            # Check for FF 55 responses
            ff55_resps = parse_ff55_response(resp)
            if ff55_resps:
                log(f"  FF55 responses: {ff55_resps}")
            # Check for f3 frames
            if 0xf3 in resp:
                log(f"  Contains f3 frames!")
        return resp

    # ==========================================
    # PHASE 1: Enter LIVE/DEBUG mode
    # ==========================================
    log("\n=== PHASE 1: Enter LIVE/DEBUG mode ===")

    # The documented live mode entry command
    live_mode = bytes([0xf3, 0x0d, 0x00, 0x00, 0xf4])
    resp = send_and_read(live_mode, "LIVE mode entry (f3 0d 00 00 f4)", 3.0)

    # Also try the upload mode command for comparison
    # upload_mode = bytes([0xf3, 0x0d, 0x00, 0x03, 0xf4])

    # Try other mode values
    for mode_byte in [0x00, 0x01, 0x02, 0x04, 0x05, 0x0a, 0x0f, 0xff]:
        mode_cmd = bytes([0xf3, 0x0d, 0x00, mode_byte, 0xf4])
        resp = send_and_read(mode_cmd, f"mode 0x{mode_byte:02x}", 0.5)

    # ==========================================
    # PHASE 2: FF 55 sensor reads (after live mode)
    # ==========================================
    log("\n=== PHASE 2: FF 55 sensor read commands ===")

    # Standard Makeblock sensor read requests
    # Format: ff 55 [len] [index] [action=01] [module] [port]
    sensor_cmds = [
        # Known Makeblock module IDs
        (1, "ultrasonic", 0x01, 0x01),      # Ultrasonic sensor, port 1
        (2, "light", 0x03, 0x00),            # Light sensor, port 0
        (3, "gyro_x", 0x06, 0x00),           # Gyro X
        (4, "sound", 0x07, 0x00),            # Sound sensor
        (5, "temperature", 0x02, 0x00),      # Temperature
        (6, "button", 0x16, 0x00),           # Button
        (7, "joystick", 0x05, 0x00),         # Joystick
        # CyberPi-specific IDs (guesses based on device capabilities)
        (8, "brightness", 0x03, 0x06),       # Brightness (light with different port?)
        (9, "loudness", 0x07, 0x06),         # Loudness (sound with different port?)
        (10, "gyro_x_p6", 0x06, 0x06),       # Gyro (port 6?)
        # Try module IDs that CyberPi might use
        (11, "cyberpi_light", 0x3c, 0x00),
        (12, "cyberpi_sound", 0x3d, 0x00),
        (13, "cyberpi_gyro", 0x3e, 0x00),
        (14, "cyberpi_button", 0x3f, 0x00),
    ]

    for idx, name, module, port in sensor_cmds:
        frame = make_ff55_request(idx, 0x01, module, port)  # action=01 = GET
        resp = send_and_read(frame, f"FF55 GET {name} (mod=0x{module:02x} port={port})", 0.5)

    # ==========================================
    # PHASE 3: Maybe need f5 handshake first?
    # ==========================================
    log("\n=== PHASE 3: f5 handshake then FF 55 ===")
    handshake = bytes([0xf3, 0xf5, 0x02, 0x00, 0x08, 0xc0, 0xc8, 0xf4])
    resp = send_and_read(handshake, "f5 handshake", 2.0)

    config = bytes([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4])
    resp = send_and_read(config, "f6 config", 1.0)

    # Now try FF 55 after handshake
    log("\nFF 55 after handshake:")
    for idx, name, module, port in sensor_cmds[:4]:
        frame = make_ff55_request(idx, 0x01, module, port)
        resp = send_and_read(frame, f"FF55 GET {name} post-handshake", 0.5)

    # ==========================================
    # PHASE 4: Reset + Live mode + FF 55 (clean state)
    # ==========================================
    log("\n=== PHASE 4: Fresh reset → Live mode → FF 55 ===")

    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    log("Reset. Waiting 5s...")
    time.sleep(5.0)
    while ser.in_waiting:
        ser.read(ser.in_waiting)
        time.sleep(0.05)

    # Enter live mode FIRST (before any other interaction)
    resp = send_and_read(live_mode, "LIVE mode (fresh)", 3.0)

    # Immediately try FF 55
    for idx, name, module, port in sensor_cmds[:4]:
        frame = make_ff55_request(idx, 0x01, module, port)
        resp = send_and_read(frame, f"FF55 {name} (post-live)", 0.5)

    # ==========================================
    # PHASE 5: Reset + FF 55 directly (no mode switch)
    # ==========================================
    log("\n=== PHASE 5: Fresh reset → FF 55 directly (no mode switch) ===")

    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    log("Reset. Waiting 5s...")
    time.sleep(5.0)
    while ser.in_waiting:
        ser.read(ser.in_waiting)
        time.sleep(0.05)

    # Try FF 55 immediately after boot (default mode)
    for idx, name, module, port in sensor_cmds[:6]:
        frame = make_ff55_request(idx, 0x01, module, port)
        resp = send_and_read(frame, f"FF55 {name} (default mode)", 0.5)

    # ==========================================
    # PHASE 6: Sweep module IDs 0x00-0x7f with FF 55
    # ==========================================
    log("\n=== PHASE 6: FF 55 module ID sweep ===")
    responders = {}
    for module in range(0, 128):
        frame = make_ff55_request(1, 0x01, module, 0)  # GET, port 0
        ser.reset_input_buffer()
        ser.write(frame)
        time.sleep(0.15)
        resp = b""
        while ser.in_waiting:
            resp += ser.read(ser.in_waiting)
            time.sleep(0.03)
        if resp:
            responders[module] = resp
            log(f"  Module 0x{module:02x}: RX [{len(resp)}] {hex_dump(resp[:32])}")
            ff55 = parse_ff55_response(resp)
            if ff55:
                log(f"    FF55 parsed: {ff55}")

    log(f"\n  Responding modules: {len(responders)}")
    if responders:
        for mod, resp in sorted(responders.items()):
            log(f"    0x{mod:02x}: {hex_dump(resp[:40])}")

    # ==========================================
    # PHASE 7: Combined approach - live mode then sweep
    # ==========================================
    if not responders:
        log("\n=== PHASE 7: Live mode entry then FF 55 sweep ===")

        # Reset again
        ser.rts = True
        time.sleep(0.1)
        ser.rts = False
        time.sleep(5.0)
        while ser.in_waiting:
            ser.read(ser.in_waiting)
            time.sleep(0.05)

        # Enter live mode
        ser.write(live_mode)
        time.sleep(2.0)
        while ser.in_waiting:
            resp = ser.read(ser.in_waiting)
            log(f"  Live mode response: {hex_dump(resp)}")
            time.sleep(0.05)

        # Sweep FF 55
        log("  Sweeping modules after live mode...")
        for module in range(0, 128):
            frame = make_ff55_request(1, 0x01, module, 0)
            ser.reset_input_buffer()
            ser.write(frame)
            time.sleep(0.15)
            resp = b""
            while ser.in_waiting:
                resp += ser.read(ser.in_waiting)
                time.sleep(0.03)
            if resp:
                log(f"  Module 0x{module:02x}: RX [{len(resp)}] {hex_dump(resp[:32])}")
                ff55 = parse_ff55_response(resp)
                if ff55:
                    log(f"    FF55 parsed: {ff55}")

    # ==========================================
    # PHASE 8: Extended listen for streaming data
    # ==========================================
    log("\n=== PHASE 8: Extended listen for streaming data (5s) ===")
    ser.reset_input_buffer()
    all_data = b""
    deadline = time.time() + 5.0
    while time.time() < deadline:
        if ser.in_waiting:
            chunk = ser.read(ser.in_waiting)
            all_data += chunk
            log(f"  Stream: [{len(chunk)}] {hex_dump(chunk)}")
        time.sleep(0.05)
    if not all_data:
        log("  No streaming data")

    ser.close()

    # Summary
    log("\n" + "="*60)
    log("LIVE MODE TEST SUMMARY")
    log("="*60)
    all_text = "\n".join(LOG)
    if "FF55 responses:" in all_text or "FF55 parsed:" in all_text:
        log("SUCCESS: FF 55 protocol responses received!")
    elif "f3 frames" in all_text:
        log("PARTIAL: Got f3 frames but no FF 55 responses")
    elif any("RX (" in l and "0 bytes" not in l for l in LOG):
        log("PARTIAL: Some responses received (check details above)")
    else:
        log("NEGATIVE: No sensor data received")

    log(f"\nLog: {LOGFILE}")
    log("="*60)
    save_log()

if __name__ == "__main__":
    main()
