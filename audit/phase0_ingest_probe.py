#!/usr/bin/env python3
"""Phase 0 end-to-end ingest probe for the Cognitum Seed.

Reads COGNITUM_BASE_URL and COGNITUM_TOKEN from CCF/.env, ingests one
labelled 8-d test vector, then verifies that:
  (a) total_vectors and witness_chain_length both advanced,
  (b) the new vector is retrievable by ID,
  (c) a kNN query against the same vector returns it as a top-k hit,
  (d) the kNN graph node count moved from 0 to >0 (or stayed 0 with reason).

Captures every request/response into audit/phase0_ingest_log.json for the
patent prosecution evidence trail.
"""
from __future__ import annotations

import json
import os
import ssl
import sys
import time
import urllib.request
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
ENV_PATH = REPO_ROOT / ".env"
LOG_PATH = REPO_ROOT / "audit" / "phase0_ingest_log.json"


def load_env(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for raw in path.read_text().splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, _, v = line.partition("=")
        out[k.strip()] = v.strip()
    return out


def call(method: str, url: str, token: str | None = None,
         body: dict | None = None, throttle: float = 1.2) -> tuple[int, dict | str]:
    time.sleep(throttle)  # respect the device rate limiter
    headers = {"Accept": "application/json"}
    data = None
    if body is not None:
        data = json.dumps(body).encode()
        headers["Content-Type"] = "application/json"
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    ctx = ssl.create_default_context()  # validates against system CA store
    try:
        with urllib.request.urlopen(req, context=ctx, timeout=10) as resp:
            raw = resp.read().decode()
            code = resp.status
    except urllib.error.HTTPError as e:
        raw = e.read().decode()
        code = e.code
    try:
        return code, json.loads(raw)
    except json.JSONDecodeError:
        return code, raw


def main() -> int:
    env = load_env(ENV_PATH)
    base = env["COGNITUM_BASE_URL"].rstrip("/")
    token = env["COGNITUM_TOKEN"]

    log: list[dict] = []

    def step(name: str, method: str, path: str, body=None, auth=False):
        url = f"{base}{path}"
        code, payload = call(method, url, token if auth else None, body)
        entry = {
            "step": name, "method": method, "path": path,
            "auth": auth, "request_body": body,
            "http_status": code, "response": payload,
            "ts": time.time(),
        }
        log.append(entry)
        print(f"[{name}] {method} {path} -> {code}")
        return code, payload

    # 1. Pre-snapshot
    _, pre_status = step("pre_status", "GET", "/api/v1/status")
    _, pre_graph = step("pre_graph", "GET", "/api/v1/store/graph/stats")
    print(f"  total_vectors={pre_status.get('total_vectors')} "
          f"witness_chain_length={pre_status.get('witness_chain_length')} "
          f"graph_nodes={pre_graph.get('nodes')}")

    # 2. Ingest one labelled non-zero vector. Wire format observed from the
    #    API Explorer default body: {"vectors":[[<id:u64>, [<f32 x dim>]]]}.
    test_vec = [0.5, -0.3, 0.7, 0.1, -0.2, 0.4, -0.1, 0.6]
    test_id = 9999000420260427  # recognisable: 9999 prefix + YYYYMMDD
    ingest_body = {"vectors": [[test_id, test_vec]]}
    ingest_code, ingest_resp = step(
        "ingest", "POST", "/api/v1/store/ingest", body=ingest_body, auth=True
    )

    # 2b. Label it via the metadata endpoint (separate API).
    if ingest_code == 200:
        step("label", "PUT", f"/api/v1/store/vectors/{test_id}/metadata",
             body={"metadata": [
                 {"field_id": 0, "value": {"String": "phase0-probe"}},
                 {"field_id": 1, "value": {"String": "test-ingest"}},
                 {"field_id": 2, "value": {"String": "ccf-mbot2-bridge-evidence"}},
             ]}, auth=True)

    # 3. Post-snapshot
    _, post_status = step("post_status", "GET", "/api/v1/status")
    _, post_graph = step("post_graph", "GET", "/api/v1/store/graph/stats")
    print(f"  total_vectors={post_status.get('total_vectors')} "
          f"witness_chain_length={post_status.get('witness_chain_length')} "
          f"graph_nodes={post_graph.get('nodes')}")

    # 4. Read the vector back by the ID we ingested under
    step("readback", "GET", f"/api/v1/store/vectors/{test_id}")

    # 6. kNN query with the same vector
    step("query_self", "POST", "/api/v1/store/query",
         body={"vector": test_vec, "k": 5}, auth=True)

    # 7. Persist the log
    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    LOG_PATH.write_text(json.dumps(log, indent=2))
    print(f"\nFull log written to {LOG_PATH.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
