#!/usr/bin/env python3
"""
CyberPi UART Bypass Tests

Test 1: Direct ESP32 UART hardware register writes
  - Bypass CyberPiOS by writing to UART0 TX FIFO at 0x3FF40000

Test 2: WiFi AP + TCP socket bridge
  - Create AP, start TCP server, confirm data flows over WiFi
"""

import serial
import time
import socket
import subprocess

PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_bypass.log"

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

def enter_repl(ser):
    """Reset and enter REPL."""
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    ser.reset_input_buffer()

    ser.write(b"mode upload\r\n")
    time.sleep(2.0)
    ser.reset_input_buffer()

    ser.write(b"\x01")
    time.sleep(2.0)
    resp = read_all(ser, 3.0)
    if b">>>" in resp:
        log("REPL ready")
        return True
    log(f"No REPL prompt. Got: {resp[:50]!r}")
    return False

def send_repl(ser, code, delay=0.5):
    """Send a line of code to REPL."""
    if isinstance(code, str):
        code = code.encode()
    ser.write(code + b"\r\n")
    time.sleep(delay)
    # Read any response (usually empty due to output capture)
    return read_all(ser, 0.5)

def main():
    with open(LOGFILE, "w") as f:
        f.write(f"=== CyberPi UART Bypass - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n\n")

    ser = serial.Serial()
    ser.port = PORT
    ser.baudrate = BAUD
    ser.timeout = 0.01
    ser.dtr = False
    ser.rts = False
    ser.open()
    ser.dtr = False
    ser.rts = False

    if not enter_repl(ser):
        log("Could not enter REPL. Aborting.")
        ser.close()
        return

    # ========================================
    # TEST 1: Direct UART register writes
    # ========================================
    log("\n=== TEST 1: Direct UART0 register write ===")
    log("Writing to ESP32 UART0 TX FIFO (0x3FF40000)...")

    ser.reset_input_buffer()

    # Send the register write code
    # ESP32 UART0 base = 0x3FF40000, FIFO offset = 0x00
    lines = [
        "from machine import mem32",
        "UART_FIFO = 0x3FF40000",
        "for c in b'HWUART_OK\\r\\n': mem32[UART_FIFO] = c",
    ]
    for line in lines:
        resp = send_repl(ser, line)
        if resp:
            log(f"  Unexpected response to '{line[:30]}': {resp.hex(' ')}")

    # Now read - the hardware UART should have bytes in the TX buffer
    time.sleep(0.5)
    resp = read_all(ser, 3.0)
    if resp:
        log(f"  GOT RESPONSE: {resp!r}")
        if b"HWUART_OK" in resp:
            log("  *** DIRECT UART REGISTER WRITE WORKS! ***")
        else:
            log(f"  Got bytes but not our marker: {resp.hex(' ')}")
    else:
        log("  No bytes received. CyberPiOS may intercept at hardware level too.")

    # Try alternative: write to UART TX register at different address
    # Some ESP32 variants use different base addresses
    log("\n  Trying alternative register addresses...")
    alt_addrs = [
        0x3FF40000,  # UART0 FIFO
        0x3FF4C000,  # UART1 FIFO
        0x3FF6E000,  # UART2 FIFO
    ]
    for addr in alt_addrs:
        ser.reset_input_buffer()
        code = f"from machine import mem32\nfor c in b'REG_{addr:08X}\\n': mem32[{addr}] = c"
        send_repl(ser, code, 1.0)
        time.sleep(0.5)
        resp = read_all(ser, 2.0)
        if resp:
            log(f"  Address 0x{addr:08X}: GOT {len(resp)} bytes: {resp!r}")
        else:
            log(f"  Address 0x{addr:08X}: nothing")

    # ========================================
    # TEST 2: WiFi AP + TCP bridge
    # ========================================
    log("\n=== TEST 2: WiFi AP bridge ===")
    log("Setting up WiFi access point on CyberPi...")

    ser.reset_input_buffer()

    # Step 1: Create WiFi AP
    wifi_lines = [
        "import network",
        "ap = network.WLAN(network.AP_IF)",
        "ap.active(True)",
        "ap.config(essid='mBot2Bridge', authmode=0)",
        # Confirm AP is active with LED
        "import cyberpi",
        "cyberpi.led.on(0, 0, 255)",  # Blue = AP starting
    ]
    for line in wifi_lines:
        send_repl(ser, line, 0.5)
        log(f"  Sent: {line}")

    log("  Waiting for AP to start (3s)...")
    time.sleep(3.0)

    # LED green = AP ready
    send_repl(ser, "cyberpi.led.on(0, 255, 0)", 0.3)

    # Step 2: Start TCP server
    tcp_lines = [
        "import socket",
        "s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)",
        "s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)",
        "s.bind(('0.0.0.0', 5000))",
        "s.listen(1)",
        # LED yellow = server ready
        "cyberpi.led.on(255, 255, 0)",
    ]
    for line in tcp_lines:
        send_repl(ser, line, 0.5)
        log(f"  Sent: {line}")

    log("\n  WiFi AP + TCP server should now be running on CyberPi!")
    log("  AP SSID: mBot2Bridge")
    log("  TCP: 192.168.4.1:5000")
    log("")
    log("  LED should be YELLOW if everything worked.")
    log("  LED BLUE means AP was created but TCP might have failed.")
    log("  LED RED means something errored.")
    log("")

    # Step 3: Set up a simple echo handler (for testing)
    # This will accept one connection and echo data back with sensor prefix
    echo_code = (
        "import time\n"
        "try:\n"
        "  conn, addr = s.accept()  # blocks until connection\n"
        "  cyberpi.led.on(0, 255, 0)  # green = connected\n"
        "  while True:\n"
        "    data = conn.recv(128)\n"
        "    if not data: break\n"
        "    snd = cyberpi.get_loudness()\n"
        "    lgt = cyberpi.get_brightness()\n"
        "    conn.send(f'SND:{snd},LGT:{lgt}\\n'.encode())\n"
        "except Exception as e:\n"
        "  cyberpi.led.on(255, 0, 0)  # red = error\n"
    )

    # We need to send this as an exec() one-liner since REPL doesn't echo
    # Replace newlines with \\n for exec string
    exec_code = "exec('" + echo_code.replace("'", "\\'").replace("\n", "\\n") + "')"

    log(f"\n  Sending echo server code ({len(exec_code)} chars)...")
    log("  NOTE: This will BLOCK the REPL waiting for TCP connection.")
    log("  Connect to WiFi 'mBot2Bridge' and then to 192.168.4.1:5000")

    # Send the echo server
    send_repl(ser, exec_code, 1.0)

    # Step 4: Try to connect (if on the right network)
    log("\n  Attempting TCP connection to 192.168.4.1:5000...")
    log("  (This will fail if laptop isn't on mBot2Bridge network)")

    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5.0)
        sock.connect(("192.168.4.1", 5000))
        log("  *** CONNECTED! ***")

        sock.send(b"ping\n")
        time.sleep(0.5)
        data = sock.recv(1024)
        log(f"  Received: {data!r}")

        if b"SND:" in data:
            log("  *** SENSOR DATA RECEIVED OVER WIFI! ***")
            log("  WiFi bridge is working!")

        sock.close()
    except (socket.timeout, ConnectionRefusedError, OSError) as e:
        log(f"  Connection failed: {e}")
        log("  This is expected if laptop isn't connected to mBot2Bridge WiFi")

    log("\n  NEXT STEPS:")
    log("  1. Check CyberPi LED color (yellow=server ready, blue=AP only, red=error)")
    log("  2. On laptop: scan for 'mBot2Bridge' WiFi network")
    log("  3. If visible: connect to it, then run the WiFi test below")
    log("  4. If not visible: WiFi AP creation may have failed")

    log("\n========================================")
    log("BYPASS TESTS COMPLETE")
    log(f"Log: {LOGFILE}")
    log("========================================")

    ser.close()

if __name__ == "__main__":
    main()
