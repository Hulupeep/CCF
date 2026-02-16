#!/usr/bin/env python3
"""
CyberPi BLE UART Communication Test

We found a BLE UART service on CyberPi:
  Service: 0000ffe1 (Vendor specific)
  ffe2: notify  (CyberPi -> laptop, RX)
  ffe3: write   (laptop -> CyberPi, TX)

This is the wireless programming channel used by mBlock.
Let's test bidirectional communication via BLE.
"""

import asyncio
import time
from bleak import BleakClient, BleakScanner

DEVICE_ADDR = "10:97:BD:8F:4D:D2"
NOTIFY_UUID = "0000ffe2-0000-1000-8000-00805f9b34fb"
WRITE_UUID = "0000ffe3-0000-1000-8000-00805f9b34fb"

LOGFILE = "/tmp/cyberpi_ble_uart.log"
LOG = []
RX_DATA = []

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

def on_notify(sender, data):
    """Callback for BLE notifications."""
    timestamp = time.time()
    RX_DATA.append((timestamp, data))
    text = data.decode('utf-8', errors='replace')
    log(f"  RX [{len(data)}]: {data.hex(' ')}")
    printable = ''.join(c if c.isprintable() or c in '\n\r\t' else '.' for c in text)
    if printable.strip():
        log(f"      TXT: {printable.strip()!r}")

async def send_and_wait(client, data, label, wait=3.0):
    """Send data and wait for notifications."""
    global RX_DATA
    RX_DATA.clear()

    log(f"\n--- {label} ---")
    if isinstance(data, str):
        data = data.encode()
    log(f"  TX [{len(data)}]: {data!r}")

    await client.write_gatt_char(WRITE_UUID, data, response=False)
    await asyncio.sleep(wait)

    if RX_DATA:
        total = b"".join(d[1] for d in RX_DATA)
        log(f"  Total RX: {len(total)} bytes in {len(RX_DATA)} notifications")
        return total
    else:
        log(f"  No response received")
        return b""

async def main():
    log(f"=== CyberPi BLE UART Test - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")

    log(f"Connecting to {DEVICE_ADDR}...")
    async with BleakClient(DEVICE_ADDR, timeout=15.0) as client:
        log(f"Connected!")

        # Subscribe to notifications
        await client.start_notify(NOTIFY_UUID, on_notify)
        log("Subscribed to ffe2 notifications\n")

        # Wait briefly for any initial data
        await asyncio.sleep(1.0)
        if RX_DATA:
            log(f"Received {len(RX_DATA)} initial notifications")

        # ========================================
        # TEST 1: Text commands
        # ========================================
        log("\n=== TEST 1: Text commands ===")

        resp = await send_and_wait(client, "help\r\n", "help", 2.0)
        resp = await send_and_wait(client, "status\r\n", "status", 3.0)
        resp = await send_and_wait(client, "version\r\n", "version", 2.0)

        # ========================================
        # TEST 2: mode upload + REPL entry
        # ========================================
        log("\n=== TEST 2: mode upload + REPL ===")

        resp = await send_and_wait(client, "mode upload\r\n", "mode upload", 3.0)

        # Ctrl+A (raw REPL or enter REPL)
        resp = await send_and_wait(client, b"\x01", "Ctrl+A", 2.0)

        # Check for >>> prompt
        all_rx = b"".join(d[1] for d in RX_DATA)
        if b">>>" in all_rx:
            log("\n*** GOT REPL PROMPT VIA BLE! ***")

        # ========================================
        # TEST 3: REPL commands over BLE
        # ========================================
        log("\n=== TEST 3: REPL commands via BLE ===")

        # Simple print
        resp = await send_and_wait(client, "print('BLE_HELLO_12345')\r\n", "print test", 3.0)
        if b"BLE_HELLO_12345" in resp:
            log("\n*** PRINT OUTPUT RECEIVED VIA BLE! ***")

        # Expression
        resp = await send_and_wait(client, "2+2\r\n", "2+2", 2.0)
        if b"4" in resp:
            log("\n*** EXPRESSION RESULT RECEIVED! ***")

        # Import and sensor read
        resp = await send_and_wait(client, "import cyberpi\r\n", "import cyberpi", 2.0)
        resp = await send_and_wait(client, "print(cyberpi.get_loudness())\r\n", "get_loudness", 2.0)
        resp = await send_and_wait(client, "print(cyberpi.get_brightness())\r\n", "get_brightness", 2.0)

        # LED test to confirm execution
        resp = await send_and_wait(client, "cyberpi.led.on(0, 255, 0)\r\n", "LED green", 1.0)

        # ========================================
        # TEST 4: Raw REPL (Ctrl+A, send code + Ctrl+D)
        # ========================================
        log("\n=== TEST 4: Raw REPL via BLE ===")

        # Enter raw REPL
        resp = await send_and_wait(client, b"\x03", "Ctrl+C", 1.0)  # Interrupt
        resp = await send_and_wait(client, b"\x01", "Ctrl+A raw REPL", 2.0)

        # Send code + Ctrl+D
        code = b"print('RAW_BLE_OK')\x04"
        resp = await send_and_wait(client, code, "raw REPL print", 3.0)
        if b"RAW_BLE_OK" in resp:
            log("\n*** RAW REPL OUTPUT VIA BLE! ***")

        # ========================================
        # TEST 5: Sensor data read via BLE
        # ========================================
        log("\n=== TEST 5: Multi-sensor read ===")

        # Back to normal REPL
        resp = await send_and_wait(client, b"\x02", "Ctrl+B normal REPL", 2.0)

        sensor_cmd = (
            "import cyberpi, mbuild; "
            "print('S:', cyberpi.get_loudness(), "
            "cyberpi.get_brightness(), "
            "cyberpi.get_gyro('x'), "
            "cyberpi.get_gyro('y'), "
            "cyberpi.get_gyro('z'))\r\n"
        )
        resp = await send_and_wait(client, sensor_cmd, "multi-sensor read", 3.0)
        if b"S:" in resp:
            log("\n*** SENSOR DATA RECEIVED VIA BLE! ***")

        # ========================================
        # TEST 6: f3 protocol commands
        # ========================================
        log("\n=== TEST 6: f3 protocol ===")

        # Try sending f3 frames
        f3_cmds = [
            bytes([0xf3, 0xf5, 0x02, 0x00, 0x08, 0xc0, 0xc8, 0xf4]),
            bytes([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4]),
            bytes([0xf3, 0x01, 0x00, 0xf4]),
        ]
        for cmd in f3_cmds:
            resp = await send_and_wait(client, cmd, f"f3 frame: {cmd.hex(' ')}", 1.0)

        # Turn off LED
        await send_and_wait(client, "cyberpi.led.off()\r\n", "LED off", 0.5)

        # Unsubscribe
        await client.stop_notify(NOTIFY_UUID)

    log("\n========================================")
    log("BLE UART TEST COMPLETE")

    # Summary
    total_rx = sum(len(d[1]) for d in RX_DATA)
    log(f"\nTotal: {len(RX_DATA)} notifications, {total_rx} bytes received")
    log(f"Log: {LOGFILE}")
    log("========================================")
    save_log()

if __name__ == "__main__":
    asyncio.run(main())
