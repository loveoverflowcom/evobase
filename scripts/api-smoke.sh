#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:3000}"
USERNAME="${USERNAME:-alice}"
PASSWORD="${PASSWORD:-super-secret-password}"

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for this script."
  exit 1
fi

echo "Registering user..."
curl -s -X POST "$BASE_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}" | jq

echo
echo "Logging in..."
LOGIN_JSON="$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")"

echo "$LOGIN_JSON" | jq

ACCESS_TOKEN="$(echo "$LOGIN_JSON" | jq -r '.tokens.access_token')"
REFRESH_TOKEN="$(echo "$LOGIN_JSON" | jq -r '.tokens.refresh_token')"
NOTIFICATION_TOKEN="$(echo "$LOGIN_JSON" | jq -r '.tokens.notification_token')"
USER_ID="$(echo "$LOGIN_JSON" | jq -r '.user_id')"

echo
echo "Refreshing token..."
curl -s -X POST "$BASE_URL/auth/refresh" \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\":\"$REFRESH_TOKEN\"}" | jq

echo
echo "Creating a note..."
CREATE_NOTE_JSON="$(curl -s -X POST "$BASE_URL/rest/public.notes" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"owner_id\":\"$USER_ID\",\"body\":\"hello from evobase\"}")"
echo "$CREATE_NOTE_JSON" | jq

NOTE_ID="$(echo "$CREATE_NOTE_JSON" | jq -r '.[0].id')"

echo
echo "Listing notes..."
curl -s "$BASE_URL/rest/public.notes?select=id,body,created_at&order=created_at.desc&limit=10" \
  -H "Authorization: Bearer $ACCESS_TOKEN" | jq

echo
echo "Updating note..."
curl -s -X PATCH "$BASE_URL/rest/public.notes?id=eq.$NOTE_ID" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"body\":\"updated body\"}" | jq

echo
echo "Sending a message..."
curl -s -X POST "$BASE_URL/messages/send" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"to_user_id\":\"$USER_ID\",\"event\":\"chat.message\",\"payload\":{\"body\":\"hello\"}}" | jq

echo
echo "Notification token:"
echo "$NOTIFICATION_TOKEN"
echo
echo "Open the SSE stream with:"
echo "curl -N \"$BASE_URL/events?token=$NOTIFICATION_TOKEN\""
