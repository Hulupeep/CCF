#!/usr/bin/env python3
"""
CyberPi Fresh BLE Test

Reset CyberPi via serial, then connect via BLE with clean state.
Test both f3 protocol AND text commands with notifications enabled.
"""

import asyncio
import serial
import time
from bleak import BleakClient, BleakScanner

SERIAL_PORT = "/dev/ttyUSB0"
DEVICE_ADDR = "10:97:BD:8F:4D:D2"
NOTIFY_UUID = "0000ffe2-0000-1000-8000-00805f9b34fb"
WRITE_UUID = "0000ffe3-0000-1000-8000-00805f9b34fb"

RX = []

def log(msg):
    print(msg)

def on_notify(sender, data):
    RX.append(data)
    log(f"  <<< [{len(data)}] {data.hex(' ')}")
    text = data.decode('utf-8', errors='replace')
    printable = ''.join(c if c.isprintable() or c in '\n\r' else '.' for c in text)
    if printable.strip():
        log(f"      TXT: {printable.strip()}")

async def tx(client, data, label="", wait=1.0):
    global RX
    RX.clear()
    if isinstance(data, str):
        data = data.encode()
    log(f"\n>>> [{label}] TX: {data.hex(' ')}" if not all(32 <= b < 127 or b in (10,13) for b in data) else f"\n>>> [{label}] TX: {data!r}")
    await client.write_gatt_char(WRITE_UUID, data, response=False)
    await asyncio.sleep(wait)
    if not RX:
        log("  (no response)")
    return b"".join(RX)

def reset_cyberpi():
    """Reset CyberPi via serial RTS toggle, then close serial."""
    log("Resetting CyberPi via serial RTS...")
    ser = serial.Serial()
    ser.port = SERIAL_PORT
    ser.baudrate = 115200
    ser.timeout = 0.1
    ser.dtr = False
    ser.rts = False
    ser.open()
    ser.dtr = False
    ser.rts = False
    time.sleep(0.1)

    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(3.0)  # Wait for boot

    # Read and discard boot output
    while ser.in_waiting:
        ser.read(ser.in_waiting)
        time.sleep(0.1)

    ser.close()
    log("Serial port closed. CyberPi should be in fresh boot state.")
    time.sleep(2.0)  # Extra wait for BLE to come up

async def main():
    log(f"=== Fresh BLE Test - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")

    # Step 1: Reset via serial
    reset_cyberpi()

    # Step 2: Scan for device
    log("\nScanning for CyberPi BLE...")
    devices = await BleakScanner.discover(timeout=10.0)
    target = None
    for d in devices:
        name = d.name or ""
        if "makeblock" in name.lower() or "mbot" in name.lower():
            log(f"  Found: {d.name} ({d.address})")
            target = d

    if not target:
        log("No Makeblock device found after reset!")
        return

    # Step 3: Connect
    log(f"\nConnecting to {target.name} ({target.address})...")
    async with BleakClient(target.address, timeout=15.0) as client:
        log("Connected!")
        await client.start_notify(NOTIFY_UUID, on_notify)
        log("Subscribed to ffe2")

        # Wait for any initial data
        log("\n--- Listening for initial data (3s) ---")
        await asyncio.sleep(3.0)
        if RX:
            log(f"  Got {len(RX)} initial notifications!")
        else:
            log("  No initial data")

        # Test 1: f5 handshake (first thing after fresh connection)
        log("\n--- TEST 1: f5 handshake ---")
        resp = await tx(client, bytes([0xf3, 0xf5, 0x02, 0x00, 0x08, 0xc0, 0xc8, 0xf4]),
                        "f5 handshake", 2.0)

        # Test 2: f6 config
        log("\n--- TEST 2: f6 config ---")
        resp = await tx(client, bytes([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4]),
                        "f6 config", 2.0)

        # Test 3: Text commands
        log("\n--- TEST 3: Text commands after handshake ---")
        await tx(client, "help\r\n", "help", 2.0)
        await tx(client, "status\r\n", "status", 2.0)
        await tx(client, "mode upload\r\n", "mode upload", 3.0)

        # Test 4: REPL
        log("\n--- TEST 4: REPL entry ---")
        await tx(client, b"\x01", "Ctrl+A", 2.0)
        await tx(client, b"print('BLE_TEST')\r\n", "print", 2.0)

        # Test 5: LED command to verify BLE writes execute
        log("\n--- TEST 5: LED via REPL over BLE ---")
        await tx(client, b"import cyberpi; cyberpi.led.on(255, 0, 0)\r\n",
                 "LED red", 2.0)
        log("  CHECK: Is LED RED? (confirms BLE writes execute)")

        await asyncio.sleep(2.0)
        await tx(client, b"cyberpi.led.off()\r\n", "LED off", 1.0)

        # Test 6: BLE-only mode (no serial involved)
        # What if the CyberPi has a BLE-specific protocol that's
        # different from serial? Let's try some non-f3 binary frames
        log("\n--- TEST 6: Alternative binary protocols ---")
        alt_protos = [
            # Makeblock protocol v2 (different header)
            bytes([0xff, 0x55, 0x02, 0x00, 0x00]),
            # ESP-NOW style
            bytes([0xfe, 0xfe, 0x00, 0x01]),
            # Simple query
            bytes([0x00]),
            bytes([0x01]),
            # Ping-style
            bytes([0xaa, 0x55, 0x00]),
        ]
        for proto in alt_protos:
            resp = await tx(client, proto, f"alt: {proto.hex(' ')}", 0.5)

        # Test 7: Long listen
        log("\n--- TEST 7: Final listen (5s) ---")
        RX.clear()
        await asyncio.sleep(5.0)
        if RX:
            for data in RX:
                log(f"  Late: {data.hex(' ')}")

        await client.stop_notify(NOTIFY_UUID)

    log("\n=== COMPLETE ===")

if __name__ == "__main__":
    asyncio.run(main())
