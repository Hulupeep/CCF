#!/usr/bin/env python3
"""
CyberPi BLE Bridge Test

WiFi is disabled in upload mode, but BLE might still work.
ESP32's BLE and WiFi use the same radio but can operate independently.

Plan:
1. Enter REPL
2. Try to import bluetooth module and activate BLE
3. Create a GATT service that streams sensor data
4. Companion connects via BLE to read sensors

Also tests: sending exec code WITHOUT entering upload mode first.
"""

import serial
import time
import subprocess

PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_ble.log"

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
    ser.reset_input_buffer()
    ser.write(b"mode upload\r\n")
    time.sleep(2.0)
    ser.reset_input_buffer()
    ser.write(b"\x01")
    time.sleep(2.0)
    resp = read_all(ser, 3.0)
    return b">>>" in resp

def send(ser, cmd, delay=0.5):
    ser.write(cmd.encode() + b"\r\n")
    time.sleep(delay)
    return read_all(ser, 0.3)

def main():
    with open(LOGFILE, "w") as f:
        f.write(f"=== CyberPi BLE Bridge Test - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n\n")

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
    log("=== Reset ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    ser.reset_input_buffer()

    # ========================================
    # TEST A: Try BLE from REPL
    # ========================================
    log("\n=== TEST A: BLE from REPL ===")
    if enter_repl(ser):
        log("REPL ready")

        # LED magenta = testing BLE
        send(ser, "import cyberpi; cyberpi.led.on(255, 0, 255)")

        # Try to import and activate BLE
        ble_setup = [
            "import bluetooth",
            "ble = bluetooth.BLE()",
            "ble.active(True)",
            # If we get here, BLE is working
            "cyberpi.led.on(0, 255, 0)",  # Green = BLE active
        ]
        for cmd in ble_setup:
            log(f"  Sending: {cmd}")
            resp = send(ser, cmd, 0.8)
            if resp:
                log(f"    Response: {resp!r}")

        log("  LED should be GREEN if BLE activated, MAGENTA if it failed")
        time.sleep(1.0)

        # Set up BLE GATT server with sensor characteristic
        log("\n  Setting up BLE GATT service...")
        gatt_setup = [
            "import struct",
            # Define service UUID and characteristic
            "_SENSOR_UUID = bluetooth.UUID('12345678-1234-5678-1234-56789abcdef0')",
            "_SENSOR_CHAR = (bluetooth.UUID('12345678-1234-5678-1234-56789abcdef1'), bluetooth.FLAG_READ | bluetooth.FLAG_NOTIFY,)",
            "_SENSOR_SERVICE = (_SENSOR_UUID, (_SENSOR_CHAR,),)",
            # Register GATT service
            "((sensor_handle,),) = ble.gatts_register_services((_SENSOR_SERVICE,))",
            # Start advertising
            "ble.gap_advertise(100000, b'\\x02\\x01\\x06\\x05\\x09mBot')",
            # Yellow LED = advertising
            "cyberpi.led.on(255, 255, 0)",
        ]
        for cmd in gatt_setup:
            log(f"  Sending: {cmd}")
            resp = send(ser, cmd, 0.5)
            if resp:
                log(f"    Response: {resp!r}")

        log("\n  BLE GATT setup sent. LED should be:")
        log("    YELLOW = BLE advertising as 'mBot'")
        log("    GREEN  = BLE active but GATT failed")
        log("    MAGENTA = BLE import/activation failed")

        # Start sensor streaming loop
        log("\n  Starting sensor data write loop...")
        sensor_loop = (
            "import time\n"
            "while True:\n"
            "  try:\n"
            "    snd = cyberpi.get_loudness()\n"
            "    lgt = cyberpi.get_brightness()\n"
            "    data = struct.pack('<HH', snd, lgt)\n"
            "    ble.gatts_write(sensor_handle, data)\n"
            "    ble.gatts_notify(0, sensor_handle, data)\n"
            "  except: pass\n"
            "  time.sleep_ms(50)\n"
        )
        # Send as exec() one-liner
        exec_cmd = "exec('" + sensor_loop.replace("'", "\\'").replace("\n", "\\n") + "')"
        log(f"  Sending sensor loop ({len(exec_cmd)} chars)...")
        send(ser, exec_cmd, 1.0)

        log("\n  Sensor streaming should be running now.")
        log("  Scanning for BLE device 'mBot'...")

        time.sleep(3.0)

    else:
        log("Could not enter REPL")

    ser.close()

    # ========================================
    # TEST B: BLE scan from laptop
    # ========================================
    log("\n=== TEST B: BLE scan from laptop ===")
    try:
        # Use bluetoothctl to scan
        result = subprocess.run(
            ["timeout", "5", "bluetoothctl", "scan", "on"],
            capture_output=True, text=True, timeout=10
        )
        log(f"  BLE scan output:\n{result.stdout[:500]}")
    except Exception as e:
        log(f"  bluetoothctl scan failed: {e}")

    # Also try hcitool
    try:
        result = subprocess.run(
            ["timeout", "5", "hcitool", "lescan"],
            capture_output=True, text=True, timeout=10
        )
        for line in result.stdout.split('\n'):
            if 'mbot' in line.lower() or 'mBot' in line or 'cyber' in line.lower():
                log(f"  *** FOUND BLE DEVICE: {line} ***")
        log(f"  hcitool output ({len(result.stdout)} chars)")
    except Exception as e:
        log(f"  hcitool scan failed: {e}")

    # Try Python BLE scan
    try:
        log("\n  Trying Python BLE scan (bleak)...")
        import asyncio

        async def scan():
            try:
                from bleak import BleakScanner
                devices = await BleakScanner.discover(timeout=5.0)
                for d in devices:
                    if 'mbot' in (d.name or '').lower() or 'cyber' in (d.name or '').lower():
                        log(f"  *** FOUND: {d.name} ({d.address}) RSSI:{d.rssi} ***")
                    elif d.name:
                        log(f"    BLE: {d.name} ({d.address})")
                if not devices:
                    log("    No BLE devices found")
            except ImportError:
                log("    bleak not installed. pip install bleak")
            except Exception as e:
                log(f"    BLE scan error: {e}")

        asyncio.run(scan())
    except Exception as e:
        log(f"  Python BLE scan failed: {e}")

    log("\n========================================")
    log("BLE BRIDGE TEST COMPLETE")
    log(f"Log: {LOGFILE}")
    log("========================================")

if __name__ == "__main__":
    main()
