#!/usr/bin/env python3
"""
CyberPi Text Mode Command Probe

In normal mode (NOT upload mode), CyberPi accepts text commands.
We know: 'help' works, 'mode upload' works.
Let's find what OTHER commands exist, especially for sensor data.

Also probes the f3 binary protocol for sensor read commands.
"""

import serial
import time
import struct

PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_textmode.log"
LOG = []

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

def read_all(ser, timeout=2.0):
    end = time.time() + timeout
    buf = b""
    while time.time() < end:
        n = ser.in_waiting
        if n:
            buf += ser.read(n)
        time.sleep(0.01)
    return buf

def send_text(ser, cmd, label=None, timeout=2.0):
    """Send text command, read response."""
    label = label or cmd
    ser.reset_input_buffer()
    ser.write(cmd.encode() + b"\r\n")
    time.sleep(0.3)
    resp = read_all(ser, timeout)
    log(f"\n  [{label}] TX: {cmd!r}")
    if resp:
        log(f"    RX ({len(resp)}): {resp.hex(' ')}")
        text = resp.decode('utf-8', errors='replace')
        for line in text.split('\n'):
            s = line.strip()
            if s:
                log(f"    > {s}")
    else:
        log(f"    RX: (empty)")
    return resp

def send_binary(ser, data, label="", timeout=1.0):
    """Send binary data, read response."""
    ser.reset_input_buffer()
    ser.write(data)
    time.sleep(0.2)
    resp = read_all(ser, timeout)
    log(f"\n  [{label}] TX: {data.hex(' ')}")
    if resp:
        log(f"    RX ({len(resp)}): {resp.hex(' ')}")
        # Try to decode text parts
        text = resp.decode('utf-8', errors='replace')
        if any(c.isalnum() for c in text):
            log(f"    TXT: {text!r}")
    else:
        log(f"    RX: (empty)")
    return resp

def main():
    log(f"=== CyberPi Text Mode Probe - {time.strftime('%Y-%m-%d %H:%M:%S')} ===")

    ser = serial.Serial()
    ser.port = PORT
    ser.baudrate = BAUD
    ser.timeout = 0.01
    ser.dtr = False
    ser.rts = False
    ser.open()
    ser.dtr = False
    ser.rts = False
    time.sleep(0.1)

    # Reset
    log("\n=== PHASE 1: Reset and boot ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    boot = read_all(ser, 2.0)
    log(f"Boot: {len(boot)} bytes")
    if boot:
        text = boot.decode('utf-8', errors='replace')
        for line in text.split('\n'):
            s = line.strip()
            if s and any(c.isalpha() for c in s):
                log(f"  BOOT> {s}")

    # PHASE 2: Text commands in normal mode
    log("\n=== PHASE 2: Text commands (normal mode) ===")

    known_cmds = [
        "help",
        "status",
        "version",
        "info",
        "sensor",
        "sensors",
        "read",
        "read sensor",
        "get ultrasonic",
        "get sound",
        "get light",
        "get gyro",
        "get battery",
        "list",
        "ls",
        "cat",
        "mode",
        "mode run",
        "mode live",
        "mode debug",
        "ping",
        "echo hello",
        "test",
        "repl",
        "python",
        "exec",
        "eval",
        "json",
        "api",
        "?",
    ]

    responses = {}
    for cmd in known_cmds:
        resp = send_text(ser, cmd, timeout=1.5)
        if resp and len(resp) > 0:
            responses[cmd] = resp

    if responses:
        log(f"\n=== Commands that got responses ({len(responses)}): ===")
        for cmd, resp in responses.items():
            log(f"  {cmd!r} -> {len(resp)} bytes")
    else:
        log("\n=== No text commands got responses ===")

    # PHASE 3: f3 binary protocol probing
    log("\n=== PHASE 3: f3 binary protocol ===")
    log("Probing f3 frame types...")

    # Known frames from CyberPi:
    # f3 f5 02 00 08 c0 c8 f4 - sent by CyberPi after mode upload
    # f3 f6 03 00 0d 00 00 0d f4 - sent by CyberPi after mode upload

    # Try various f3 command types
    for cmd_type in range(0x01, 0x20):
        frame = bytes([0xf3, cmd_type, 0x00, 0xf4])
        resp = send_binary(ser, frame, f"f3 {cmd_type:02x} 00", 0.5)
        if resp:
            log(f"    *** GOT RESPONSE for type 0x{cmd_type:02x}! ***")

    # Try with more payload variations
    log("\n  --- Extended f3 probing ---")
    for cmd_type in [0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb]:
        for subcmd in [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x10, 0x20]:
            frame = bytes([0xf3, cmd_type, subcmd, 0xf4])
            resp = send_binary(ser, frame, f"f3 {cmd_type:02x} {subcmd:02x}", 0.3)
            if resp:
                log(f"    *** GOT RESPONSE for f3 {cmd_type:02x} {subcmd:02x}! ***")

    # PHASE 4: Try Makeblock's mblock-link protocol
    # mBlock uses a different protocol for live sensor monitoring
    # The protocol might be: ff 55 LL CMD [data]
    # Or it might be a JSON-based protocol
    log("\n=== PHASE 4: mBlock link protocol probing ===")

    # Try JSON request
    json_cmds = [
        '{"method":"getSensorData"}',
        '{"cmd":"read","sensor":"all"}',
        '{"type":"sensor","action":"read"}',
        '{"protocol":"sensor","cmd":"read_all"}',
    ]
    for jcmd in json_cmds:
        resp = send_text(ser, jcmd, f"JSON: {jcmd[:30]}", 1.0)
        if resp:
            log(f"    *** JSON RESPONSE! ***")

    # PHASE 5: Enter mode upload and probe the f3 protocol there
    log("\n=== PHASE 5: f3 protocol in upload mode ===")
    ser.reset_input_buffer()
    ser.write(b"mode upload\r\n")
    time.sleep(2.0)
    resp = read_all(ser, 2.0)
    log(f"  mode upload response: {len(resp)} bytes")
    if resp:
        log(f"  HEX: {resp.hex(' ')}")

    # Wait for initial f3 frames
    time.sleep(1.0)
    resp = read_all(ser, 2.0)
    if resp:
        log(f"  Additional data: {resp.hex(' ')}")

    # Now try f3 commands in upload mode
    log("\n  --- f3 probing in upload mode ---")
    for cmd_type in range(0x01, 0x20):
        frame = bytes([0xf3, cmd_type, 0x00, 0xf4])
        resp = send_binary(ser, frame, f"upload: f3 {cmd_type:02x}", 0.5)
        if resp:
            log(f"    *** RESPONSE in upload mode for 0x{cmd_type:02x}! ***")

    # Try echoing back the frames CyberPi sent us
    log("\n  --- Echoing CyberPi's own frames ---")
    echo_frames = [
        bytes([0xf3, 0xf5, 0x02, 0x00, 0x08, 0xc0, 0xc8, 0xf4]),
        bytes([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4]),
    ]
    for frame in echo_frames:
        resp = send_binary(ser, frame, f"echo: {frame.hex(' ')}", 1.0)

    # Try f3 with extended type bytes (f5, f6 etc - might be the actual protocol space)
    log("\n  --- Extended f3 types (0xf0-0xff) ---")
    for cmd_type in range(0xf0, 0x100):
        for payload in [b"\x00", b"\x01", b"\x01\x00", b"\x02\x00"]:
            frame = bytes([0xf3, cmd_type]) + payload + bytes([0xf4])
            resp = send_binary(ser, frame, f"f3 {cmd_type:02x} {payload.hex()}", 0.3)
            if resp:
                log(f"    *** RESPONSE for f3 {cmd_type:02x} {payload.hex()}! ***")
                break  # Found a working type, move to next

    # PHASE 6: Try Scratch-link style protocol
    # Makeblock Scratch extensions use a specific binary protocol
    log("\n=== PHASE 6: Scratch/mBlock binary protocol ===")

    # mBlock protocol might use: 0xff 0x55 [len] [cmd] [device] [port] [slot] [data...]
    # This is similar to Makeblock's universal protocol
    # Device IDs: ultrasonic=1, light=3, sound=7, gyro=6, etc.
    scratch_cmds = [
        # Read ultrasonic on port 1
        bytes([0xff, 0x55, 0x04, 0x01, 0x01, 0x01, 0x01]),
        # Read all sensors
        bytes([0xff, 0x55, 0x03, 0x01, 0x00, 0x00]),
        # Get device info
        bytes([0xff, 0x55, 0x02, 0x00, 0x00]),
        # Ping
        bytes([0xff, 0x55, 0x01, 0x00]),
    ]
    for cmd in scratch_cmds:
        resp = send_binary(ser, cmd, f"scratch: {cmd.hex(' ')}", 1.0)
        if resp:
            log(f"    *** SCRATCH PROTOCOL RESPONSE! ***")

    log("\n========================================")
    log("TEXT MODE PROBE COMPLETE")
    log(f"Log: {LOGFILE}")
    log("========================================")

    ser.close()
    save_log()

if __name__ == "__main__":
    main()
