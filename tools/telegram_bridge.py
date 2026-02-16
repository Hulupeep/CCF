#!/usr/bin/env python3
"""Telegram <-> mBot Voice API bridge.

Polls Telegram for messages, forwards them to the voice API /api/text endpoint,
sends the response back to Telegram. No pip install needed (stdlib only).

Usage:
    python3 tools/telegram_bridge.py

Requires:
    MBOT_TELEGRAM_TOKEN env var (or reads from .env file)
    Voice API running on localhost:8088
"""

import json
import os
import sys
import time
import urllib.request
import urllib.error

VOICE_API = os.environ.get("MBOT_VOICE_API_URL", "http://localhost:8088")

def load_env():
    """Load .env file if present."""
    env_path = os.path.join(os.path.dirname(__file__), "..", ".env")
    if os.path.exists(env_path):
        with open(env_path) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, _, val = line.partition("=")
                    os.environ.setdefault(key.strip(), val.strip())

def tg_request(token, method, data=None):
    """Call Telegram Bot API."""
    url = f"https://api.telegram.org/bot{token}/{method}"
    if data:
        body = json.dumps(data).encode()
        req = urllib.request.Request(url, body, {"Content-Type": "application/json"})
    else:
        req = urllib.request.Request(url)
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        print(f"Telegram API error ({method}): {e.code} {e.read().decode()}")
        return None
    except Exception as e:
        print(f"Telegram request failed ({method}): {e}")
        return None

def voice_api_text(text):
    """Send text command to voice API, return response."""
    url = f"{VOICE_API}/api/text"
    body = json.dumps({"text": text}).encode()
    req = urllib.request.Request(url, body, {"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            return json.loads(resp.read())
    except Exception as e:
        return {"text": f"Voice API error: {e}", "actions": [], "mood": "?"}

def format_response(data):
    """Format voice API response for Telegram."""
    parts = []
    if data.get("text"):
        parts.append(data["text"])
    if data.get("actions"):
        parts.append(f"Actions: {', '.join(data['actions'])}")
    if data.get("mood"):
        moods = {"CALM": "\U0001f916", "Active": "\U0001f914", "Spike": "\U0001f631", "Protect": "\U0001f628"}
        emoji = moods.get(data["mood"], "\U0001f916")
        parts.append(f"Mood: {emoji} {data['mood']}")
    return "\n".join(parts) if parts else "..."

def main():
    load_env()
    token = os.environ.get("MBOT_TELEGRAM_TOKEN") or os.environ.get("TELEGRAM_BOT_TOKEN")
    if not token:
        print("Error: MBOT_TELEGRAM_TOKEN not set in env or .env file")
        sys.exit(1)

    # Verify bot token
    me = tg_request(token, "getMe")
    if not me or not me.get("ok"):
        print(f"Invalid bot token: {me}")
        sys.exit(1)
    bot_name = me["result"]["username"]
    print(f"Telegram bot @{bot_name} connected!")
    print(f"Voice API: {VOICE_API}")
    print("Send messages to the bot in Telegram. Ctrl+C to stop.\n")

    offset = 0
    while True:
        try:
            updates = tg_request(token, "getUpdates", {
                "offset": offset,
                "timeout": 30,
                "allowed_updates": ["message"]
            })
            if not updates or not updates.get("ok"):
                time.sleep(2)
                continue

            for update in updates["result"]:
                offset = update["update_id"] + 1
                msg = update.get("message", {})
                text = msg.get("text", "")
                chat_id = msg.get("chat", {}).get("id")
                user = msg.get("from", {}).get("first_name", "?")

                if not text or not chat_id:
                    continue

                # Handle /start
                if text == "/start":
                    tg_request(token, "sendMessage", {
                        "chat_id": chat_id,
                        "text": (
                            "Hi! I'm mBot \U0001f916\n\n"
                            "Send me commands:\n"
                            "  forward / back / left / right\n"
                            "  circle / spin / dance\n"
                            "  stop\n"
                            "  say <text>\n"
                            "  hello / how are you\n\n"
                            "I'll move the real robot!"
                        )
                    })
                    continue

                print(f"[{user}] {text}")

                # Forward to voice API
                resp = voice_api_text(text)
                reply = format_response(resp)
                print(f"  -> {reply}")

                tg_request(token, "sendMessage", {
                    "chat_id": chat_id,
                    "text": reply
                })

        except KeyboardInterrupt:
            print("\nStopping Telegram bridge...")
            break
        except Exception as e:
            print(f"Error: {e}")
            time.sleep(2)

if __name__ == "__main__":
    main()
