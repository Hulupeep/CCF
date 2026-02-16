#!/usr/bin/env python3
"""
CyberPi REPL Probe v2

Tries the exact sequence that worked in the previous session:
  mode upload -> Ctrl+A -> Ctrl+B -> got >>> prompt

Then tests every possible way to read output back.
"""

import serial
import time
import sys

PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_probe2.log"

def log(msg, data=None):
    print(msg)
    with open(LOGFILE, "a") as f:
        f.write(f"{msg}\n")
        if data:
            f.write(f"  HEX: {data.hex(' ')}\n")

def read_all(ser, timeout=2.0):
    end = time.time() + timeout
    buf = b""
    while time.time() < end:
        n = ser.in_waiting
        if n > 0:
            buf += ser.read(n)
        else:
            time.sleep(0.02)
    return buf

def show(label, data):
    log(f"\n[{label}] ({len(data)} bytes)")
    if data:
        log(f"  HEX: {data[:200].hex(' ')}", data)
        text = data.decode('utf-8', errors='replace')
        for line in text.split('\n'):
            s = line.strip()
            if s:
                log(f"  > {s}")
    else:
        log("  (empty)")

def main():
    with open(LOGFILE, "w") as f:
        f.write(f"=== CyberPi REPL Probe v2 - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n\n")

    log(f"Opening {PORT}...")
    ser = serial.Serial()
    ser.port = PORT
    ser.baudrate = BAUD
    ser.timeout = 0.1
    ser.dtr = False
    ser.rts = False
    ser.open()
    ser.dtr = False
    ser.rts = False
    time.sleep(0.1)

    # Reset
    log("\n=== RESET ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False

    # Wait for full boot
    boot = read_all(ser, 5.0)
    show("BOOT", boot)

    # Try "mode upload"
    log("\n=== MODE UPLOAD ===")
    ser.reset_input_buffer()
    ser.write(b"mode upload\r\n")
    time.sleep(0.5)
    resp = read_all(ser, 3.0)
    show("mode upload response", resp)

    # Now the exact sequence: Ctrl+A then Ctrl+B
    log("\n=== CTRL+A then CTRL+B ===")
    ser.reset_input_buffer()
    ser.write(b"\x01")  # Ctrl+A
    time.sleep(0.5)
    resp_a = read_all(ser, 2.0)
    show("After Ctrl+A", resp_a)

    ser.write(b"\x02")  # Ctrl+B
    time.sleep(0.5)
    resp_b = read_all(ser, 3.0)
    show("After Ctrl+B", resp_b)

    got_prompt = b">>>" in resp_b or b">>>" in resp_a

    if not got_prompt:
        # Maybe we need to try harder - send multiple Ctrl+C then Ctrl+A/B
        log("\n=== RETRY: Ctrl+C x3, then Ctrl+A, Ctrl+B ===")
        ser.reset_input_buffer()
        ser.write(b"\x03\x03\x03")  # 3x Ctrl+C
        time.sleep(0.5)
        resp = read_all(ser, 1.0)
        show("After 3x Ctrl+C", resp)

        ser.write(b"\x01")  # Ctrl+A
        time.sleep(0.3)
        ser.write(b"\x02")  # Ctrl+B
        time.sleep(0.5)
        resp = read_all(ser, 3.0)
        show("After Ctrl+A Ctrl+B retry", resp)
        got_prompt = b">>>" in resp

    if not got_prompt:
        # Try sending just \r\n to see if we get any prompt
        log("\n=== RETRY: Just send Enter ===")
        ser.reset_input_buffer()
        ser.write(b"\r\n")
        time.sleep(0.3)
        resp = read_all(ser, 2.0)
        show("After Enter", resp)
        got_prompt = b">>>" in resp

    if not got_prompt:
        # Maybe mode upload takes longer, or we need "mode upload\n" (no \r)
        log("\n=== RETRY: Full reset + mode upload (no \\r) ===")
        ser.rts = True
        time.sleep(0.1)
        ser.rts = False
        boot2 = read_all(ser, 5.0)
        show("BOOT2", boot2)

        ser.reset_input_buffer()
        ser.write(b"mode upload\n")
        time.sleep(1.0)
        resp = read_all(ser, 3.0)
        show("mode upload (no \\r)", resp)

        ser.write(b"\x01\x02")  # Ctrl+A Ctrl+B together
        time.sleep(1.0)
        resp = read_all(ser, 3.0)
        show("Ctrl+A+B together", resp)
        got_prompt = b">>>" in resp

    if not got_prompt:
        # One more try: maybe the CyberPi is already in some state
        # Try sending "help" to see if normal text mode works
        log("\n=== TEST: Send 'help' in text mode ===")
        ser.reset_input_buffer()
        ser.write(b"help\r\n")
        time.sleep(0.5)
        resp = read_all(ser, 2.0)
        show("help response", resp)

        # Try "reboot" command
        ser.reset_input_buffer()
        ser.write(b"reboot\r\n")
        time.sleep(3.0)
        resp = read_all(ser, 3.0)
        show("reboot response", resp)

        # After reboot try the sequence again
        ser.reset_input_buffer()
        ser.write(b"mode upload\r\n")
        time.sleep(2.0)
        resp = read_all(ser, 2.0)
        show("mode upload after reboot", resp)

        ser.write(b"\x01")
        time.sleep(0.5)
        ser.write(b"\x02")
        time.sleep(1.0)
        resp = read_all(ser, 3.0)
        show("Ctrl+A Ctrl+B after reboot", resp)
        got_prompt = b">>>" in resp

    if got_prompt:
        log("\n*** GOT PROMPT! Testing output... ***")

        # Test 1: Simple print
        ser.reset_input_buffer()
        ser.write(b"print('HELLO_CYBERPI')\r\n")
        time.sleep(0.5)
        resp = read_all(ser, 2.0)
        show("print test", resp)

        if b"HELLO_CYBERPI" in resp:
            log("\n*** PRINT OUTPUT WORKS IN NORMAL REPL! ***")
        else:
            log("  print() output NOT visible in response")

            # Test 2: Check if output is in f3 frames
            if b"\xf3" in resp:
                log("  Detected f3 framing - parsing...")
                parse_f3_frames(resp)

        # Test 3: Check what sys.stdout is
        ser.reset_input_buffer()
        ser.write(b"import sys; print(type(sys.stdout))\r\n")
        time.sleep(0.5)
        resp = read_all(ser, 2.0)
        show("sys.stdout type", resp)

        # Test 4: Try repr trick - the REPL echoes expression results
        ser.reset_input_buffer()
        ser.write(b"'EXPR_' + 'TEST'\r\n")
        time.sleep(0.5)
        resp = read_all(ser, 2.0)
        show("expression eval test", resp)

        # Test 5: Read back filesystem
        ser.reset_input_buffer()
        ser.write(b"import os; print(os.listdir('/'))\r\n")
        time.sleep(0.5)
        resp = read_all(ser, 2.0)
        show("os.listdir test", resp)

    else:
        log("\n*** COULD NOT GET REPL PROMPT ***")
        log("  The CyberPi may need manual interaction or a different entry sequence")

        # Last resort: dump raw bytes from whatever state we're in
        log("\n=== FINAL: Raw byte dump for 5 seconds ===")
        ser.reset_input_buffer()
        # Send various probe bytes
        for probe in [b"\r\n", b"\x03", b"\x01", b"\x04", b"help\r\n"]:
            ser.write(probe)
            time.sleep(0.2)

        resp = read_all(ser, 5.0)
        show("Final raw dump", resp)

    log("\n========================================")
    log("PROBE COMPLETE")
    log(f"Full log: {LOGFILE}")
    log("========================================")

    ser.close()

def parse_f3_frames(data):
    idx = 0
    num = 0
    while idx < len(data):
        if data[idx] == 0xf3:
            end = data.find(b"\xf4", idx + 1)
            if end != -1:
                frame = data[idx:end+1]
                num += 1
                log(f"  Frame {num}: type=0x{frame[1]:02x} len={len(frame)}")
                log(f"    HEX: {frame.hex(' ')}")
                if len(frame) > 2:
                    payload = frame[2:-1]
                    log(f"    Payload text: {payload.decode('utf-8', errors='replace')}")
                idx = end + 1
            else:
                break
        else:
            idx += 1

if __name__ == "__main__":
    main()
