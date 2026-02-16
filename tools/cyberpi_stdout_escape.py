#!/usr/bin/env python3
"""
CyberPi stdout Escape Probe

We KNOW:
  1. REPL works - commands execute (LED flash confirmed)
  2. Print output is captured by CyberPiOS (goes to LCD, not serial)
  3. The f3 f5 handshake enters REPL and initially stdout works (banner appears)

Strategy: Try EVERY possible way to push data back over serial from within Python:
  - machine.UART direct write
  - os.dupterm() to re-attach stdout
  - sys.stdout replacement
  - machine.mem32 direct UART FIFO write
  - Writing f3 frames manually (CyberPiOS might forward them)
  - Accessing cyberpi module's communication methods

Each attempt uses a unique marker string so we know WHICH method worked.
"""

import serial
import time
import sys

SERIAL_PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_stdout_escape.log"
LOG = []

MARKERS = {}  # marker -> method name

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

def send_cmd(ser, cmd, label, wait=1.5):
    """Send a command to the REPL and collect any response."""
    ser.reset_input_buffer()
    if isinstance(cmd, str):
        cmd = cmd.encode()
    ser.write(cmd)
    time.sleep(wait)
    resp = b""
    while ser.in_waiting:
        resp += ser.read(ser.in_waiting)
        time.sleep(0.05)

    log(f"\n>>> [{label}]")
    log(f"  TX: {cmd!r}")
    log(f"  RX ({len(resp)} bytes): {resp.hex(' ') if resp else '(empty)'}")
    if resp:
        text = resp.decode('utf-8', errors='replace')
        printable = ''.join(c if c.isprintable() or c in '\n\r' else '.' for c in text)
        if printable.strip():
            log(f"  TXT: {printable.strip()[:200]}")

        # Check for ANY marker
        for marker, method in MARKERS.items():
            if marker.encode() in resp:
                log(f"  *** MARKER FOUND: '{marker}' via {method}! ***")
                log(f"  *** STDOUT ESCAPE SUCCESSFUL! ***")
    return resp

def main():
    log(f"=== CyberPi stdout Escape Probe - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")

    # Define markers for each escape method
    global MARKERS
    MARKERS = {
        "ESCAPE_UART0": "machine.UART(0) direct write",
        "ESCAPE_UART1": "machine.UART(1) direct write",
        "ESCAPE_DUPTERM": "os.dupterm()",
        "ESCAPE_STDOUT": "sys.stdout replacement",
        "ESCAPE_MEM32": "machine.mem32 UART FIFO",
        "ESCAPE_F3FRM": "manual f3 frame construction",
        "ESCAPE_CPSER": "cyberpi.serial/communication",
        "ESCAPE_RAWWR": "raw file descriptor write",
        "ESCAPE_PRINT": "normal print (control)",
        "ESCAPE_WRITE": "sys.stdout.write direct",
        "ESCAPE_BUFWR": "sys.stdout.buffer.write",
    }

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
    log("=== Resetting CyberPi ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(4.0)

    # Drain boot data
    boot = b""
    while ser.in_waiting:
        boot += ser.read(ser.in_waiting)
        time.sleep(0.05)
    log(f"Boot data: {len(boot)} bytes")

    # Enter REPL via text command (known working)
    log("\n=== Entering REPL ===")
    resp = send_cmd(ser, b"mode upload\r\n", "mode upload", 3.0)
    resp = send_cmd(ser, b"\x01", "Ctrl+A enter REPL", 2.0)

    # Verify we're in REPL (look for >>> in response)
    if b">>>" in resp:
        log("REPL confirmed!")
    else:
        log("REPL not confirmed, continuing anyway...")

    # ==========================================
    # METHOD 1: Normal print (control - expected to fail)
    # ==========================================
    log("\n=== METHOD 1: Normal print (control) ===")
    send_cmd(ser, b"print('ESCAPE_PRINT')\r\n", "normal print", 1.0)

    # ==========================================
    # METHOD 2: sys.stdout.write
    # ==========================================
    log("\n=== METHOD 2: sys.stdout.write ===")
    send_cmd(ser, b"import sys; sys.stdout.write('ESCAPE_WRITE\\n')\r\n",
             "sys.stdout.write", 1.0)

    # ==========================================
    # METHOD 3: sys.stdout.buffer.write (if exists)
    # ==========================================
    log("\n=== METHOD 3: sys.stdout.buffer.write ===")
    send_cmd(ser,
             b"import sys\r\n", "import sys", 0.3)
    send_cmd(ser,
             b"try:\r\n sys.stdout.buffer.write(b'ESCAPE_BUFWR\\n')\r\nexcept: pass\r\n\r\n",
             "stdout.buffer.write", 1.0)

    # ==========================================
    # METHOD 4: machine.UART(0) direct write
    # ==========================================
    log("\n=== METHOD 4: machine.UART(0) direct write ===")
    # This is the most promising - write directly to UART hardware
    send_cmd(ser,
             b"import machine; u=machine.UART(0,115200); u.write(b'ESCAPE_UART0\\n')\r\n",
             "UART(0) write", 1.5)

    # ==========================================
    # METHOD 5: machine.UART(1) direct write
    # ==========================================
    log("\n=== METHOD 5: machine.UART(1) direct write ===")
    send_cmd(ser,
             b"try:\r\n import machine; u=machine.UART(1,115200,tx=1,rx=3); u.write(b'ESCAPE_UART1\\n')\r\nexcept: pass\r\n\r\n",
             "UART(1) write", 1.5)

    # ==========================================
    # METHOD 6: os.dupterm()
    # ==========================================
    log("\n=== METHOD 6: os.dupterm() ===")
    # dupterm duplicates REPL output to a stream
    send_cmd(ser,
             b"import os, machine\r\n",
             "import os,machine", 0.3)
    send_cmd(ser,
             b"try:\r\n u=machine.UART(0,115200); os.dupterm(u,1); print('ESCAPE_DUPTERM')\r\nexcept Exception as e: pass\r\n\r\n",
             "os.dupterm", 2.0)
    # Also try dupterm with index 0
    send_cmd(ser,
             b"try:\r\n u=machine.UART(0,115200); os.dupterm(u,0); print('ESCAPE_DUPTERM')\r\nexcept: pass\r\n\r\n",
             "os.dupterm idx=0", 2.0)

    # ==========================================
    # METHOD 7: machine.mem32 direct UART FIFO
    # ==========================================
    log("\n=== METHOD 7: Direct UART FIFO via mem32 ===")
    # ESP32 UART0 FIFO register at 0x3FF40000
    # Write bytes directly to TX FIFO
    # Each byte written to offset 0x00 goes to TX
    fifo_code = (
        b"import machine\r\n"
    )
    send_cmd(ser, fifo_code, "import machine", 0.3)

    # Write each character of marker to FIFO
    marker_bytes = b"ESCAPE_MEM32\n"
    for byte in marker_bytes:
        send_cmd(ser,
                 f"machine.mem32[0x3FF40000]={byte}\r\n".encode(),
                 f"FIFO byte 0x{byte:02x}", 0.1)
    time.sleep(0.5)
    # Read any response
    if ser.in_waiting:
        data = ser.read(ser.in_waiting)
        log(f"  After FIFO writes: {data.hex(' ')}")
        text = data.decode('utf-8', errors='replace')
        if 'ESCAPE_MEM32' in text:
            log("  *** UART FIFO WRITE WORKED! ***")

    # ==========================================
    # METHOD 8: Construct f3 frame from Python
    # ==========================================
    log("\n=== METHOD 8: Manual f3 frame from Python ===")
    # If CyberPiOS intercepts f3 frames from UART TX and forwards them,
    # we could encapsulate our data in f3 frames
    send_cmd(ser,
             b"import machine; u=machine.UART(0,115200); u.write(bytes([0xf3,0xf7])+b'ESCAPE_F3FRM'+bytes([0xf4]))\r\n",
             "f3 frame from Python", 1.5)

    # ==========================================
    # METHOD 9: cyberpi module communication
    # ==========================================
    log("\n=== METHOD 9: cyberpi module methods ===")
    # Try accessing CyberPiOS's own communication methods
    commands = [
        b"import cyberpi\r\n",
        # Check what's available
        b"try:\r\n d=dir(cyberpi); [machine.UART(0,115200).write((x+'\\n').encode()) for x in d if 'ser' in x.lower() or 'ble' in x.lower() or 'com' in x.lower() or 'uart' in x.lower() or 'send' in x.lower() or 'write' in x.lower()]\r\nexcept: pass\r\n\r\n",
        # Try common communication methods
        b"try: cyberpi.serial.write('ESCAPE_CPSER\\n')\r\nexcept: pass\r\n\r\n",
        b"try: cyberpi.uart.write('ESCAPE_CPSER\\n')\r\nexcept: pass\r\n\r\n",
        b"try: cyberpi.communication.send('ESCAPE_CPSER')\r\nexcept: pass\r\n\r\n",
        b"try: cyberpi.ble.send('ESCAPE_CPSER')\r\nexcept: pass\r\n\r\n",
    ]
    for cmd in commands:
        send_cmd(ser, cmd, f"cyberpi: {cmd[:60]}", 1.0)

    # ==========================================
    # METHOD 10: Explore cyberpi module via UART write
    # ==========================================
    log("\n=== METHOD 10: Dump cyberpi dir via UART ===")
    # Since we can't see print output, use UART write to dump dir()
    send_cmd(ser,
             b"import cyberpi, machine\r\n",
             "imports", 0.3)
    # Write each attribute name to UART
    send_cmd(ser,
             b"u=machine.UART(0,115200)\r\n",
             "get uart", 0.3)
    send_cmd(ser,
             b"for x in dir(cyberpi): u.write((x+'\\n').encode())\r\n",
             "dump cyberpi dir", 3.0)

    # ==========================================
    # METHOD 11: Try raw REPL mode (Ctrl+A) for structured output
    # ==========================================
    log("\n=== METHOD 11: Raw REPL mode ===")
    send_cmd(ser, b"\x03", "Ctrl+C interrupt", 0.5)
    send_cmd(ser, b"\x01", "Ctrl+A raw REPL", 1.0)
    # In raw REPL: send code + Ctrl+D, get structured response
    # Format: OK<code output>\x04<error output>\x04>
    send_cmd(ser, b"print('ESCAPE_PRINT')\x04", "raw REPL print", 2.0)
    # Back to normal
    send_cmd(ser, b"\x02", "Ctrl+B normal REPL", 1.0)

    # ==========================================
    # METHOD 12: Multiple UART instances
    # ==========================================
    log("\n=== METHOD 12: UART variations ===")
    uart_configs = [
        # Different UART configs
        b"machine.UART(0, 115200).write(b'U0_115200\\n')\r\n",
        b"machine.UART(0, 9600).write(b'U0_9600\\n')\r\n",  # different baud
        # Try UART with specific pins (TX=1 is the USB-serial TX pin)
        b"try: machine.UART(2, 115200, tx=1, rx=3).write(b'U2_PIN1\\n')\r\nexcept: pass\r\n\r\n",
    ]
    for cmd in uart_configs:
        send_cmd(ser, cmd, f"UART: {cmd[:40]}", 0.5)

    # ==========================================
    # METHOD 13: os.dupterm with None first (clear existing)
    # ==========================================
    log("\n=== METHOD 13: Clear dupterm then re-set ===")
    send_cmd(ser,
             b"import os, machine\r\n",
             "imports", 0.3)
    # Clear all dupterms first
    send_cmd(ser,
             b"try: os.dupterm(None, 0)\r\nexcept: pass\r\n\r\n",
             "clear dupterm 0", 0.3)
    send_cmd(ser,
             b"try: os.dupterm(None, 1)\r\nexcept: pass\r\n\r\n",
             "clear dupterm 1", 0.3)
    # Now set UART as dupterm
    send_cmd(ser,
             b"u=machine.UART(0,115200); os.dupterm(u,0); print('ESCAPE_DUPTERM')\r\n",
             "set dupterm after clear", 2.0)

    # ==========================================
    # METHOD 14: Write to UART with flush
    # ==========================================
    log("\n=== METHOD 14: UART write with explicit flush ===")
    send_cmd(ser,
             b"import machine, time; u=machine.UART(0,115200)\r\n",
             "setup", 0.3)
    # Write with delays to ensure bytes get out
    send_cmd(ser,
             b"u.write(b'ESCAPE_UART0'); time.sleep_ms(100); u.write(b'\\n')\r\n",
             "write with delay", 1.0)

    # ==========================================
    # FINAL: Extended listen for any delayed data
    # ==========================================
    log("\n=== FINAL: Extended listen (5s) ===")
    ser.reset_input_buffer()
    all_data = b""
    deadline = time.time() + 5.0
    while time.time() < deadline:
        if ser.in_waiting:
            chunk = ser.read(ser.in_waiting)
            all_data += chunk
            log(f"  Late RX: {chunk.hex(' ')}")
            text = chunk.decode('utf-8', errors='replace')
            for marker in MARKERS:
                if marker in text:
                    log(f"  *** LATE MARKER: {marker} ***")
        time.sleep(0.05)

    if not all_data:
        log("  No data received")

    ser.close()

    # Summary
    log("\n" + "="*60)
    log("SUMMARY")
    log("="*60)
    found = []
    for marker, method in MARKERS.items():
        # Check all logged data for markers
        all_log = "\n".join(LOG)
        if f"MARKER FOUND: '{marker}'" in all_log or f"LATE MARKER: {marker}" in all_log:
            found.append(f"  YES: {method}")

    if found:
        log("WORKING ESCAPE METHODS:")
        for f in found:
            log(f)
    else:
        log("NO escape methods worked. CyberPiOS captures ALL output channels.")

    log(f"\nLog: {LOGFILE}")
    log("="*60)
    save_log()

if __name__ == "__main__":
    main()
