#!/usr/bin/env python3
"""
CyberPi Dual-Channel Communication Test

KEY HYPOTHESIS: CyberPiOS routes REPL output to BLE, not serial!
mBlock uses BLE for wireless programming. CyberPiOS keeps BLE active.
Maybe stdout goes to BLE ffe2 (notify characteristic).

Strategy:
1. Reset CyberPi
2. Connect BOTH serial AND BLE simultaneously
3. Enter REPL via serial
4. Send Python commands via serial
5. Watch BLE notifications for output

Also test:
- Enter REPL via BLE, read output on serial
- Enter REPL via serial, explicitly write to BLE from Python
- MicroPython bluetooth module access
"""

import asyncio
import serial
import time
from bleak import BleakClient, BleakScanner

SERIAL_PORT = "/dev/ttyUSB0"
BAUD = 115200
DEVICE_ADDR = "10:97:BD:8F:4D:D2"
NOTIFY_UUID = "0000ffe2-0000-1000-8000-00805f9b34fb"
WRITE_UUID  = "0000ffe3-0000-1000-8000-00805f9b34fb"

LOGFILE = "/tmp/cyberpi_dual_channel.log"
LOG = []
BLE_RX = []

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

def on_ble_notify(sender, data):
    """BLE notification callback."""
    BLE_RX.append(data)
    log(f"  BLE <<< [{len(data)}] {data.hex(' ')}")
    text = data.decode('utf-8', errors='replace')
    printable = ''.join(c if c.isprintable() or c in '\n\r' else '.' for c in text)
    if printable.strip():
        log(f"  BLE TXT: {printable.strip()}")

def serial_send(ser, cmd, label, wait=1.0):
    """Send via serial, read serial response."""
    ser.reset_input_buffer()
    if isinstance(cmd, str):
        cmd = cmd.encode()
    ser.write(cmd)
    time.sleep(wait)
    resp = b""
    while ser.in_waiting:
        resp += ser.read(ser.in_waiting)
        time.sleep(0.05)
    return resp

async def ble_send(client, data, label, wait=1.0):
    """Send via BLE, wait for notifications."""
    global BLE_RX
    BLE_RX.clear()
    if isinstance(data, str):
        data = data.encode()
    await client.write_gatt_char(WRITE_UUID, data, response=False)
    await asyncio.sleep(wait)
    return b"".join(BLE_RX)

def hex_dump(data):
    if not data:
        return "(empty)"
    return data.hex(' ')[:120]

async def main():
    log(f"=== CyberPi Dual-Channel Test - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")
    log("HYPOTHESIS: Serial IN + BLE OUT = full bidirectional communication\n")

    # ==========================================
    # STEP 1: Open serial and reset CyberPi
    # ==========================================
    log("=== STEP 1: Open serial, reset CyberPi ===")
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

    # Reset
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    log("Reset pulse sent. Waiting for boot (5s)...")
    time.sleep(5.0)

    # Drain boot data
    boot = b""
    while ser.in_waiting:
        boot += ser.read(ser.in_waiting)
        time.sleep(0.05)
    log(f"Boot data: {len(boot)} bytes drained")

    # ==========================================
    # STEP 2: Connect BLE while serial is open
    # ==========================================
    log("\n=== STEP 2: Connect BLE (serial stays open) ===")
    log(f"Scanning for {DEVICE_ADDR}...")

    try:
        client = BleakClient(DEVICE_ADDR, timeout=15.0)
        await client.connect()
        log(f"BLE connected! MTU: {client.mtu_size}")
        await client.start_notify(NOTIFY_UUID, on_ble_notify)
        log("Subscribed to BLE ffe2 notifications")
        await asyncio.sleep(1.0)

        if BLE_RX:
            log(f"  Got {len(BLE_RX)} initial BLE notifications!")
        else:
            log("  No initial BLE data")

    except Exception as e:
        log(f"BLE connection failed: {e}")
        log("Continuing with serial-only...")
        ser.close()
        save_log()
        return

    # ==========================================
    # TEST 1: Enter REPL via serial, watch BLE
    # ==========================================
    log("\n=== TEST 1: Serial REPL entry, watch BLE for output ===")

    BLE_RX.clear()
    log("Sending 'mode upload' via serial...")
    resp_serial = serial_send(ser, b"mode upload\r\n", "mode upload", 3.0)
    log(f"  Serial RX: [{len(resp_serial)}] {hex_dump(resp_serial)}")
    await asyncio.sleep(1.0)  # Give BLE time
    if BLE_RX:
        log(f"  BLE got {len(BLE_RX)} notifications during 'mode upload'!")
    else:
        log("  BLE: no notifications")

    BLE_RX.clear()
    log("\nSending Ctrl+A via serial...")
    resp_serial = serial_send(ser, b"\x01", "Ctrl+A", 2.0)
    log(f"  Serial RX: [{len(resp_serial)}] {hex_dump(resp_serial)}")
    await asyncio.sleep(1.0)
    if BLE_RX:
        log(f"  BLE got {len(BLE_RX)} notifications during Ctrl+A!")
    else:
        log("  BLE: no notifications")

    # Check if REPL is up
    if b">>>" in resp_serial:
        log("  REPL confirmed via serial!")

    # ==========================================
    # TEST 2: Send Python via serial, watch BLE for output
    # ==========================================
    log("\n=== TEST 2: Python commands via serial, BLE output? ===")

    commands = [
        (b"print('HELLO_BLE')\r\n", "print test"),
        (b"2+2\r\n", "expression"),
        (b"import sys; print(type(sys.stdout))\r\n", "stdout type"),
        (b"print('SENSOR_TEST_12345')\r\n", "sensor marker"),
    ]

    for cmd, label in commands:
        BLE_RX.clear()
        log(f"\n  Sending via serial: {cmd!r}")
        resp_serial = serial_send(ser, cmd, label, 1.5)
        log(f"  Serial RX: [{len(resp_serial)}] {hex_dump(resp_serial)}")
        await asyncio.sleep(0.5)
        if BLE_RX:
            log(f"  *** BLE got {len(BLE_RX)} notifications! ***")
            for data in BLE_RX:
                text = data.decode('utf-8', errors='replace')
                if 'HELLO' in text or 'SENSOR' in text or '4' in text:
                    log(f"  *** OUTPUT RECEIVED VIA BLE! ***")
        else:
            log(f"  BLE: no notifications")

    # ==========================================
    # TEST 3: Send Python via serial that writes to BLE
    # ==========================================
    log("\n=== TEST 3: Python writes to BLE from serial REPL ===")

    ble_write_cmds = [
        # Try MicroPython bluetooth module
        b"try:\r\n import bluetooth; print('BT_OK')\r\nexcept: pass\r\n\r\n",
        # Try to access BLE GATT
        b"try:\r\n import ubluetooth; print('UBT_OK')\r\nexcept: pass\r\n\r\n",
        # CyberPi might have BLE send methods
        b"import cyberpi\r\n",
        b"try:\r\n d=[x for x in dir(cyberpi) if 'ble' in x.lower() or 'blue' in x.lower() or 'wire' in x.lower() or 'broad' in x.lower()]; exec('for x in d: machine.UART(0,115200).write((x+chr(10)).encode())')\r\nexcept: pass\r\n\r\n",
    ]

    for cmd in ble_write_cmds:
        BLE_RX.clear()
        log(f"\n  Sending: {cmd[:80]!r}...")
        resp_serial = serial_send(ser, cmd, "ble write", 2.0)
        log(f"  Serial RX: [{len(resp_serial)}] {hex_dump(resp_serial)}")
        await asyncio.sleep(0.5)
        if BLE_RX:
            log(f"  *** BLE got {len(BLE_RX)} notifications! ***")

    # ==========================================
    # TEST 4: Send REPL commands via BLE, read on serial
    # ==========================================
    log("\n=== TEST 4: BLE commands, serial output? ===")
    log("(CyberPi might be in REPL mode, try sending commands via BLE)")

    ble_commands = [
        (b"print('BLE_TO_SERIAL')\r\n", "print via BLE"),
        (b"3+3\r\n", "expr via BLE"),
        (b"import cyberpi; cyberpi.led.on(0,0,255)\r\n", "LED blue via BLE"),
    ]

    for cmd, label in ble_commands:
        BLE_RX.clear()
        ser.reset_input_buffer()
        log(f"\n  Sending via BLE: {cmd!r}")
        await ble_send(client, cmd, label, 2.0)
        # Check serial for response
        time.sleep(0.5)
        serial_data = b""
        while ser.in_waiting:
            serial_data += ser.read(ser.in_waiting)
            time.sleep(0.05)
        if serial_data:
            log(f"  *** SERIAL got response! [{len(serial_data)}] {hex_dump(serial_data)} ***")
            text = serial_data.decode('utf-8', errors='replace')
            if text.strip():
                log(f"  *** SERIAL TXT: {text.strip()} ***")
        else:
            log(f"  Serial: no response")
        if BLE_RX:
            log(f"  BLE got {len(BLE_RX)} notifications")

    # ==========================================
    # TEST 5: f3 handshake via BLE while in serial REPL
    # ==========================================
    log("\n=== TEST 5: f3 frames via BLE while serial REPL active ===")

    BLE_RX.clear()
    handshake = bytes([0xf3, 0xf5, 0x02, 0x00, 0x08, 0xc0, 0xc8, 0xf4])
    log(f"  Sending f5 via BLE: {handshake.hex(' ')}")
    resp = await ble_send(client, handshake, "f5 via BLE", 2.0)
    if BLE_RX:
        log(f"  BLE f5 response: YES! {len(BLE_RX)} notifications")
    # Check serial too
    time.sleep(0.3)
    serial_data = b""
    while ser.in_waiting:
        serial_data += ser.read(ser.in_waiting)
    if serial_data:
        log(f"  Serial during BLE f5: {hex_dump(serial_data)}")

    # ==========================================
    # TEST 6: Try LED to confirm BLE commands execute
    # ==========================================
    log("\n=== TEST 6: Confirm BLE command execution via LED ===")
    log("Sending LED red via BLE...")
    BLE_RX.clear()
    await ble_send(client, b"import cyberpi; cyberpi.led.on(255,0,0)\r\n", "LED red", 2.0)
    log("CHECK: Is LED RED? (confirms BLE commands execute)")
    if BLE_RX:
        log(f"  BLE notifications: {len(BLE_RX)}")

    await asyncio.sleep(2.0)
    await ble_send(client, b"cyberpi.led.off()\r\n", "LED off", 1.0)

    # ==========================================
    # TEST 7: Dual-write test (serial command that also writes to BLE)
    # ==========================================
    log("\n=== TEST 7: Serial Python that uses BLE module ===")
    ble_module_cmds = [
        # Access ESP32 BLE via MicroPython
        b"try:\r\n import bluetooth as bt; b=bt.BLE(); b.active(True); print('BLE_ACTIVE')\r\nexcept Exception as e: import machine; machine.UART(0,115200).write(str(e).encode()+'\\n')\r\n\r\n",
        # Try ubluetooth
        b"try:\r\n from ubluetooth import BLE as B; b=B(); b.active(True)\r\nexcept Exception as e: pass\r\n\r\n",
        # Try ESP-specific
        b"try:\r\n import network; s=network.WLAN(network.STA_IF); s.active(True)\r\nexcept: pass\r\n\r\n",
    ]

    for cmd in ble_module_cmds:
        BLE_RX.clear()
        ser.reset_input_buffer()
        log(f"\n  Sending via serial: {cmd[:70]!r}...")
        serial_send(ser, cmd, "BLE module", 2.0)
        await asyncio.sleep(1.0)
        if BLE_RX:
            log(f"  *** BLE notifications during BLE module test! ***")
        serial_data = b""
        while ser.in_waiting:
            serial_data += ser.read(ser.in_waiting)
        if serial_data:
            log(f"  *** Serial data: {hex_dump(serial_data)} ***")

    # ==========================================
    # TEST 8: Raw bytes over BLE (not text)
    # ==========================================
    log("\n=== TEST 8: Various BLE write patterns ===")
    patterns = [
        # What if BLE needs the f6 config before text works?
        bytes([0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x00, 0x0d, 0xf4]),
        # Then try text
        b"print('AFTER_F6')\r\n",
        # Maybe REPL entry over BLE
        b"mode upload\r\n",
        b"\x01",  # Ctrl+A
        b"print('AFTER_CTRL_A')\r\n",
    ]
    for pat in patterns:
        BLE_RX.clear()
        log(f"\n  BLE TX: {pat!r}")
        await ble_send(client, pat, "pattern", 1.5)
        if BLE_RX:
            log(f"  BLE RX: {len(BLE_RX)} notifications")
        time.sleep(0.3)
        if ser.in_waiting:
            d = ser.read(ser.in_waiting)
            log(f"  Serial: {hex_dump(d)}")

    # ==========================================
    # FINAL: Extended dual-channel listen
    # ==========================================
    log("\n=== FINAL: Extended listen on both channels (5s) ===")
    BLE_RX.clear()
    ser.reset_input_buffer()
    await asyncio.sleep(5.0)
    if BLE_RX:
        log(f"Late BLE: {len(BLE_RX)} notifications")
    if ser.in_waiting:
        d = ser.read(ser.in_waiting)
        log(f"Late serial: {hex_dump(d)}")
    if not BLE_RX and not ser.in_waiting:
        log("No late data on either channel")

    # Cleanup
    try:
        await client.stop_notify(NOTIFY_UUID)
        await client.disconnect()
    except:
        pass
    ser.close()

    # Summary
    log("\n" + "="*60)
    log("DUAL-CHANNEL SUMMARY")
    log("="*60)
    all_log = "\n".join(LOG)
    if "OUTPUT RECEIVED VIA BLE" in all_log:
        log("SUCCESS: REPL output found on BLE channel!")
    elif "SERIAL got response" in all_log:
        log("SUCCESS: BLE commands produced serial output!")
    elif "BLE notifications during" in all_log:
        log("PARTIAL: Some BLE activity detected during commands")
    else:
        log("NEGATIVE: No cross-channel output detected")
        log("\nCyberPiOS appears to capture output on ALL channels.")
        log("Remaining options:")
        log("  1. Flash Arduino firmware (PlatformIO) - replaces CyberPiOS")
        log("  2. Sniff mBlock protocol with Wireshark/serial sniffer")
        log("  3. Use mBlock to upload a TCP bridge program")
        log("  4. Accept write-only serial + use CyberPi sensors as black box")

    log(f"\nLog: {LOGFILE}")
    log("="*60)
    save_log()

if __name__ == "__main__":
    asyncio.run(main())
