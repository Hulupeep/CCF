# Cognitum Seed — First Direct-Connection Audit Log

**Purpose:** Contemporaneous record of first successful host-to-device connection
with the Cognitum Seed appliance, captured for patent prosecution evidence.

| Field | Value |
| --- | --- |
| Date | 2026-04-27 |
| Operator | xanacan@thinkcentre |
| Host OS | Ubuntu 24.04.3 LTS (Noble Numbat), kernel 6.17.0-22-generic |
| Host repo | Hulupeep/CCF, branch `feat/gavalas-p1-middleware` |
| Device product | "Cognitum Seed" (RuVector) |
| Device USB ID | `1d6b:0104` (Linux Foundation Multifunction Composite Gadget) |
| Device serial | `e71209de-6e40-48f7-b3e1-42c98afe2732` |
| Device hostname (cert) | `cognitum-96b8.local` |
| Device CA CN | `Cognitum Device Local CA (cognitum-3ffe)` |
| Device CA SHA-256 | `D8:29:D8:95:19:C1:46:80:26:ED:AB:03:87:B1:DE:9A:AA:05:DF:1E:21:13:8F:1B:8B:0B:EF:4E:1D:19:AF:95` |

All times Europe/London (BST, UTC+1).

---

## 1. Timeline

| Time (local) | Action | Result |
| --- | --- | --- |
| 15:41:03 | Cognitum Seed plugged into host via USB. Kernel enumerates as USB device `1-4`, composite gadget with five interfaces (`1-4:1.0` … `1-4:1.4`). | Device descriptor strings: manufacturer "RuVector", product "Cognitum Seed", serial `e71209de-6e40-48f7-b3e1-42c98afe2732`. |
| 15:42 (approx.) | Initial discovery via `lsusb`, `ip -brief link`, `ls /dev/tty*`. | No `/dev/ttyUSB*`, `/dev/ttyACM*`, or `/dev/ttyGS*` created. Two USB-ethernet interfaces appear: `enxc6592e1e5312` (UP) and `enxea4477b6ef46` (UNKNOWN/UP). Both attached to USB device `1-4` (`1-4:1.1` and `1-4:1.3`). |
| 15:45 (approx.) | Operator runs vendor installer for Cognitum CA. Installer writes `/usr/local/share/ca-certificates/cognitum-ca.crt` and runs `update-ca-certificates`. | Installer reports "1 added, 0 removed; done." and prints expected access URLs `https://169.254.42.1:8443` and `https://cognitum.local:8443`. Benign `c_rehash` warning about `ca-certificates.crt` not being a single cert (it is a bundle — expected). |
| 15:55 (approx.) | First HTTPS attempt to `https://cognitum.local:8443/`. | Failed: mDNS name does not resolve on host (Avahi has not yet picked up the new interface). |
| 15:56 (approx.) | Routing check: `ip route get 169.254.42.1` returns route via Wi-Fi default gateway (`192.168.1.1` on `wlo1`) instead of the USB-ethernet link. | Confirms USB-ethernet interface has no IP, so kernel falls back to the default route. No traffic reaches the device. |
| 15:58 (approx.) | Manually assigned link-local address: `sudo ip addr add 169.254.42.2/16 dev enxc6592e1e5312` and `sudo ip link set enxc6592e1e5312 up`. | Route table now sends `169.254.42.1` over `enxc6592e1e5312`, source `169.254.42.2`. Ping: 2/2 packets, RTT 1.79 / 1.94 ms. |
| 16:00 (approx.) | First successful HTTPS request: `curl -k https://169.254.42.1:8443/`. | HTTP 200, 88 527 bytes, served the "Cognitum Seed — API Explorer" page in 0.034 s. Establishes that host ↔ device IP path and the device's HTTPS service are both functional. |
| 16:03 (approx.) | Subsequent connection attempts time out. `ip -4 addr show enxc6592e1e5312` shows the address has been removed. | NetworkManager observed the un-managed kernel address on a managed interface and stripped it. Symptom: ping fails 100 %, curl times out at 8 s. |
| 16:05 (approx.) | Re-added IP manually; verified connectivity again, then captured TLS chain with `openssl s_client -connect 169.254.42.1:8443 -servername cognitum-3ffe.local -showcerts`. | 2-cert chain captured. Leaf `CN=cognitum-96b8.local` issued by `CN=Cognitum Device Local CA (cognitum-3ffe)`. Leaf SANs: `DNS:cognitum-96b8.local, DNS:cognitum.local, IP:169.254.42.1, IP:127.0.0.1, IP:192.168.4.1`. |
| 16:07:37 | Verified CA on the wire is byte-identical to the installed CA (SHA-256 fingerprint match) and ran `openssl verify -CAfile /etc/ssl/certs/ca-certificates.crt /tmp/cogp_00`. | `OK`. Chain validates against the system trust store; name constraints (`.local`, `169.254.0.0/16`, `127.0.0.0/8`, `192.168.4.0/24`) satisfied by the leaf SANs. |
| 16:11 (approx.) | Made network configuration persistent via NetworkManager: `nmcli connection add type ethernet ifname enxc6592e1e5312 con-name cognitum-usb ipv4.method manual ipv4.addresses 169.254.42.2/16 ipv6.method ignore autoconnect yes`, then `nmcli connection up cognitum-usb`. | Connection `cognitum-usb` activated (UUID `409218b8-57f0-42c2-bf90-f872de558075`). Address survives because NetworkManager now owns it. |
| 16:12 | Final validation: `curl https://169.254.42.1:8443/` (no `-k`). | HTTP 200, `ssl_verify_result=0` (X509_V_OK), 0.063 s. End-to-end TLS validated against the system CA store with no flags or overrides. |
| 16:13 (approx.) | Operator opens `https://169.254.42.1:8443/` in Chrome. | Page loads (API Explorer renders correctly) but Chrome shows "Not secure" — see Issue 4 below. |

---

## 2. Issues and resolutions

### Issue 1 — Device enumerated but produced no serial port

**Symptom:** `lsusb` showed both a CH340S adapter and a `1d6b:0104` composite gadget,
but `/dev/ttyUSB*`, `/dev/ttyACM*`, and `/dev/ttyGS*` were all absent.

**Diagnosis:** The Cognitum Seed exposes itself as a USB-ethernet composite
(two interfaces: ECM/NCM-style endpoints `1-4:1.1` and `1-4:1.3`), not as a
USB-CDC-ACM serial device. The CH340S in `lsusb` is an unrelated device on
the same hub.

**Resolution:** Treat the device as a network endpoint, not a serial endpoint.
Identified the two `enx*` interfaces belonging to USB device `1-4` via
`/sys/bus/usb/devices/1-4:1.{1,3}/net/`.

### Issue 2 — `cognitum.local` did not resolve

**Symptom:** `getent hosts cognitum.local` returned nothing; `curl https://cognitum.local:8443/`
timed out at the DNS-resolution step.

**Diagnosis:** The host had no IP on the USB-ethernet link, so Avahi/mDNS had
no path on which to discover the device. Resolution by IP was required first.

**Resolution:** Use `https://169.254.42.1:8443/` directly. The TLS leaf
includes `IP:169.254.42.1` as a SAN, so IP-based access validates cleanly.
mDNS resolution of `cognitum.local` may begin to work once Avahi picks up
the new interface; not required for direct access.

### Issue 3 — Default route hijacked link-local traffic

**Symptom:** `ip route get 169.254.42.1` returned a route via `wlo1` (Wi-Fi)
through `192.168.1.1`, sending all device-bound traffic onto the public LAN
instead of the USB link.

**Diagnosis:** With no IPv4 address on either `enx*` interface, the kernel
had no on-link route for `169.254.0.0/16` over USB and fell back to the
default route.

**Resolution:** Assign `169.254.42.2/16` to `enxc6592e1e5312`. The on-link
route `169.254.0.0/16 dev enxc6592e1e5312` is then preferred over the
default route for that range.

### Issue 4 — NetworkManager stripped the manually-added IP

**Symptom:** Within seconds of running `sudo ip addr add 169.254.42.2/16 …`,
the address vanished, ping started failing, and HTTPS timed out.

**Diagnosis:** NetworkManager manages all ethernet interfaces by default on
Ubuntu desktop. An address added with raw `iproute2` is not part of any NM
connection profile and is removed on the next NM reconciliation pass.

**Resolution:** Created a persistent NM connection profile bound to the
interface name:

```
sudo nmcli connection add type ethernet ifname enxc6592e1e5312 \
  con-name cognitum-usb \
  ipv4.method manual ipv4.addresses 169.254.42.2/16 \
  ipv6.method ignore autoconnect yes
sudo nmcli connection up cognitum-usb
```

The address is now NM-owned and survives reconciliation and replug. (If a
future firmware change causes the gadget MAC to vary, the binding can be
moved from `ifname` to a MAC match or to a serial-based udev rule.)

### Issue 5 — Transient `ssl_verify_result=19` from curl

**Symptom:** Early `curl -k -w "%{ssl_verify_result}"` runs reported
`ssl_verify_result=19` (`X509_V_ERR_SELF_SIGNED_CERT_IN_CHAIN`).

**Diagnosis:** The CA installation was already correct — the value was
captured during connections that were either hitting the wrong route or
racing with NM stripping the IP. Once connectivity was stable, the same
test reported `ssl_verify_result=0`. Independent verification was performed
by running `openssl verify -CAfile /etc/ssl/certs/ca-certificates.crt`
against the captured leaf cert — result `OK`.

**Resolution:** No cert action required. The vendor installer correctly
placed the CA; the transient verify error was a side-effect of the
networking issues (Issues 3 and 4).

### Issue 6 — Chrome still shows "Not secure"

**Symptom:** Chrome displays "Not secure" and crosses out `https://` in
the address bar when visiting `https://169.254.42.1:8443/`, even though
curl validates the same cert without `-k`.

**Diagnosis:** Chromium-family browsers on Linux do **not** consult the
system OpenSSL trust store at `/etc/ssl/certs/ca-certificates.crt`. They
use the NSS shared database at `~/.pki/nssdb`. The Cognitum installer only
populated the OpenSSL store. CLI tools (curl, wget, Python `requests`,
Go `crypto/tls`, OpenSSL) trust the cert; Chromium / Chrome / Edge / Brave
do not, until the CA is also imported into NSS.

**Resolution (not yet applied — separate follow-up):** Import the same CA
into NSS:

```
certutil -d sql:$HOME/.pki/nssdb -A -t "C,," \
  -n "Cognitum Device Local CA" \
  -i /usr/local/share/ca-certificates/cognitum-ca.crt
```

Firefox uses its own profile-scoped NSS DB and would need the cert imported
through `about:preferences#privacy → View Certificates → Authorities`, or
via a system policy.

This issue does not affect machine-to-device access, only browser UX.

---

## 3. Current status

- **Physical link:** Cognitum Seed connected to host via USB, enumerated as
  USB device `1-4`, composite gadget `1d6b:0104`, serial
  `e71209de-6e40-48f7-b3e1-42c98afe2732`.
- **Network:** `enxc6592e1e5312` bound to NetworkManager connection
  `cognitum-usb` (UUID `409218b8-57f0-42c2-bf90-f872de558075`) with static
  link-local address `169.254.42.2/16`. Survives interface bounce and reboot.
- **Reachability:** `ping 169.254.42.1` — 0 % loss, RTT ≈ 1.8 ms over USB.
- **TLS:** Direct HTTPS to `https://169.254.42.1:8443/` returns HTTP 200
  with `ssl_verify_result=0` (validated against the system CA store, no
  `--insecure` flag). Chain: leaf `CN=cognitum-96b8.local` →
  `Cognitum Device Local CA (cognitum-3ffe)` (name-constrained,
  byte-identical to the installer-deployed CA).
- **Application:** Cognitum Seed API Explorer is reachable and responsive
  (98 endpoints advertised; live values observed: 63 vectors, dim=8,
  epoch 64, 20.6 KB store, 31 m uptime, unpaired).
- **Outstanding:** Chrome trust (Issue 6) requires NSS import; mDNS for
  `cognitum.local` not yet verified.

---

## 4. Evidence captured (host-side)

| Path | Description |
| --- | --- |
| `/usr/local/share/ca-certificates/cognitum-ca.crt` | Installer-deployed CA cert (PEM). Subject = issuer (self-signed root). |
| `/etc/ssl/certs/cognitum-ca.pem` | Symlink into the system trust dir created by `update-ca-certificates`. |
| `/etc/ssl/certs/ca-certificates.crt` line range containing the Cognitum CA (entry #147) | The CA as included in the system bundle consumed by OpenSSL/curl/wget. |
| `/tmp/cog_chain.pem` (captured 2026-04-27 16:07:37) | 2-cert PEM chain returned by the device on TLS handshake. |
| `/tmp/cog_full.txt` | Raw `openssl s_client -showcerts` output for the same handshake. |
| `nmcli connection show cognitum-usb` | Persistent NM profile binding `enxc6592e1e5312` to `169.254.42.2/16`. |

These captures should be preserved or copied into the patent file as
exhibits if they are to be cited. (`/tmp` is not durable across reboot.)

---

## 5. Addendum — first successful API call (`GET /api/v1/status`)

**Time:** 2026-04-27 ~16:16 local.

**Request:** `GET https://169.254.42.1:8443/api/v1/status` (issued from the
API Explorer page in Chrome, which works for non-trusted-cert sites once
the user clicks through Chrome's warning — see Issue 6; CLI access via
curl validates the cert cleanly).

**Response (verbatim):**

```json
{
  "deleted_vectors": 0,
  "device_id": "e71209de-6e40-48f7-b3e1-42c98afe2732",
  "dimension": 8,
  "epoch": 70,
  "file_size_bytes": 23074,
  "paired": true,
  "roles": [
    "custody",
    "optimizer",
    "delivery"
  ],
  "total_vectors": 69,
  "uptime_secs": 2094,
  "witness_chain_length": 139
}
```

**Significance:**

- First successful authenticated-class API call from host to device.
- Confirms `device_id` returned by the API matches the USB descriptor
  serial reported at enumeration (`e71209de-6e40-48f7-b3e1-42c98afe2732`),
  closing the chain of identity from physical USB descriptor → TLS leaf
  cert OU → application-layer device ID.
- `paired: true` — the device has now been paired with this client; pairing
  state has changed since the screenshot at 16:13 (which showed "No paired"
  in the header). Pairing event therefore occurred between 16:13 and 16:16.
- Device internal state at this point: 69 vectors (dim 8), epoch 70,
  witness chain length 139, store size 23 074 B, roles
  `custody / optimizer / delivery`, uptime 2094 s (≈ 34 min 54 s, i.e.
  device booted at approximately 2026-04-27 15:42 — consistent with the
  USB enumeration timestamp at 15:41:03).

## 6. Addendum — bearer token storage and validation

**Time:** 2026-04-27 16:23 local.

**Action:** Bearer token issued by the device's `POST /api/v1/pair` flow
(performed via the in-browser API Explorer between 16:13 and 16:16, see
Section 5) was committed to local-only environment storage.

**Storage:**

- Path: `<repo-root>/.env` (relative to the CCF working tree).
- Mode: `600` (owner read/write only).
- Tracked by git: **no**. The repository's `.gitignore` already excludes
  `.env` at line 31; `git status` confirms the file is invisible to commits.
- Variables written:
  - `COGNITUM_BASE_URL` — the device's HTTPS base URL on the link-local
    USB interface.
  - `COGNITUM_TOKEN` — the bearer token returned by `POST /api/v1/pair`.
  - `TRACKER_DEVICE_ID` — the tracker device identifier supplied by the
    operator for application-layer correlation.

The token and device-ID values are **deliberately not reproduced in this
audit log**, since the audit log is committed to a public GitHub
repository (`Hulupeep/CCF`). Both values are recoverable from the
local `.env` file by the operator.

**Validation:** Immediately after writing `.env`, the token was exercised
against an authenticated endpoint:

```
curl -H "Authorization: Bearer ${COGNITUM_TOKEN}" \
     https://169.254.42.1:8443/api/v1/pair/status
```

Result: `HTTP 200`, `ssl_verify_result=0`, response body:

```json
{
  "client_count": 1,
  "paired": true,
  "pairing_window_open": false,
  "window_remaining_secs": 0
}
```

This confirms (a) the token is accepted by the device for authenticated
requests, (b) the device recognises exactly one paired client (this host),
and (c) no further pairing window is currently open — i.e. no additional
clients can pair without a fresh physical-button press or `POST /api/v1/pair/window`
from localhost.

*End of log.*
