#!/usr/bin/env python3
"""
CyberPi Multi-Strategy Probe

Strategy 1: Check if REPL output is at a different baud rate
Strategy 2: Try to re-enable WiFi after mode upload
Strategy 3: Try NOT using mode upload - use 'status' command instead
Strategy 4: Send code in normal mode (before mode upload)
Strategy 5: Use file write + soft reset (no RTS toggle) to run main.py
"""

import serial
import time

PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_multistrat.log"

def log(msg):
    print(msg)
    with open(LOGFILE, "a") as f:
        f.write(f"{msg}\n")

def read_all(ser, timeout=2.0):
    end = time.time() + timeout
    buf = b""
    while time.time() < end:
        n = ser.in_waiting
        if n:
            buf += ser.read(n)
        time.sleep(0.01)
    return buf

def show(label, data):
    log(f"\n  [{label}] {len(data)} bytes")
    if data:
        log(f"    HEX: {data[:100].hex(' ')}")
        text = data.decode('utf-8', errors='replace')
        for line in text.split('\n'):
            s = line.strip()
            if s:
                log(f"    > {s}")
    else:
        log(f"    (empty)")

def enter_repl(ser):
    """Enter REPL and return True if >>> prompt received."""
    ser.reset_input_buffer()
    ser.write(b"mode upload\r\n")
    time.sleep(2.0)
    ser.reset_input_buffer()
    ser.write(b"\x01")
    time.sleep(2.0)
    resp = read_all(ser, 3.0)
    return b">>>" in resp, resp

def main():
    with open(LOGFILE, "w") as f:
        f.write(f"=== Multi-Strategy Probe - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n\n")

    ser = serial.Serial()
    ser.port = PORT
    ser.baudrate = BAUD
    ser.timeout = 0.01
    ser.dtr = False
    ser.rts = False
    ser.open()
    ser.dtr = False
    ser.rts = False

    # Reset
    log("=== Resetting CyberPi ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    ser.reset_input_buffer()

    # Strategy 1: Baud rate sweep during REPL
    log("\n=== STRATEGY 1: Baud rate sweep ===")
    log("Enter REPL at 115200, send print(), then check other bauds")

    ok, _ = enter_repl(ser)
    if ok:
        # Send a repeating marker
        ser.write(b"exec('[print(chr(65+i)*40) for i in range(26)]')\r\n")
        time.sleep(1.0)
        ser.close()

        # Now reopen at different baud rates and check for data
        bauds = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600]
        for baud in bauds:
            try:
                s = serial.Serial()
                s.port = PORT
                s.baudrate = baud
                s.timeout = 0.01
                s.dtr = False
                s.rts = False
                s.open()
                s.dtr = False
                s.rts = False
                time.sleep(0.2)
                data = read_all(s, 1.0)
                if data and len(data) > 5:
                    text = data.decode('utf-8', errors='replace')
                    printable = sum(1 for c in text if c.isprintable())
                    ratio = printable / len(text) if text else 0
                    log(f"  {baud}: {len(data)} bytes, {ratio:.0%} printable")
                    if ratio > 0.5:
                        log(f"    *** READABLE DATA at {baud}! ***")
                        log(f"    {text[:100]!r}")
                else:
                    log(f"  {baud}: {len(data)} bytes")
                s.close()
            except Exception as e:
                log(f"  {baud}: error: {e}")

        # Reopen at original baud
        ser = serial.Serial()
        ser.port = PORT
        ser.baudrate = BAUD
        ser.timeout = 0.01
        ser.dtr = False
        ser.rts = False
        ser.open()
        ser.dtr = False
        ser.rts = False
    else:
        log("  Could not enter REPL for baud sweep")

    # Strategy 2: Re-enable WiFi in REPL
    log("\n=== STRATEGY 2: Re-enable WiFi in REPL ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    ser.reset_input_buffer()

    ok, _ = enter_repl(ser)
    if ok:
        # First flash LED to confirm execution
        ser.write(b"import cyberpi; cyberpi.led.on(255, 0, 255)\r\n")
        time.sleep(0.5)

        # Try to force WiFi back on
        wifi_cmds = [
            # Standard MicroPython WiFi init
            "import network",
            "w = network.WLAN(network.AP_IF)",
            "w.active(True)",
            "w.config(essid='mBot2Test')",
            # Green LED = WiFi commands sent
            "cyberpi.led.on(0, 255, 0)",
        ]
        for cmd in wifi_cmds:
            ser.write(cmd.encode() + b"\r\n")
            time.sleep(0.5)
            # Check for any response
            resp = read_all(ser, 0.3)
            if resp:
                show(f"WiFi cmd: {cmd[:20]}", resp)

        # Check for AP
        log("  WiFi commands sent. Checking for AP in 5 seconds...")
        time.sleep(5.0)

    # Strategy 3: Use 'status' instead of 'mode upload'
    log("\n=== STRATEGY 3: 'status' command instead of 'mode upload' ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    ser.reset_input_buffer()

    # DON'T use mode upload. Use status.
    ser.write(b"status\r\n")
    time.sleep(3.0)
    resp = read_all(ser, 3.0)
    show("status response", resp)

    if b">>>" in resp:
        log("  Got REPL via 'status'!")
        # Try WiFi without mode upload
        wifi_cmds = [
            "import network",
            "w = network.WLAN(network.AP_IF)",
            "w.active(True)",
            "w.config(essid='mBot2Status')",
            "import cyberpi; cyberpi.led.on(0, 255, 255)",
        ]
        for cmd in wifi_cmds:
            ser.write(cmd.encode() + b"\r\n")
            time.sleep(0.5)

        log("  WiFi commands sent via 'status' mode. Checking...")
        time.sleep(5.0)

    # Strategy 4: Send MicroPython via text mode (before upload)
    log("\n=== STRATEGY 4: Text mode code execution ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    ser.reset_input_buffer()

    # Try exec() as a text command before mode upload
    text_codes = [
        "exec import cyberpi; cyberpi.led.on(255,255,0)",
        "python import cyberpi; cyberpi.led.on(255,255,0)",
        "run import cyberpi; cyberpi.led.on(255,255,0)",
        "eval import cyberpi; cyberpi.led.on(255,255,0)",
    ]
    for cmd in text_codes:
        ser.reset_input_buffer()
        ser.write(cmd.encode() + b"\r\n")
        time.sleep(1.0)
        resp = read_all(ser, 1.0)
        show(f"text: {cmd[:30]}", resp)

    # Strategy 5: File write + machine.soft_reset()
    log("\n=== STRATEGY 5: Write main.py + soft reset ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    ser.reset_input_buffer()

    ok, _ = enter_repl(ser)
    if ok:
        # Write a minimal main.py that flashes LEDs in a distinctive pattern
        # If this runs after soft reset, we'll know main.py autorun works
        log("  Writing main.py with LED pattern...")
        file_lines = [
            "f = open('main.py', 'w')",
            "f.write('import cyberpi, time\\n')",
            "f.write('for i in range(10):\\n')",
            "f.write('  cyberpi.led.on(255, 0, 0)\\n')",
            "f.write('  time.sleep(0.3)\\n')",
            "f.write('  cyberpi.led.on(0, 255, 0)\\n')",
            "f.write('  time.sleep(0.3)\\n')",
            "f.close()",
        ]
        for line in file_lines:
            ser.write(line.encode() + b"\r\n")
            time.sleep(0.3)

        log("  main.py written. Attempting soft reset...")
        # machine.soft_reset() is different from RTS toggle
        # It restarts MicroPython but not the whole ESP32
        ser.write(b"import machine; machine.soft_reset()\r\n")
        time.sleep(3.0)
        resp = read_all(ser, 5.0)
        show("After soft_reset", resp)
        log("  CHECK: Is LED alternating red/green? (main.py autorun)")

        # Also try machine.reset() (full hardware reset)
        time.sleep(5.0)

        # Try Ctrl+D (MicroPython soft reset)
        log("\n  Trying Ctrl+D (MicroPython soft reboot)...")
        ser.write(b"\x04")  # Ctrl+D
        time.sleep(3.0)
        resp = read_all(ser, 5.0)
        show("After Ctrl+D", resp)
        log("  CHECK: Is LED alternating red/green? (main.py autorun)")

    log("\n========================================")
    log("MULTI-STRATEGY PROBE COMPLETE")

    # Final WiFi scan
    log("\nChecking for mBot WiFi networks...")
    ser.close()

    import subprocess
    try:
        result = subprocess.run(
            ["nmcli", "dev", "wifi", "list"],
            capture_output=True, text=True, timeout=10
        )
        for line in result.stdout.split('\n'):
            if 'mbot' in line.lower() or 'mBot' in line:
                log(f"  *** FOUND: {line} ***")
        else:
            if 'mbot' not in result.stdout.lower():
                log("  No mBot WiFi networks found")
    except Exception as e:
        log(f"  WiFi scan error: {e}")

    log(f"\nLog: {LOGFILE}")
    save_log()

def save_log():
    pass  # Already logging to file inline

if __name__ == "__main__":
    main()
