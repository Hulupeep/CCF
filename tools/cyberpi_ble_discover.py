#!/usr/bin/env python3
"""
CyberPi BLE Discovery

Connect to the CyberPi's BLE GATT server and enumerate all services,
characteristics, and descriptors. Read any readable values.

The CyberPi advertises as "Makeblock_LE..." via CyberPiOS.
If CyberPiOS exposes sensor data via BLE, we can read it directly
without any serial hacking.
"""

import asyncio
import sys
from bleak import BleakClient, BleakScanner

LOGFILE = "/tmp/cyberpi_ble_discover.log"
LOG = []

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

async def main():
    log(f"=== CyberPi BLE Discovery ===\n")

    # Scan for CyberPi
    log("Scanning for Makeblock BLE devices...")
    devices = await BleakScanner.discover(timeout=10.0)

    target = None
    for d in devices:
        name = d.name or ""
        if "makeblock" in name.lower() or "mbot" in name.lower() or "cyber" in name.lower():
            log(f"  FOUND: {d.name} ({d.address})")
            target = d
        elif d.name:
            pass  # skip other devices

    if not target:
        log("No Makeblock BLE device found!")
        log("Make sure CyberPi is powered on and nearby.")
        save_log()
        return

    log(f"\nConnecting to {target.name} ({target.address})...")

    try:
        async with BleakClient(target.address, timeout=15.0) as client:
            log(f"Connected! MTU: {client.mtu_size}")

            # Enumerate all services
            log(f"\n=== GATT Services ({len(client.services.services)}) ===\n")

            for service in client.services:
                log(f"Service: {service.uuid}")
                log(f"  Description: {service.description}")
                log(f"  Handle: {service.handle}")

                for char in service.characteristics:
                    props = ", ".join(char.properties)
                    log(f"\n  Characteristic: {char.uuid}")
                    log(f"    Description: {char.description}")
                    log(f"    Properties: {props}")
                    log(f"    Handle: {char.handle}")

                    # Try to read if readable
                    if "read" in char.properties:
                        try:
                            value = await client.read_gatt_char(char)
                            log(f"    Value ({len(value)} bytes): {value.hex(' ')}")
                            # Try to decode as text
                            try:
                                text = value.decode('utf-8')
                                if text.isprintable():
                                    log(f"    Text: {text!r}")
                            except:
                                pass
                            # Try to decode as numbers
                            if len(value) == 2:
                                import struct
                                val = struct.unpack('<H', value)[0]
                                log(f"    uint16: {val}")
                            elif len(value) == 4:
                                import struct
                                val = struct.unpack('<f', value)[0]
                                log(f"    float32: {val}")
                                val2 = struct.unpack('<I', value)[0]
                                log(f"    uint32: {val2}")
                        except Exception as e:
                            log(f"    Read error: {e}")

                    # List descriptors
                    for desc in char.descriptors:
                        log(f"    Descriptor: {desc.uuid}")
                        log(f"      Description: {desc.description}")
                        try:
                            value = await client.read_gatt_descriptor(desc.handle)
                            log(f"      Value: {value.hex(' ')}")
                        except Exception as e:
                            log(f"      Read error: {e}")

                log("")

            # Try subscribing to notify characteristics
            log("\n=== Testing Notify Characteristics ===\n")

            notify_chars = []
            for service in client.services:
                for char in service.characteristics:
                    if "notify" in char.properties or "indicate" in char.properties:
                        notify_chars.append(char)

            if notify_chars:
                received_data = {}

                for char in notify_chars:
                    log(f"Subscribing to notifications: {char.uuid}...")

                    def make_handler(uuid):
                        def handler(sender, data):
                            if uuid not in received_data:
                                received_data[uuid] = []
                            received_data[uuid].append(data)
                        return handler

                    try:
                        await client.start_notify(char, make_handler(char.uuid))
                        log(f"  Subscribed to {char.uuid}")
                    except Exception as e:
                        log(f"  Subscribe error: {e}")

                # Wait for notifications
                log(f"\nWaiting 5 seconds for notifications...")
                await asyncio.sleep(5.0)

                # Report what we received
                for uuid, data_list in received_data.items():
                    log(f"\n  {uuid}: received {len(data_list)} notifications")
                    for i, data in enumerate(data_list[:5]):
                        log(f"    [{i}] {data.hex(' ')} ({len(data)} bytes)")
                        # Try decoding
                        try:
                            text = data.decode('utf-8')
                            if text.isprintable():
                                log(f"         Text: {text!r}")
                        except:
                            pass

                # Unsubscribe
                for char in notify_chars:
                    try:
                        await client.stop_notify(char)
                    except:
                        pass

            else:
                log("No notify/indicate characteristics found.")

            # Check for our custom service (from the BLE bridge test)
            custom_uuid = "12345678-1234-5678-1234-56789abcdef0"
            for service in client.services:
                if custom_uuid in service.uuid:
                    log(f"\n*** CUSTOM SERVICE FOUND: {service.uuid} ***")
                    log("*** Our BLE bridge setup worked! ***")

            log(f"\nDisconnecting...")

    except Exception as e:
        log(f"Connection failed: {e}")
        log("The device might require pairing or a different connection method.")

    log("\n========================================")
    log("BLE DISCOVERY COMPLETE")
    log(f"Log: {LOGFILE}")
    log("========================================")
    save_log()

if __name__ == "__main__":
    asyncio.run(main())
