#!/usr/bin/env bash
set -euo pipefail
repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_dir"
: "${PLAYWRIGHT_MODULE_PATH:?Set PLAYWRIGHT_MODULE_PATH}"
: "${AXE_MODULE_PATH:?Set AXE_MODULE_PATH}"
: "${EVIDENCE_DIR:?Set EVIDENCE_DIR outside tracked source}"
export RUNTIME_URL="${RUNTIME_URL:-http://127.0.0.1:4173/?runtime=1}"
export RUNTIME_SUPPORT_HOST=http://127.0.0.1:18080
export RUNTIME_LIBRARY_HOST=http://127.0.0.1:18081
mkdir -p "$EVIDENCE_DIR"
fixture_dir="$(mktemp -d /tmp/evobase-runtime-XXXXXX)"
host_pids=()
cleanup() {
  for pid in "${host_pids[@]}"; do kill "$pid" 2>/dev/null || true; done
  for pid in "${host_pids[@]}"; do wait "$pid" 2>/dev/null || true; done
  rm -rf "$fixture_dir"
}
trap cleanup EXIT
python3 - "$fixture_dir" "$RUNTIME_URL" <<'PY'
import hashlib, json, pathlib, secrets, sys, time
from urllib.parse import urlsplit
root = pathlib.Path(sys.argv[1])
origin = urlsplit(sys.argv[2])
assert origin.scheme in ('http', 'https') and origin.netloc
(root / 'origin.txt').write_text(f'{origin.scheme}://{origin.netloc}')
token = secrets.token_hex(32)
(root / 'token.txt').write_text(token)
(root / 'token.txt').chmod(0o600)
access = dict(token_sha256=hashlib.sha256(token.encode()).hexdigest(), actor_id='actor_alice',
              expires_at=int(time.time()) + 3600, revoked=False, active=True,
              membership_revision=1, roles=[], grants=['read', 'write'])
for app in ('support', 'library'):
    app_access = dict(access, roles=['role_editor', 'role_auditor'] if app == 'support' else [])
    (root / f'{app}-access.json').write_text(json.dumps(app_access))
    (root / f'{app}-access.json').chmod(0o600)
source = pathlib.Path('crates/evobase-appspec/tests/fixtures')
support_records = json.loads((source / 'support-records.json').read_text())
support_records[1]['values']['fld_request_owner'] = dict(type='text', value='actor_bob')
(root / 'support-records.json').write_text(json.dumps(support_records))
library = json.loads((source / 'library-v2.json').read_text())
for command in library['commands']:
    for field in command.get('inputs', []):
        if field['field_id'] == 'fld_loan_note':
            field['required'] = False
records = json.loads((source / 'library-records.json').read_text())
extra = json.loads(json.dumps(records[0]))
extra['id'] = 'rec_loan_2'
extra['values']['fld_loan_note'] = dict(type='text', value='Preserve untouched optional note')
records.append(extra)
(root / 'library-v2.json').write_text(json.dumps(library))
(root / 'library-records.json').write_text(json.dumps(records))
PY
export EVOBASE_TENANT_ID=tenant_demo
export EVOBASE_ALLOWED_ORIGINS="$(cat "$fixture_dir/origin.txt")"
host_binary="${EVOBASE_HOST_BINARY:-$repo_dir/target/debug/evobase-host}"
if [[ ! -x "$host_binary" ]]; then echo "Build evobase-host before this check." >&2; exit 1; fi
EVOBASE_APP_ID=app_support EVOBASE_DB_URL="file:$fixture_dir/support.db" \
  "$host_binary" fixture-bootstrap crates/evobase-appspec/tests/fixtures/support-v2.json \
  "$fixture_dir/support-records.json" > "$EVIDENCE_DIR/support-bootstrap.log"
EVOBASE_APP_ID=app_library EVOBASE_DB_URL="file:$fixture_dir/library.db" \
  "$host_binary" fixture-bootstrap "$fixture_dir/library-v2.json" \
  "$fixture_dir/library-records.json" > "$EVIDENCE_DIR/library-bootstrap.log"
EVOBASE_APP_ID=app_support EVOBASE_DB_URL="file:$fixture_dir/support.db" \
  EVOBASE_ACCESS_FILE="$fixture_dir/support-access.json" EVOBASE_LISTEN=127.0.0.1:18080 \
  "$host_binary" serve > "$EVIDENCE_DIR/support-host.log" 2>&1 &
host_pids+=("$!")
EVOBASE_APP_ID=app_library EVOBASE_DB_URL="file:$fixture_dir/library.db" \
  EVOBASE_ACCESS_FILE="$fixture_dir/library-access.json" EVOBASE_LISTEN=127.0.0.1:18081 \
  "$host_binary" serve > "$EVIDENCE_DIR/library-host.log" 2>&1 &
host_pids+=("$!")
for app in support library; do
  if [[ "$app" == support ]]; then port=18080; else port=18081; fi
  ready=false
  for attempt in {1..50}; do
    if curl --silent --output /dev/null "http://127.0.0.1:$port/v1/apps/app_$app/runtime-metadata"; then ready=true; break; fi
    sleep 0.1
  done
  if [[ "$ready" != true ]]; then echo "Fixture HTTP host did not start." >&2; exit 1; fi
done
export RUNTIME_SUPPORT_ACCESS_FILE="$fixture_dir/support-access.json"
python3 - "$fixture_dir/token.txt" <<'PY'
import os, pathlib, subprocess, sys
env = dict(os.environ, RUNTIME_TOKEN=pathlib.Path(sys.argv[1]).read_text())
sys.exit(subprocess.call(['node', 'scripts/check_runtime.mjs'], env=env))
PY
