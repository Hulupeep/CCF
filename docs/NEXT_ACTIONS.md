# Pre-Release Action List

## P0 — Blocking (DONE)

- [x] Fix 14 test compilation errors in mbot-core (cfg feature unification)
- [x] Commit unstaged brain layer (32 modules + 3 contracts + 18 tools)
- [x] Push all commits to origin/main (23 commits rebased and pushed)
- [x] All 649 tests pass, 0 failures

## P1 — Important (before release)

- [ ] **Test ElevenLabs TTS end-to-end** — Code in voice_api.rs, needs ELEVENLABS_API_KEY in .env and phone connected to voice API
- [ ] **Test web dashboard phone UI** — Chat bubbles, mood display, voice commands via phone browser
- [ ] **Wire up `--draw` flag to ArtBot mode** — CLI flag is parsed but ArtBot handler is a stub
- [x] **Redesign web dashboard** — Single-page educational dashboard with one-click connect, live annotations, experiment cards, personality that actually works
- [ ] **Clean up ~65 dead code warnings** — Noise in cargo build output hides real issues

## P2 — Nice to have

- [ ] **Complete or remove Telegram/Discord channel stubs** — brain/channels/ has trait + stubs, not wired to anything
- [ ] **Document brain layer setup in MASTER_GUIDE.md** — `--brain` flag, LLM config, Ollama fallback
- [ ] **Tag a release version** — No version tags exist yet (v0.1.0?)

## Testing Checklist (manual)

| Feature | How to test | Status |
|---------|-------------|--------|
| Serial connect | USB-C cable, `cargo run -- --serial` | Tested |
| BLE connect | `cargo run -- --ble` | Tested |
| Telegram bot | `python3 -u tools/telegram_bridge.py` | Tested |
| Voice commands | Send "forward", "dance", "circle" via Telegram | Tested |
| ElevenLabs TTS | Set ELEVENLABS_API_KEY, send text, hear speech | Not tested |
| Web dashboard | `node web/server.js`, open localhost:3000 | Partially tested |
| Phone voice UI | Open voice API URL on phone browser | Not tested |
| Drawing mode | `cargo run -- --draw` | Not wired up |
| Brain layer | `cargo run -- --brain` | Not tested end-to-end |
