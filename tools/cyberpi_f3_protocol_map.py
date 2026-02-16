#!/usr/bin/env python3
"""
CyberPi f3 Protocol Mapper

Now that we know f3 frames work over BLE, let's map the full protocol.
Known:
  f3 f5 = handshake (version/capability)
  f3 f6 = configuration

Unknown: which frame types read sensors, control motors, etc.

Strategy: Send every f3 command type (0x00-0xff) with minimal payloads
and record which ones get responses. Then vary payloads for responders.
"""

import asyncio
import time
from bleak import BleakClient, BleakScanner

DEVICE_ADDR = "10:97:BD:8F:4D:D2"
NOTIFY_UUID = "0000ffe2-0000-1000-8000-00805f9b34fb"
WRITE_UUID = "0000ffe3-0000-1000-8000-00805f9b34fb"

LOGFILE = "/tmp/cyberpi_f3_map.log"
LOG = []
RX_BUFFER = []

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

def on_notify(sender, data):
    RX_BUFFER.append((time.time(), data))

async def send_f3(client, cmd_type, payload=b"", wait=0.3):
    """Send f3 frame and return response."""
    RX_BUFFER.clear()
    frame = bytes([0xf3, cmd_type]) + payload + bytes([0xf4])
    await client.write_gatt_char(WRITE_UUID, frame, response=False)
    await asyncio.sleep(wait)

    if RX_BUFFER:
        resp = b"".join(d[1] for d in RX_BUFFER)
        return resp
    return None

async def main():
    log(f"=== CyberPi f3 Protocol Mapper - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")

    log(f"Connecting to {DEVICE_ADDR}...")
    async with BleakClient(DEVICE_ADDR, timeout=15.0) as client:
        log("Connected!")
        await client.start_notify(NOTIFY_UUID, on_notify)
        log("Subscribed to notifications\n")
        await asyncio.sleep(0.5)

        # ========================================
        # PHASE 1: Handshake (known working)
        # ========================================
        log("=== PHASE 1: Handshake ===")
        resp = await send_f3(client, 0xf5, bytes([0x02, 0x00, 0x08, 0xc0, 0xc8]), 1.0)
        if resp:
            log(f"  f5 handshake: {resp.hex(' ')}")
        resp = await send_f3(client, 0xf6, bytes([0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d]), 1.0)
        if resp:
            log(f"  f6 config: {resp.hex(' ')}")

        # ========================================
        # PHASE 2: Sweep all command types (0x00-0xff)
        # ========================================
        log("\n=== PHASE 2: Command type sweep ===")
        log("Sending f3 [type] 00 f4 for types 0x00-0xff...\n")

        responders = {}
        for cmd in range(0x00, 0x100):
            resp = await send_f3(client, cmd, bytes([0x00]), 0.15)
            if resp:
                responders[cmd] = resp
                log(f"  0x{cmd:02x}: RESPONSE {resp.hex(' ')}")

        log(f"\n  {len(responders)} command types responded")

        # ========================================
        # PHASE 3: Sweep with longer payloads
        # ========================================
        log("\n=== PHASE 3: Varied payloads for responders ===")
        for cmd in sorted(responders.keys()):
            log(f"\n  --- Type 0x{cmd:02x} ---")

            # Try various payload lengths
            payloads = [
                bytes([]),
                bytes([0x00]),
                bytes([0x01]),
                bytes([0x00, 0x00]),
                bytes([0x01, 0x00]),
                bytes([0x00, 0x01]),
                bytes([0x02, 0x00]),
                bytes([0x00, 0x00, 0x00]),
                bytes([0x01, 0x00, 0x00]),
                bytes([0x02, 0x00, 0x08]),
                bytes([0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d]),
            ]
            for payload in payloads:
                resp = await send_f3(client, cmd, payload, 0.15)
                if resp:
                    log(f"    payload {payload.hex(' '):20s} -> {resp.hex(' ')}")

        # ========================================
        # PHASE 4: Try f3 with MicroPython code
        # ========================================
        log("\n=== PHASE 4: f3 with embedded code/data ===")

        # Maybe f3 frames can carry Python code?
        code_payloads = [
            b"\x01print('hello')",
            b"\x02print('hello')",
            b"print('hello')",
            b"\x01\x00" + b"print('hello')",
        ]
        for payload in code_payloads:
            resp = await send_f3(client, 0xf5, payload, 0.5)
            if resp:
                log(f"  f5 + {payload[:20]!r}: {resp.hex(' ')}")
            resp = await send_f3(client, 0xf6, payload, 0.5)
            if resp:
                log(f"  f6 + {payload[:20]!r}: {resp.hex(' ')}")

        # ========================================
        # PHASE 5: Try known Makeblock sensor IDs
        # ========================================
        log("\n=== PHASE 5: Makeblock sensor device IDs ===")
        log("Trying standard Makeblock device IDs in f3 frames...\n")

        # Makeblock device IDs (from old protocol):
        # 1=ultrasonic, 3=light, 6=gyro, 7=sound, etc.
        for device_id in range(0, 32):
            for port in range(0, 4):
                payload = bytes([device_id, port])
                for cmd_type in [0x01, 0x02, 0x03, 0x04, 0x05, 0x10, 0x20,
                                 0xf5, 0xf6, 0xf7, 0xf8, 0xf9]:
                    resp = await send_f3(client, cmd_type, payload, 0.1)
                    if resp and resp != bytes([0xf3, cmd_type]) + payload + bytes([0xf4]):
                        log(f"  type=0x{cmd_type:02x} dev={device_id} port={port}: {resp.hex(' ')}")

        # ========================================
        # PHASE 6: Continuous monitoring
        # ========================================
        log("\n=== PHASE 6: Monitor for 5 seconds ===")
        log("Listening for any unsolicited data...")
        RX_BUFFER.clear()
        await asyncio.sleep(5.0)
        if RX_BUFFER:
            for ts, data in RX_BUFFER:
                log(f"  Unsolicited: {data.hex(' ')}")
        else:
            log("  No unsolicited data")

        await client.stop_notify(NOTIFY_UUID)

    # Summary
    log("\n========================================")
    log("PROTOCOL MAP COMPLETE")
    log(f"\nResponding command types: {sorted(f'0x{k:02x}' for k in responders.keys())}")
    log(f"Log: {LOGFILE}")
    log("========================================")
    save_log()

if __name__ == "__main__":
    asyncio.run(main())
