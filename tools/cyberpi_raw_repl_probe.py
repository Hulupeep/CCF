#!/usr/bin/env python3
"""
CyberPi Raw REPL Probe

Tests whether MicroPython raw REPL mode allows us to read output back
from the CyberPi over serial. CyberPiOS captures stdout in normal mode,
but raw REPL uses a different framing (OK + stdout + \x04 + stderr + \x04>)
that may bypass the capture.

Strategy:
  1. Open serial with DTR=False, RTS=False (prevents ESP32 reset)
  2. Toggle RTS to cleanly reset
  3. Enter upload mode via "mode upload"
  4. Enter raw REPL via Ctrl+A (WITHOUT Ctrl+B)
  5. Send test code + Ctrl+D
  6. Look for: OK + output + \x04 + \x04>

If raw REPL works, we have our communication channel for the bridge.
"""

import serial
import time
import sys
import os

PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_probe.log"

def log(msg, data=None):
    """Print and log to file."""
    print(msg)
    with open(LOGFILE, "a") as f:
        f.write(f"{msg}\n")
        if data:
            f.write(f"  HEX: {data.hex(' ')}\n")
            try:
                f.write(f"  TXT: {data.decode('utf-8', errors='replace')}\n")
            except:
                pass

def read_all(ser, timeout=2.0):
    """Read all available bytes within timeout."""
    end = time.time() + timeout
    buf = b""
    while time.time() < end:
        n = ser.in_waiting
        if n > 0:
            chunk = ser.read(n)
            buf += chunk
        else:
            time.sleep(0.05)
    return buf

def send_and_read(ser, data, label="", timeout=2.0):
    """Send data and read response."""
    if isinstance(data, str):
        data = data.encode()
    ser.write(data)
    time.sleep(0.1)
    resp = read_all(ser, timeout)
    if label:
        log(f"\n--- {label} ---")
        log(f"  TX ({len(data)}): {data!r}")
        log(f"  RX ({len(resp)}):", resp)
        if resp:
            # Show printable text
            text = resp.decode('utf-8', errors='replace')
            for line in text.split('\n'):
                stripped = line.strip()
                if stripped:
                    log(f"  > {stripped}")
    return resp

def main():
    # Clear log
    with open(LOGFILE, "w") as f:
        f.write(f"=== CyberPi Raw REPL Probe - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n\n")

    log(f"Opening {PORT} at {BAUD} baud...")

    # Open with DTR/RTS control disabled
    ser = serial.Serial()
    ser.port = PORT
    ser.baudrate = BAUD
    ser.timeout = 0.1
    ser.dtr = False
    ser.rts = False
    ser.open()

    # Ensure DTR/RTS are off
    ser.dtr = False
    ser.rts = False
    time.sleep(0.1)

    log("Port opened. DTR=False, RTS=False")

    # Reset via RTS toggle
    log("\n=== PHASE 1: Reset CyberPi ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(2.0)  # Wait for boot

    boot = read_all(ser, 3.0)
    log("Boot output:", boot)
    if boot:
        text = boot.decode('utf-8', errors='replace')
        for line in text.split('\n'):
            stripped = line.strip()
            if stripped:
                log(f"  BOOT> {stripped}")

    # Flush
    ser.reset_input_buffer()

    # Phase 2: Enter upload mode
    log("\n=== PHASE 2: Enter upload mode ===")
    resp = send_and_read(ser, "mode upload\r\n", "mode upload", 3.0)

    time.sleep(1.0)
    ser.reset_input_buffer()

    # Phase 3: Try RAW REPL (Ctrl+A only, NO Ctrl+B)
    log("\n=== PHASE 3: Raw REPL (Ctrl+A only) ===")
    resp = send_and_read(ser, b"\x01", "Ctrl+A (raw REPL)", 3.0)

    # Check for raw REPL prompt
    raw_repl_active = b"raw REPL" in resp or b">" in resp
    log(f"  Raw REPL detected: {raw_repl_active}")

    if not raw_repl_active:
        # Maybe we need to send Ctrl+A twice or wait
        log("  Retrying Ctrl+A...")
        resp2 = send_and_read(ser, b"\x01", "Ctrl+A retry", 2.0)
        raw_repl_active = b"raw REPL" in resp2 or b">" in resp2
        resp = resp + resp2

    # Phase 4: Test raw REPL with simple print
    log("\n=== PHASE 4: Raw REPL test - print('PROBE_OK') ===")
    # In raw REPL: send code + Ctrl+D (\x04)
    test_code = b"print('PROBE_OK_12345')\x04"
    resp = send_and_read(ser, test_code, "raw REPL print test", 3.0)

    if b"PROBE_OK_12345" in resp:
        log("\n*** SUCCESS: Raw REPL output received! ***")
        log("*** We can read print() output via raw REPL! ***")
    elif b"OK" in resp:
        log("\n*** PARTIAL: Got OK but no print output ***")
        log("*** Output might be in f3 framing ***")
    else:
        log("\n*** Raw REPL print test: no recognizable output ***")

    # Check for f3-framed output
    if b"\xf3" in resp:
        log("  Found f3 bytes - CyberPiOS protocol framing detected")
        # Try to extract f3 frames
        idx = 0
        frame_num = 0
        while idx < len(resp):
            if resp[idx] == 0xf3:
                # Find f4 terminator
                end_idx = resp.find(b"\xf4", idx + 1)
                if end_idx != -1:
                    frame = resp[idx:end_idx+1]
                    frame_num += 1
                    log(f"  Frame {frame_num}: {frame.hex(' ')}")
                    # Payload is between f3 XX and f4
                    if len(frame) > 2:
                        payload = frame[2:-1]  # skip f3, type byte, and f4
                        log(f"    Payload: {payload!r}")
                    idx = end_idx + 1
                else:
                    break
            else:
                idx += 1

    # Phase 5: Try normal REPL too (Ctrl+B) and test print there
    log("\n=== PHASE 5: Switch to normal REPL (Ctrl+B) ===")
    resp = send_and_read(ser, b"\x02", "Ctrl+B (normal REPL)", 3.0)

    if b">>>" in resp:
        log("  Normal REPL prompt detected")

        log("\n=== PHASE 6: Normal REPL print test ===")
        resp = send_and_read(ser, b"print('NORMAL_OK_67890')\r\n", "normal REPL print", 3.0)

        if b"NORMAL_OK_67890" in resp:
            log("\n*** SUCCESS: Normal REPL output received! ***")
        elif b"\xf3" in resp:
            log("  Output wrapped in f3 frames")
        else:
            log("  No print output detected in normal REPL either")

    # Phase 7: Test sensor read (if REPL works)
    log("\n=== PHASE 7: Try sensor read ===")
    if b">>>" in resp:
        resp = send_and_read(ser,
            b"import cyberpi; print('SND:' + str(cyberpi.get_loudness()))\r\n",
            "sensor read test", 3.0)
        if b"SND:" in resp:
            log("\n*** SENSOR DATA RECEIVED! ***")

    # Phase 8: Try machine.UART direct write
    log("\n=== PHASE 8: machine.UART direct write ===")
    # Go back to raw REPL for multi-line
    ser.reset_input_buffer()
    send_and_read(ser, b"\x01", "back to raw REPL", 2.0)

    uart_code = (
        b"import machine\n"
        b"u = machine.UART(0, 115200)\n"
        b"u.write(b'UART_DIRECT_OK\\n')\n"
        b"\x04"
    )
    resp = send_and_read(ser, uart_code, "UART direct write", 3.0)
    if b"UART_DIRECT_OK" in resp:
        log("\n*** SUCCESS: machine.UART direct write works! ***")

    # Phase 9: Try sys.stdout.buffer.write
    log("\n=== PHASE 9: sys.stdout.buffer.write ===")
    stdout_code = (
        b"import sys\n"
        b"sys.stdout.buffer.write(b'STDOUT_BUF_OK\\n')\n"
        b"\x04"
    )
    resp = send_and_read(ser, stdout_code, "sys.stdout.buffer.write", 3.0)
    if b"STDOUT_BUF_OK" in resp:
        log("\n*** SUCCESS: sys.stdout.buffer.write works! ***")

    # Summary
    log("\n\n========================================")
    log("PROBE COMPLETE - Results in " + LOGFILE)
    log("========================================")

    ser.close()
    log("Port closed.")

if __name__ == "__main__":
    main()
