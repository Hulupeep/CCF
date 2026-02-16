#!/usr/bin/env python3
"""
CyberPi Live Mode f3 Command Sweep

KNOWN:
  f3 0d 00 00 f4 = enter Live mode (gets WARNING response)
  f3 0d 00 01 f4 = enter Upload/REPL mode
  In Live mode, CyberPiOS is actively listening on serial.

THEORY: In Live mode, additional f3 command types become active for
sensor reads and motor control. Previous sweeps were done in the wrong
mode (default or upload). This sweep happens IN live mode.

Also try: text commands in live mode, since CyberPiOS seems to parse
serial input as commands.
"""

import serial
import time
import struct

SERIAL_PORT = "/dev/ttyUSB0"
BAUD = 115200
LOGFILE = "/tmp/cyberpi_live_sweep.log"
LOG = []

def log(msg):
    print(msg)
    LOG.append(msg)

def save_log():
    with open(LOGFILE, "w") as f:
        f.write("\n".join(LOG))

def hex_dump(data):
    return data.hex(' ') if data else "(empty)"

def main():
    log(f"=== CyberPi Live Mode f3 Sweep - {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")

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
    log("=== Reset CyberPi ===")
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    while ser.in_waiting:
        ser.read(ser.in_waiting)
        time.sleep(0.05)

    def send(data, label, wait=0.3):
        ser.reset_input_buffer()
        ser.write(data)
        time.sleep(wait)
        resp = b""
        while ser.in_waiting:
            resp += ser.read(ser.in_waiting)
            time.sleep(0.03)
        return resp

    # ==========================================
    # Enter LIVE mode
    # ==========================================
    log("=== Entering Live mode ===")
    resp = send(bytes([0xf3, 0x0d, 0x00, 0x00, 0xf4]), "live mode", 3.0)
    log(f"  Live mode response ({len(resp)}): {hex_dump(resp)}")
    if resp:
        text = resp.decode('utf-8', errors='replace')
        printable = ''.join(c if c.isprintable() or c in '\n\r' else '.' for c in text)
        if printable.strip():
            log(f"  TXT: {printable.strip()}")

    # ==========================================
    # SWEEP 1: f3 command types 0x00-0xff IN LIVE MODE
    # ==========================================
    log("\n=== SWEEP 1: f3 [type] 00 f4 in Live mode ===")
    responders = {}
    for cmd in range(0x00, 0x100):
        if cmd == 0x0d:  # Skip mode switch
            continue
        frame = bytes([0xf3, cmd, 0x00, 0xf4])
        resp = send(frame, f"f3 0x{cmd:02x}", 0.12)
        if resp:
            responders[cmd] = resp
            log(f"  0x{cmd:02x}: [{len(resp)}] {hex_dump(resp[:40])}")

    log(f"\n  Responders: {len(responders)}")
    if responders:
        for cmd, resp in sorted(responders.items()):
            log(f"    0x{cmd:02x}: {hex_dump(resp[:60])}")

    # ==========================================
    # SWEEP 2: f3 with 2-byte payloads
    # ==========================================
    log("\n=== SWEEP 2: f3 [type] [00|01|02] 00 f4 ===")
    for payload_byte in [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x08, 0x0d, 0x10, 0x20]:
        for cmd in range(0x00, 0x100):
            if cmd == 0x0d:
                continue
            frame = bytes([0xf3, cmd, payload_byte, 0x00, 0xf4])
            resp = send(frame, f"f3 {cmd:02x} {payload_byte:02x}", 0.08)
            if resp:
                if cmd not in responders:
                    responders[cmd] = resp
                log(f"  type=0x{cmd:02x} p1=0x{payload_byte:02x}: [{len(resp)}] {hex_dump(resp[:40])}")

    log(f"\n  Total responders: {len(responders)}")

    # ==========================================
    # SWEEP 3: Text commands in Live mode
    # ==========================================
    log("\n=== SWEEP 3: Text commands in Live mode ===")

    # CyberPiOS might have a text command interface in live mode
    text_cmds = [
        b"\r\n",
        b"help\r\n",
        b"status\r\n",
        b"version\r\n",
        b"sensor\r\n",
        b"read\r\n",
        b"get_loudness\r\n",
        b"get_brightness\r\n",
        b"mode\r\n",
        b"mode live\r\n",
        b"live\r\n",
        b"debug\r\n",
        # mBlock might send JSON
        b'{"type":"sensor","name":"loudness"}\r\n',
        b'{"cmd":"read"}\r\n',
        # Or URL-style
        b"GET /sensor/loudness\r\n",
        # Newline-terminated queries
        b"loudness\r\n",
        b"brightness\r\n",
        b"gyro\r\n",
    ]
    for cmd in text_cmds:
        resp = send(cmd, f"text: {cmd.strip()[:30]}", 0.5)
        if resp:
            log(f"  '{cmd.strip().decode(errors='replace')}': [{len(resp)}] {hex_dump(resp[:40])}")
            text = resp.decode('utf-8', errors='replace')
            printable = ''.join(c if c.isprintable() or c in '\n\r' else '.' for c in text)
            if printable.strip():
                log(f"    TXT: {printable.strip()[:100]}")

    # ==========================================
    # SWEEP 4: f3 with known mBlock patterns
    # ==========================================
    log("\n=== SWEEP 4: mBlock-style f3 commands ===")

    # From mBlock extension API: asyncWriteProtocol('f3f4', payload)
    # The f3f4 wrapper adds f3 header and f4 footer
    # What if sensor reads use specific payload patterns?
    mblock_patterns = [
        # Mode 0 sub-commands?
        bytes([0xf3, 0x0d, 0x01, 0x00, 0xf4]),  # 0d with extra byte
        bytes([0xf3, 0x0d, 0x02, 0x00, 0xf4]),
        bytes([0xf3, 0x0d, 0x03, 0x00, 0xf4]),
        bytes([0xf3, 0x0d, 0x04, 0x00, 0xf4]),
        bytes([0xf3, 0x0d, 0x05, 0x00, 0xf4]),
        # Sensor read patterns (maybe type=0x0e or nearby)
        bytes([0xf3, 0x0e, 0x01, 0x00, 0xf4]),
        bytes([0xf3, 0x0e, 0x02, 0x00, 0xf4]),
        bytes([0xf3, 0x0e, 0x03, 0x00, 0xf4]),
        bytes([0xf3, 0x0e, 0x04, 0x00, 0xf4]),
        # Maybe the f3 payload IS a nested protocol
        # f3 [len] [cmd] [subcmd] [data...] f4
        bytes([0xf3, 0x02, 0x01, 0x00, 0xf4]),  # read loudness?
        bytes([0xf3, 0x02, 0x02, 0x00, 0xf4]),  # read brightness?
        bytes([0xf3, 0x02, 0x03, 0x00, 0xf4]),  # read gyro?
        bytes([0xf3, 0x03, 0x01, 0x00, 0x00, 0xf4]),
        bytes([0xf3, 0x03, 0x02, 0x00, 0x00, 0xf4]),
        # What about very long payloads?
        bytes([0xf3, 0x0d, 0x00, 0x00, 0x01, 0x00, 0xf4]),
        bytes([0xf3, 0x0d, 0x00, 0x00, 0x02, 0x00, 0xf4]),
        # CyberPi uses ESPNOW for chassis communication
        # Maybe similar protocol over serial
        bytes([0xf3, 0xf7, 0x00, 0xf4]),
        bytes([0xf3, 0xf8, 0x00, 0xf4]),
        bytes([0xf3, 0xf9, 0x00, 0xf4]),
        bytes([0xf3, 0xfa, 0x00, 0xf4]),
        bytes([0xf3, 0xfb, 0x00, 0xf4]),
        bytes([0xf3, 0xfc, 0x00, 0xf4]),
        bytes([0xf3, 0xfd, 0x00, 0xf4]),
        bytes([0xf3, 0xfe, 0x00, 0xf4]),
    ]
    for pat in mblock_patterns:
        resp = send(pat, f"f3: {hex_dump(pat)}", 0.3)
        if resp:
            log(f"  TX: {hex_dump(pat)}")
            log(f"  RX: [{len(resp)}] {hex_dump(resp[:40])}")

    # ==========================================
    # SWEEP 5: Try re-entering live mode and waiting
    # ==========================================
    log("\n=== SWEEP 5: Re-enter live mode, long listen ===")

    # Reset
    ser.rts = True
    time.sleep(0.1)
    ser.rts = False
    time.sleep(5.0)
    while ser.in_waiting:
        ser.read(ser.in_waiting)
        time.sleep(0.05)

    # Enter live mode cleanly
    resp = send(bytes([0xf3, 0x0d, 0x00, 0x00, 0xf4]), "live mode", 2.0)
    log(f"  Live mode: [{len(resp)}] {hex_dump(resp[:40])}")

    # Now listen for 10 seconds - maybe CyberPi streams data in live mode
    log("\n  Listening for 10 seconds...")
    ser.reset_input_buffer()
    all_data = b""
    deadline = time.time() + 10.0
    while time.time() < deadline:
        if ser.in_waiting:
            chunk = ser.read(ser.in_waiting)
            all_data += chunk
            log(f"  RX @ {time.time()-deadline+10:.1f}s: [{len(chunk)}] {hex_dump(chunk)}")
        time.sleep(0.05)

    if all_data:
        log(f"\n  Total received: {len(all_data)} bytes")
        log(f"  Data: {hex_dump(all_data[:200])}")
    else:
        log("  No data received during listen period")

    # ==========================================
    # SWEEP 6: f5 handshake IN live mode
    # ==========================================
    log("\n=== SWEEP 6: f5/f6 handshake in live mode ===")
    hs = bytes([0xf3, 0xf5, 0x02, 0x00, 0x08, 0xc0, 0xc8, 0xf4])
    resp = send(hs, "f5 in live mode", 1.0)
    log(f"  f5 TX: {hex_dump(hs)}")
    log(f"  f5 RX: [{len(resp)}] {hex_dump(resp[:60])}")
    if resp:
        text = resp.decode('utf-8', errors='replace')
        printable = ''.join(c if c.isprintable() or c in '\n\r' else '.' for c in text)
        if printable.strip():
            log(f"  TXT: {printable.strip()[:100]}")

    # Did f5 change the mode? Try f3 sweep again
    log("\n  Quick re-sweep after f5...")
    for cmd in [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a,
                0x0e, 0x0f, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80,
                0x90, 0xa0, 0xb0, 0xf0, 0xf1, 0xf2, 0xf3, 0xf7, 0xf8, 0xf9]:
        frame = bytes([0xf3, cmd, 0x00, 0xf4])
        resp = send(frame, f"re-sweep 0x{cmd:02x}", 0.12)
        if resp:
            log(f"  0x{cmd:02x}: [{len(resp)}] {hex_dump(resp[:40])}")

    ser.close()

    # Summary
    log("\n" + "="*60)
    log("LIVE MODE SWEEP SUMMARY")
    log("="*60)
    log(f"f3 command types that responded: {len(responders)}")
    if responders:
        for cmd, resp in sorted(responders.items()):
            log(f"  0x{cmd:02x}: {hex_dump(resp[:50])}")
    else:
        log("  NONE - CyberPi does not respond to any f3 commands in live mode")
        log("\n  CyberPiOS in live mode appears to be a write-only channel.")
        log("  The device accepts commands but provides no serial feedback.")
        log("  mBlock likely uses a DIFFERENT transport for responses (WebSocket? BLE?)")

    log(f"\nLog: {LOGFILE}")
    log("="*60)
    save_log()

if __name__ == "__main__":
    main()
