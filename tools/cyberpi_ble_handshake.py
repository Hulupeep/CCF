#!/usr/bin/env python3
"""
CyberPi BLE Handshake + Protocol Test

After f5/f6 handshake, try text commands and REPL.
The handshake might "unlock" the text protocol on BLE.
"""

import asyncio
import time
from bleak import BleakClient

DEVICE_ADDR = "10:97:BD:8F:4D:D2"
NOTIFY_UUID = "0000ffe2-0000-1000-8000-00805f9b34fb"
WRITE_UUID = "0000ffe3-0000-1000-8000-00805f9b34fb"

RX = []

def log(msg):
    print(msg)

def on_notify(sender, data):
    RX.append(data)
    text = data.decode('utf-8', errors='replace')
    printable = ''.join(c if c.isprintable() or c in '\n\r' else '.' for c in text)
    log(f"  <<< [{len(data)}] HEX: {data.hex(' ')}")
    if printable.strip():
        log(f"      TXT: {printable.strip()}")

async def tx(client, data, label="", wait=1.0):
    global RX
    RX.clear()
    if isinstance(data, str):
        data = data.encode()
    log(f"\n>>> {label} TX: {data!r}")
    await client.write_gatt_char(WRITE_UUID, data, response=False)
    await asyncio.sleep(wait)
    result = b"".join(RX)
    if not RX:
        log("  (no response)")
    return result

async def main():
    log(f"=== BLE Handshake + Protocol Test ===\n")
    async with BleakClient(DEVICE_ADDR, timeout=15.0) as client:
        log("Connected!")
        await client.start_notify(NOTIFY_UUID, on_notify)
        await asyncio.sleep(0.5)

        # Step 1: f5 handshake
        log("\n--- HANDSHAKE ---")
        await tx(client, bytes([0xf3, 0xf5, 0x02, 0x00, 0x08, 0xc0, 0xc8, 0xf4]), "f5 handshake", 1.0)
        await tx(client, bytes([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4]), "f6 config", 1.0)

        # Step 2: Try text commands AFTER handshake
        log("\n--- TEXT COMMANDS (post-handshake) ---")
        await tx(client, "help\r\n", "help", 2.0)
        await tx(client, "status\r\n", "status", 2.0)
        await tx(client, "mode upload\r\n", "mode upload", 3.0)

        # Step 3: Try Ctrl+A for REPL entry
        log("\n--- REPL ENTRY (post-handshake) ---")
        await tx(client, b"\x01", "Ctrl+A", 2.0)
        await tx(client, b"\x02", "Ctrl+B", 2.0)

        # Step 4: Try print if we might be in REPL
        await tx(client, b"print('HELLO')\r\n", "print test", 2.0)

        # Step 5: Try with different f3 frame after handshake
        log("\n--- MORE F3 FRAMES (post-handshake) ---")

        # Maybe there's a "start live mode" command
        # Try f3 with types near f5/f6
        for cmd_type in [0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff]:
            for payload in [b"\x00", b"\x01", b"\x02\x00\x08",
                           b"\x03\x00\x0d\x00\x00\x0d",
                           b"\x01\x00\x00\x00"]:
                resp = await tx(client, bytes([0xf3, cmd_type]) + payload + bytes([0xf4]),
                               f"f3 {cmd_type:02x} {payload.hex()}", 0.2)
                if resp:
                    log(f"  *** GOT RESPONSE for f3 {cmd_type:02x} ***")

        # Step 6: What about doing a second handshake with different values?
        log("\n--- VARIED HANDSHAKES ---")
        handshake_variants = [
            bytes([0xf3, 0xf5, 0x01, 0x00, 0x08, 0xc0, 0xc8, 0xf4]),  # byte[2]=01
            bytes([0xf3, 0xf5, 0x02, 0x00, 0x04, 0xc0, 0xc8, 0xf4]),  # byte[4]=04
            bytes([0xf3, 0xf5, 0x02, 0x00, 0x10, 0xc0, 0xc8, 0xf4]),  # byte[4]=10
            bytes([0xf3, 0xf5, 0x02, 0x01, 0x08, 0xc0, 0xc8, 0xf4]),  # byte[3]=01
            bytes([0xf3, 0xf5, 0x03, 0x00, 0x08, 0xc0, 0xc8, 0xf4]),  # byte[2]=03
            bytes([0xf3, 0xf5, 0x04, 0x00, 0x08, 0xc0, 0xc8, 0xf4]),  # byte[2]=04
        ]
        for variant in handshake_variants:
            resp = await tx(client, variant, f"f5 variant: {variant.hex(' ')}", 0.5)

        # Step 7: Try the f6 with different configs
        log("\n--- VARIED CONFIGS ---")
        config_variants = [
            bytes([0xf3, 0xf6, 0x01, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4]),
            bytes([0xf3, 0xf6, 0x02, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4]),
            bytes([0xf3, 0xf6, 0x03, 0x01, 0x0d, 0x00, 0x00, 0x0d, 0xf4]),
            bytes([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x01, 0x00, 0x0d, 0xf4]),
            bytes([0xf3, 0xf6, 0x04, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4]),
        ]
        for variant in config_variants:
            resp = await tx(client, variant, f"f6 variant: {variant.hex(' ')}", 0.5)

        # Step 8: After all that, check for late notifications
        log("\n--- FINAL LISTEN (3s) ---")
        RX.clear()
        await asyncio.sleep(3.0)
        if RX:
            for data in RX:
                log(f"  Late data: {data.hex(' ')}")

        await client.stop_notify(NOTIFY_UUID)

    log("\n=== COMPLETE ===")

if __name__ == "__main__":
    asyncio.run(main())
