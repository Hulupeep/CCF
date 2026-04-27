# Phase 0 Verification & API Mapping — Cognitum Seed

**Date:** 2026-04-27
**Operator:** xanacan@thinkcentre
**Device under test:** Cognitum Seed, Pi Zero 2 W class, firmware 0.10.11
**Network:** USB-C link-local, host `169.254.42.2/16` ↔ device `169.254.42.1:8443`
**Companion artifacts (this folder):**

- `2026-04-27-cognitum-seed-first-connection.md` — connection / pairing audit
- `api-probe-results.json` — verbatim responses from every probed endpoint
- `api-from-explorer.txt` — 101 endpoint paths extracted from the API Explorer page
- `phase0_ingest_probe.py` — end-to-end ingest probe source
- `phase0_ingest_log.json` — verbatim request/response trail of the probe

---

## 1. Device fingerprint

From `GET /api/v1/identity` and `GET /api/v1/status`:

| Field | Value |
| --- | --- |
| `device_id` | `e71209de-6e40-48f7-b3e1-42c98afe2732` |
| `firmware_version` | `0.10.11` |
| `dimension` | 8 (RVF stride) |
| `roles` | `["custody","optimizer","delivery"]` |
| `paired` | `true` |
| `public_key` (Ed25519, PEM) | `MCowBQYDK2VwAyEAnKEQceZ9cZL1GY4egeqwk6goQSCnXo0p7PciQZOqJPs=` |
| `epoch` (snapshot at start of session) | 70 |
| `total_vectors` (snapshot) | 69 |
| `witness_chain_length` (snapshot) | 139 |

**Identity chain confirmed.** USB descriptor serial == TLS leaf cert OU ==
API-returned `device_id` == Ed25519 attestation `device_id`. Four
independent layers, one identity.

---

## 2. Token acquisition outcome

The device was already paired by the time the API session began (paired
state was set during the user's interaction with the in-browser API
Explorer between 16:13 and 16:16 local). The bearer token was extracted
from the API Explorer's response panel, written to the local
`<repo-root>/.env` file (mode 600, gitignored), and validated against
`GET /api/v1/pair/status`:

```json
{"client_count":1, "paired":true, "pairing_window_open":false, "window_remaining_secs":0}
```

Detail in Section 6 of the connection audit log. Token never reproduced
in this report, in commit messages, or in any committed file.

---

## 3. Documented endpoint behaviours

All 14 endpoints listed in the brief were probed. Verbatim responses are
stored in `api-probe-results.json`.

### 3.1 Read endpoints (no auth)

| Endpoint | HTTP | Notes |
| --- | --- | --- |
| `GET /api/v1/status` | 200 | Snapshot of vector/witness/epoch/uptime/paired state. Roles list. |
| `GET /api/v1/identity` | 200 | UUID, firmware version, dimension, **PEM-encoded Ed25519 public key**. |
| `GET /api/v1/pair/status` | 200 | `client_count, paired, pairing_window_open, window_remaining_secs`. |
| `GET /api/v1/store/status` | 200 | Adds `profile_id`, `dead_space_ratio` to the status surface. |
| `GET /api/v1/store/sync` | 200 | Full corpus dump as `vectors:[[u64_id, [f32 x dim]], …]` plus `current_epoch`, `from_epoch`, `is_delta`, `deletions:[]`. **Pull-based replication primitive.** |
| `GET /api/v1/store/deleted` | 200 | `count:0, deleted_ids:[], epoch`. |
| `GET /api/v1/store/graph/stats` | 200 | `nodes, total_edges, avg_degree, orphan_count, dirty_count, pending_inserts, mean_max_sim, core_count, boundary_count`. **Note `core_count`/`boundary_count` — this is the partition cardinalities surface.** |
| `GET /api/v1/store/vectors/{id}` | 200 / 404 | 200 for any of the u64 IDs returned by `/store/sync`; 404 (`{"error":"vector N not found"}`) for non-existent IDs. **404 is per-ID lookup miss**, NOT "endpoint missing". |

### 3.2 Write endpoints (bearer auth)

| Endpoint | HTTP | Notes |
| --- | --- | --- |
| `POST /api/v1/store/query` | 200 | `metric:"cosine"` confirmed. Echoes `k`, returns `total_searched` (= corpus size), `filtered:bool`, `results:[{distance, id, metadata}]`. **Cosine distance, k client-supplied per call.** |

### 3.3 Per-vector probe — important findings

- The brief's request to read `/api/v1/store/vectors/0..9` was based on an
  assumed sequential ID scheme. **The Seed does not use sequential IDs.**
  Every stored vector has a u64 ID — appears content-derived (likely a
  hash). Real IDs were obtained from `/store/sync` (e.g.
  `10302729586686596762`, `7540844832768980995`, …).
- **All 90+ pre-existing vectors carry the payload `[0,0,0,0,0,0,0,0]`.**
  They are not application data; they are **per-epoch witness anchors**.
  Each carries a single metadata field:

  ```
  metadata[0].value.String =
      "cognitive:epoch=N,hash=<sha256>"
  ```

  with `slot:N`. So the pre-load is a chain of cognitive epoch markers,
  not a vocabulary of features. The mBot2 corpus is going to overwrite
  the *signal* on top of an existing *clock*.

### 3.4 Rate-limit caveat

The device's rate limiter returns `HTTP 429 {"error":"rate limited — retry after 1s"}`
for **both** real-but-throttled paths and **non-existent** paths during
high-frequency probing. **In other words: 429 is not informative about
endpoint existence.** Always re-probe with ≥1.2 s spacing before
recording a 404. The bridge must implement exponential back-off on 429
or it will mistake throttling for missing routes.

---

## 4. Undocumented endpoints discovered

The API Explorer page (`GET /`) lists every route. After fixing a regex,
**101 paths** were extracted into `api-from-explorer.txt`. The 14 paths
in the brief account for ~14 % of the surface. Highlights of the other
~87:

### 4.1 Cognitive layer (the patent-relevant surface)

| Path | Method | Auth | Probed? | Behaviour |
| --- | --- | --- | --- | --- |
| `/api/v1/cognitive/status` | GET | no | ✓ | `chain_valid:{chain_length, first_epoch, last_epoch, valid}, decision, latest_tick_us, spectral_scs, tick_count`. |
| `/api/v1/cognitive/snapshot` | GET | no | ✓ | `config:{instance_id, max_receipts, slab_size}, epoch, evidence_accumulated, graph_edges, spectral_scs`. |
| `/api/v1/cognitive/config` | GET | no | ✓ | `enabled, max_receipts, slab_size, tick_every_n_cycles, total_ticks`. |
| `/api/v1/cognitive/witness/chain` | GET | no | ✓ | **Full Merkle chain.** Each entry `{epoch, prev_hash, receipt_hash, decision, spectral_scs}`. Genesis `prev_hash = 0×32`. Forward-verifiable. |
| `/api/v1/cognitive/tick` | (POST?) | likely | not probed (write) | Manual tick trigger. |
| `/api/v1/coherence/profile` | GET | no | ✓ | `coherence_trend, fragility_profile, global_rupture, pending_boundaries, phase_boundaries, slice_count, slice_indices, slices_needed, temporal_coherence, mono_timestamps_us, wall_timestamps_us, is_partial`. **This is the temporal-coherence / fragility / rupture surface.** Needs ≥3 slices (firmware constant). |
| `/api/v1/coherence/profile/history` | GET | no | ✓ | `count, max_history:10, profiles:[]`. Sliding history of profiles. |
| `/api/v1/coherence/orphans` | GET | no | ✓ | `count, orphan_ids, orphan_threshold:0.1, window_max:60, window_slices`. |
| `/api/v1/coherence/phases` | GET | no | ✓ | `count, epoch_range, phase_boundaries, temporal_coherence`. |
| `/api/v1/coherence/config` | GET | no | ✓ | 404. (Listed in HTML but not implemented, or moved.) |
| `/api/v1/boundary` | GET | no | ✓ | `{"hint":"POST /api/v1/boundary/recompute to trigger analysis", "knn_nodes":0, "status":"no boundary analysis yet"}`. **The min-cut/partition trigger.** |
| `/api/v1/boundary/recompute` | POST | unknown | not probed (write) | Manual partition recompute. |
| `/api/v1/custody/attestation` | GET | no | ✓ | **Ed25519-signed snapshot** of `{device_id, epoch, witness_head, total_vectors, timestamp, proof_verification:{enabled, errors, proofs_constructed, proofs_verified, reduction_steps, cache_hit_rate_bps}}` plus `public_key`, `signature`. |
| `/api/v1/custody/epoch` | GET | no | ✓ | Lightweight: `{epoch, witness_head}`. |
| `/api/v1/custody/sign` `/verify` `/witness` | various | unknown | not probed | Custody primitives. |
| `/api/v1/witness/chain` | GET | no | ✓ | `{depth, epoch, head_hash}` — chain head summary, separate from per-receipt chain. |
| `/api/v1/optimize/status` | GET | no | ✓ | `compaction_threshold:0.2, optimization_interval_secs:3600` — **auto-compaction every hour at 20 % dead-space**. |
| `/api/v1/optimize/metrics` | GET | no | ✓ | Per-epoch optimisation metrics. |

### 4.2 Sensor / I/O layer

| Path | Method | Behaviour |
| --- | --- | --- |
| `/api/v1/sensor/list` | GET | `enabled:true, healthy:true, sensors:[{name:"reed-switch", pin:5, type:"gpio", channels:1}, {name:"pir-motion", pin:6, …}, {name:"vibration", pin:13, …}, …]`. **Pi GPIO sensors enumerated.** |
| `/api/v1/sensor/embedding/latest` | GET | **45-dim vector**, `{dimension:45, gate_passed, hd_distance, timestamp_us, vector:[…]}`. The sensor pipeline produces 45-d (≠ store's 8-d). The Seed's RVF dimension and its sensor-embedding dimension are independent. |
| `/api/v1/sensor/{stream,gpio/pins,actuators,reflex/rules,drift/{status,history},store/status,coprocessor/status,embedding/config}` | various | All listed in HTML; not probed in Phase 0. |

### 4.3 Other surfaces (listed, not probed)

- **Mesh / swarm:** `/mesh/{status,peers,enable,disable,security}`, `/swarm/{status,peers}`, `/peers`.
- **Apps / cogs:** `/apps`, `/apps/available`, `/apps/install`, `/apps/{id}/start`. Currently `installed:[]`.
- **Thermal:** 12 endpoints under `/thermal/*` — DVFS profile, governor, turbo, silicon profile, characterize, telemetry, accuracy, coherence, boost, stats, state, config. **Bridge can passively read `/thermal/state` and `/thermal/telemetry` to correlate cognitive load with thermal headroom.**
- **OTA / firmware:** `/firmware/{status,update}`, `/upgrade/{check,apply,status}`, `/ota/{config,log}`.
- **Wifi:** `/wifi/{status,scan,connect}`. Currently `state:"disconnected"`. Scan returned three nearby SSIDs.
- **Security:** `/security/{status,build-key}`. Status: `auth_method:pairing_flag, dm_verity:configured, measured_boot:true, slot_ota:true, rollback_support:true, build_key_configured:false`.
- **Demos:** `/demo/coherence`, `/demo/ingest-sample`. Useful for future regression-testing the bridge against canonical loads.
- **Profiles, delivery, delta:** `/profiles`, `/delivery/image`, `/delta/history`, `/delta/stream`.

---

## 5. kNN graph & partition structure — verdict

**Question 1 — Does graph stats expose Stoer-Wagner min-cut output?**

Indirectly. `/api/v1/store/graph/stats` exposes the partition
*cardinalities* (`core_count`, `boundary_count`) and quality summary
(`mean_max_sim`, `avg_degree`). The actual cut value and partition
positions live on `/api/v1/coherence/profile`:

- `global_rupture` — the cut value (currently `null` because the corpus is
  degenerate).
- `phase_boundaries` — the cut positions in the temporal stream.
- `pending_boundaries` — boundaries computed but not yet committed.

Recompute is **manually triggerable** via `POST /api/v1/boundary/recompute`.

**Question 2 — Cluster IDs / partition assignment per vector?**

Partial. There is no `cluster_id` field on `GET /store/vectors/{id}`. The
Seed exposes:

- `coherence/orphans → orphan_ids:[u64]` — vectors NOT assigned to any cluster.
- `coherence/profile.phase_boundaries` + `slice_indices` — implicit
  cluster membership derivable by binning vectors into the temporal
  slices defined by the boundaries.

So per-vector cluster ID must be **derived client-side from the boundary
positions**. The Seed doesn't decorate individual vectors with their
partition assignment.

**Question 3 — Cognitive container / fragility scoring (RuView ADR-069)?**

Both, distinctly:

- **Cognitive container:** `/api/v1/cognitive/snapshot` exposes
  `config:{instance_id, max_receipts:1000, slab_size:524288}`,
  `evidence_accumulated`, `graph_edges`, `spectral_scs`, `epoch`.
  This matches "container" semantics: a slab-allocated cognitive working
  set bounded by `max_receipts` and `slab_size`.
- **Fragility scoring:** `/api/v1/coherence/profile.fragility_profile` is
  an array. Currently empty; populates when ≥3 temporal slices exist.

**Question 4 — kNN k, distance metric?**

- **Metric:** `cosine` (server-echoed: `"metric":"cosine"`). Confirmed
  twice (zero-vector query and self-similarity query).
- **k:** **Client-supplied per query.** Server echoes the k it used
  (we sent 5, it returned k:5). The graph's internal k (the kNN
  pre-computed degree) is not directly exposed; observed `avg_degree:0`
  with `nodes:104` in the degenerate case. Will be visible once the
  corpus carries diverse non-zero vectors.

**Architectural implication for CCF:**

> Because the Seed already emits `global_rupture` and `phase_boundaries`,
> and because the CA-signed `custody/attestation` already attests epoch
> + witness head + total_vectors, **the CCF crate can treat the Seed's
> partition output as authoritative** rather than re-implementing
> Stoer-Wagner client-side. The CCF code path should:
>
> 1. Read `/coherence/profile` and `/coherence/orphans` for partition
>    state.
> 2. Sign-verify a `custody/attestation` snapshot at each Phase 1
>    decision boundary, treating it as the canonical "what the device
>    saw at epoch N" record.
> 3. Implement local re-partition only as a divergence-detection check
>    (does my client-side cut agree with `global_rupture`?), not as
>    primary logic.
>
> This is a different code shape than what the brief assumed (CCF
> computes its own partition from raw vectors). **Recommend revisiting
> Phase 1 architecture.**

---

## 6. SSH introspection — incomplete

`ssh genesis@169.254.42.1` was attempted and was reachable (mDNS now
resolves `cognitum-96b8.local` → `169.254.42.1`), but auth requires
either a password or a deployed public key, neither of which is in scope
for this session. SSH was deferred. Recommended follow-up: deploy
`~/.ssh/id_ed25519.pub` into the Seed's `genesis` user `authorized_keys`
during the next interactive maintenance window, then re-run the
introspection bundle:

```
uname -a
cat /etc/os-release
head -20 /proc/cpuinfo
i2cdetect -y 1
ls /opt/cognitum/
systemctl list-units --type=service | grep -i cognit
cat /proc/loadavg
```

The HTTP API surface alone has been generous enough to characterise
Phase 0 — SSH would primarily confirm the OS / hardware / I2C bus
state, not the application semantics.

---

## 7. End-to-end ingest test — succeeded

Probe source: `audit/phase0_ingest_probe.py`. Full request/response
trail: `audit/phase0_ingest_log.json`.

**Pre-state:** `total_vectors=103, witness_chain_length=207, graph_nodes=0`.

**Steps performed:**

1. `POST /api/v1/store/ingest` with body
   `{"vectors":[[9999000420260427, [0.5,-0.3,0.7,0.1,-0.2,0.4,-0.1,0.6]]]}`
   → **HTTP 200**, response
   `{"accepted":true, "count":1, "new_epoch":105, "rejected":0, "verified":false, "witness_head":"e4abbf6dc88b3e712c4a829233c58d1a05b058a840d6fa34ed99ea142644eb94"}`.

2. `PUT /api/v1/store/vectors/9999000420260427/metadata` with three
   string-typed fields → **HTTP 200**,
   `{"id":9999000420260427, "new_epoch":106, "updated":true, "witness_head":"115bd462982e81d1618156670e477d9e4db4c265d895a1c129bfffc874781418"}`.
   **Each metadata edit advances the witness chain** — relevant for
   audit semantics.

3. `GET /api/v1/store/vectors/9999000420260427` → **HTTP 200** with the
   vector data and metadata reflected back verbatim, plus
   `slot:103, deleted:false`.

4. `POST /api/v1/store/query` with the ingested vector and `k:5` →
   **HTTP 200**, `metric:"cosine", total_searched:104, results:[{id:9999000420260427, distance:0.0, metadata:[…]}]`.
   Self-match at distance 0.0 confirmed. (Only one result returned despite
   `k:5` because every other vector is zero-norm and cosine-undefined
   relative to the query.)

5. After ~10 s, `GET /api/v1/store/graph/stats` →
   `nodes:104, total_edges:0, orphan_count:104, pending_inserts:0, mean_max_sim:-1.007874`.
   **Graph is built lazily** — `pending_inserts` was 2 immediately after
   the writes and drained to 0 within ten seconds. All 104 nodes are
   orphans because the corpus is degenerate (103 zero-vectors + 1 outlier).

6. `GET /api/v1/coherence/profile` →
   `epoch_range:[96,107], slice_count:5, temporal_coherence:1.0, phase_boundaries:[], is_partial:true, fragility_profile:[]`.
   Temporal pipeline is active (5 slices computed, ≥3 needed) but
   produces a degenerate output because input data is uniform.

**Vector ID assigned:** `9999000420260427` (client-supplied, accepted by
the server; the API does not auto-generate IDs — the client sets them).

**Round-trip latency observations** (single trials, throttled by
1.2 s sleeps to respect the rate limiter):

- Ingest: < 200 ms.
- Metadata update: < 200 ms.
- Readback: < 100 ms.
- Query: < 200 ms.
- Graph rebuild lag after a write: ~10 s.

---

## 8. Open questions / things that didn't behave as expected

1. **`verified:false` in the ingest response.** What does it mean? Is the
   vector pending some downstream verification step (e.g. signature
   check, attestation co-sign)? Worth probing on next session: does it
   eventually flip to `true`?
2. **No vector exposes its cluster ID directly.** Whole-corpus partition
   is computable but per-vector membership requires client-side derivation
   from boundaries + slice indices. Confirm with the vendor whether this
   is intentional or whether a `cluster_id` field is roadmap.
3. **Client-supplied vector IDs.** This is unusual for a vector store.
   What collision behaviour applies if the client supplies a duplicate
   ID? Does the server reject, overwrite, or version? Not tested.
4. **Why the autonomous tick rate ≈ 1 vector per ~10 s** even with no
   sensor input wired to the relevant pins? Is there an idle generator,
   or is one of the listed GPIO sensors wired and triggering?
5. **`/api/v1/coherence/config` returns 404** despite being listed in the
   API Explorer page. Either dropped from the firmware or moved.
6. **`mean_max_sim:-1.007874`** — looks like a sentinel for "no real
   similarity computed yet" but the value is suspiciously close to
   `-1 - epsilon`. Worth confirming with the vendor.
7. **The "98 endpoints" the device advertises vs. 101 paths the regex
   extracted.** The discrepancy is template paths (`{id}` placeholders)
   counted twice — accurate count probably 95–98 unique endpoints. Not
   critical.

---

## 9. Recommendations for Phase 1 architecture

1. **Treat the Seed's partition output as authoritative.** Don't
   re-implement Stoer-Wagner client-side as a primary code path. Read
   `/coherence/profile.{global_rupture, phase_boundaries, fragility_profile}`
   and `/coherence/orphans` directly. Implement client-side partition
   only as a *divergence sentinel* — disagreement between client cut and
   `global_rupture` is a signal worth surfacing, not the basis of
   decisions.

2. **The witness chain is the ledger, the custody attestation is the
   notarised receipt.** Phase 1 should pull
   `GET /api/v1/custody/attestation` at each consequential decision
   boundary, sign-verify it against the Ed25519 `public_key` from
   `/api/v1/identity`, and persist the whole envelope (attestation +
   signature + verifier identity + verifier timestamp) into the CCF
   evidence store. That envelope is what the prosecution exhibit looks
   like.

3. **Bridge transport: HTTP, not WebSocket, for v1.** The Seed exposes
   `/sensor/stream` and `/delta/stream` (probably SSE or WS) — useful
   later, but Phase 1 round-trip works fine over HTTP and stays
   compatible with the rate-limit semantics already characterised. Add
   streaming when the use-case actually needs sub-second push.

4. **Implement 429 back-off as a first-class concern.** The rate limiter
   returns `429 retry after 1s` for any over-pressure, including for
   non-existent paths. The bridge must:
   - Treat 429 as transient regardless of path.
   - Use exponential back-off with full jitter, base 1.2 s.
   - Never interpret 404 unless preceded by a successful request to the
     same host within the last 2 s (otherwise the 404 may be a
     post-throttle false negative).

5. **Use client-supplied IDs that encode provenance.** Since the API
   accepts client-chosen u64 IDs, encode `(timestamp_us & 0xFFFF_FFFF) << 32 | mbot_session_id`
   or similar. This lets Phase 1 vectors be identified by source without
   a metadata round-trip and survives `/store/sync` replication.

6. **Metadata writes count as witness events.** Every metadata edit
   advances the witness chain. Choose carefully whether mBot2
   interactions should cause metadata churn. If labels need to evolve
   often, consider a separate side-channel store rather than thrashing
   the chain.

7. **Sensor embedding is 45-d, RVF store is 8-d.** The Seed's own sensor
   pipeline doesn't write into the 8-d RVF store directly. The CCF
   bridge is therefore the *first* writer of meaningful 8-d vectors —
   the existing 90+ stored "vectors" are witness anchors, not data.
   This is a clean greenfield for CCF semantics. Confirm with the
   vendor that 8 dimensions is what we want for mBot2-derived vectors;
   if not, change `dimension` before any production ingest (probably
   requires firmware/config change, not an API call).

8. **Deploy an SSH key during the next maintenance window** so future
   Phase reports can include OS/I2C bus introspection without manual
   intervention.

9. **Schedule a probe of the write/sign endpoints** (`/cognitive/tick`,
   `/boundary/recompute`, `/custody/sign`, `/custody/verify`) in a
   controlled session — these were deferred from Phase 0 because they
   are state-changing and the brief restricted writes to ingest only.

---

*End of report.*
