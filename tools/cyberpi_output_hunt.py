#!/usr/bin/env python3
"""
CyberPi Output Hunt

We know:
  - REPL prompt works (>>>)
  - Commands execute (LED flash confirmed)
  - But command OUTPUT returns 0 bytes

This script hunts for where the output goes:
  1. Sends commands with very long read windows
  2. Reads in tight loop looking for ANY bytes
  3. Tests if output is in f3 frames
  4. Tests if prompt (>>>) repeats after commands
  5. Tries LED flash to confirm execution
"""

import serial
import time
import sys
import threading

PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_output_hunt.log"
LOG = []

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

def setup_repl(ser):
    """Reset and get to REPL prompt."""
    # Reset
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(4.0)
    ser.reset_input_buffer()

    # Enter upload mode
    ser.write(b"mode upload\r\n")
    time.sleep(2.0)
    ser.reset_input_buffer()

    # Ctrl+A to enter REPL (this gave us >>> before)
    ser.write(b"\x01")
    time.sleep(2.0)

    buf = b""
    end = time.time() + 3.0
    while time.time() < end:
        n = ser.in_waiting
        if n:
            buf += ser.read(n)
        time.sleep(0.01)

    if b">>>" in buf:
        log("REPL prompt obtained!")
        return True
    else:
        log(f"No REPL prompt. Got {len(buf)} bytes: {buf[:100]!r}")
        return False

def continuous_read(ser, duration=5.0):
    """Read bytes continuously, logging timestamps."""
    chunks = []
    start = time.time()
    end = start + duration
    while time.time() < end:
        n = ser.in_waiting
        if n:
            data = ser.read(n)
            elapsed = time.time() - start
            chunks.append((elapsed, data))
        time.sleep(0.005)  # 5ms poll
    return chunks

def main():
    log(f"=== CyberPi Output Hunt - {time.strftime('%Y-%m-%d %H:%M:%S')} ===")

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

    if not setup_repl(ser):
        log("FATAL: Could not get REPL prompt")
        ser.close()
        save_log()
        return

    ser.reset_input_buffer()

    # TEST 1: Send bare Enter, see if >>> repeats
    log("\n=== TEST 1: Bare Enter - does >>> repeat? ===")
    ser.write(b"\r\n")
    chunks = continuous_read(ser, 3.0)
    total = b"".join(c[1] for c in chunks)
    log(f"  Received {len(total)} bytes")
    if total:
        log(f"  HEX: {total.hex(' ')}")
        log(f"  TXT: {total.decode('utf-8', errors='replace')!r}")
        if b">>>" in total:
            log("  >>> PROMPT REPEATS!")
        if b"\xf3" in total:
            log("  f3 framing detected!")
            show_frames(total)
    else:
        log("  Nothing received. REPL might be one-shot.")

    # TEST 2: Send simple math (2+2), read for 5 seconds
    log("\n=== TEST 2: Simple expression (2+2) ===")
    ser.reset_input_buffer()
    ser.write(b"2+2\r\n")
    chunks = continuous_read(ser, 5.0)
    total = b"".join(c[1] for c in chunks)
    log(f"  Received {len(total)} bytes in {len(chunks)} chunks")
    for elapsed, data in chunks:
        log(f"    +{elapsed:.3f}s: {data.hex(' ')} | {data.decode('utf-8', errors='replace')!r}")
    if b"4" in total:
        log("  *** FOUND '4' IN OUTPUT! ***")

    # TEST 3: LED flash (confirm execution)
    log("\n=== TEST 3: LED flash (execution proof) ===")
    ser.reset_input_buffer()
    ser.write(b"import cyberpi; cyberpi.led.on(255, 0, 0)\r\n")
    chunks = continuous_read(ser, 3.0)
    total = b"".join(c[1] for c in chunks)
    log(f"  Received {len(total)} bytes")
    if total:
        log(f"  HEX: {total.hex(' ')}")
    log("  (Check: is the LED red?)")

    time.sleep(1.0)
    ser.write(b"cyberpi.led.off()\r\n")
    time.sleep(1.0)

    # TEST 4: Try raw REPL mode properly
    # Raw REPL: stay at Ctrl+A, DON'T send Ctrl+B
    log("\n=== TEST 4: True raw REPL (Ctrl+C, Ctrl+A, NO Ctrl+B) ===")
    ser.reset_input_buffer()
    ser.write(b"\x03")  # Ctrl+C to interrupt
    time.sleep(0.3)
    ser.write(b"\x03")  # Again
    time.sleep(0.3)

    # Read any interrupt response
    resp = ser.read(1000)
    log(f"  After Ctrl+C: {resp!r}")

    ser.write(b"\x01")  # Ctrl+A - enter raw REPL
    time.sleep(0.5)
    chunks = continuous_read(ser, 2.0)
    total = b"".join(c[1] for c in chunks)
    log(f"  After Ctrl+A: {len(total)} bytes")
    if total:
        log(f"  HEX: {total.hex(' ')}")
        log(f"  TXT: {total.decode('utf-8', errors='replace')!r}")

    if b"raw REPL" in total or b">" in total:
        log("  Raw REPL mode detected!")

        # In raw REPL: send code + Ctrl+D
        # Expected response: OK<stdout>\x04<stderr>\x04
        log("\n=== TEST 4b: Raw REPL print test ===")
        ser.reset_input_buffer()
        ser.write(b"print('RAW_HELLO')\x04")
        chunks = continuous_read(ser, 5.0)
        total = b"".join(c[1] for c in chunks)
        log(f"  Received {len(total)} bytes")
        for elapsed, data in chunks:
            log(f"    +{elapsed:.3f}s: {data.hex(' ')} | {data.decode('utf-8', errors='replace')!r}")

        if b"RAW_HELLO" in total:
            log("  *** RAW REPL OUTPUT WORKS! ***")
        if b"OK" in total:
            log("  Got OK response marker")

    # TEST 5: Try paste mode (Ctrl+E) from normal REPL
    log("\n=== TEST 5: Paste mode (Ctrl+E from normal REPL) ===")
    ser.reset_input_buffer()
    ser.write(b"\x02")  # Ctrl+B back to normal REPL
    time.sleep(0.5)
    resp = ser.read(1000)
    log(f"  After Ctrl+B: {resp!r}")

    ser.write(b"\x05")  # Ctrl+E - paste mode
    time.sleep(0.5)
    chunks = continuous_read(ser, 2.0)
    total = b"".join(c[1] for c in chunks)
    log(f"  After Ctrl+E: {len(total)} bytes")
    if total:
        log(f"  TXT: {total.decode('utf-8', errors='replace')!r}")

    if b"paste" in total.lower() or b"===" in total:
        log("  Paste mode detected!")
        ser.write(b"print('PASTE_TEST')\r\n")
        ser.write(b"\x04")  # Ctrl+D to execute
        chunks = continuous_read(ser, 3.0)
        total = b"".join(c[1] for c in chunks)
        log(f"  Paste output: {len(total)} bytes")
        if total:
            log(f"  TXT: {total.decode('utf-8', errors='replace')!r}")

    # TEST 6: Try the CyberPiOS f3 protocol to request output
    log("\n=== TEST 6: f3 protocol probing ===")
    ser.reset_input_buffer()
    # Try sending f3 f5 (same type as what CyberPi sent us)
    # with a "request output" style payload
    for probe_type in [0xf5, 0xf6, 0xf7, 0xf8]:
        ser.write(bytes([0xf3, probe_type, 0x00, 0xf4]))
        time.sleep(0.3)
        resp = ser.read(1000)
        if resp:
            log(f"  f3 {probe_type:02x}: {resp.hex(' ')}")
            if len(resp) > 4:
                log(f"    TXT: {resp.decode('utf-8', errors='replace')!r}")

    # TEST 7: Exhaustive - read for 10 seconds after a command
    log("\n=== TEST 7: Extended read (10s) after print command ===")
    # Re-enter REPL
    ser.reset_input_buffer()
    ser.write(b"\x03\x03")
    time.sleep(0.3)
    ser.write(b"\x01")  # Ctrl+A
    time.sleep(2.0)
    resp = ser.read(2000)
    if b">>>" in resp or b">" in resp:
        log("  Back in REPL")
        ser.reset_input_buffer()
        ser.write(b"print('EXTENDED_TEST_98765')\r\n")
        log("  Command sent, reading for 10 seconds...")
        chunks = continuous_read(ser, 10.0)
        total = b"".join(c[1] for c in chunks)
        log(f"  Total received: {len(total)} bytes in {len(chunks)} chunks")
        for elapsed, data in chunks:
            log(f"    +{elapsed:.3f}s: {data.hex(' ')} | {data.decode('utf-8', errors='replace')!r}")

        if b"EXTENDED_TEST" in total:
            log("  *** FOUND OUTPUT WITH EXTENDED WAIT! ***")
        elif b"\xf3" in total:
            log("  f3 frames found!")
            show_frames(total)

    log("\n========================================")
    log("OUTPUT HUNT COMPLETE")
    log(f"Log: {LOGFILE}")
    log("========================================")

    ser.close()
    save_log()

def show_frames(data):
    idx = 0
    num = 0
    while idx < len(data):
        if data[idx] == 0xf3:
            end = data.find(b"\xf4", idx + 1)
            if end != -1:
                frame = data[idx:end+1]
                num += 1
                log(f"    Frame {num}: type=0x{frame[1]:02x} ({frame.hex(' ')})")
                if len(frame) > 3:
                    payload = frame[2:-1]
                    log(f"      Payload: {payload.hex(' ')} | {payload.decode('utf-8', errors='replace')!r}")
                idx = end + 1
            else:
                break
        else:
            idx += 1

if __name__ == "__main__":
    main()
