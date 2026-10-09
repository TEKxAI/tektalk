#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
for file in clients/web/index.html clients/web/styles.css clients/web/app.js clients/web/README.md; do
  test -s "$root/$file" || { echo "missing Web asset: $file" >&2; exit 1; }
done
command -v node >/dev/null 2>&1 || { echo "Node.js is required to validate JavaScript syntax." >&2; exit 1; }
node --check "$root/clients/web/app.js"
grep -q '<meta name="viewport"' "$root/clients/web/index.html"
grep -q 'data-tab="messages"' "$root/clients/web/index.html"
grep -q 'data-tab="ai"' "$root/clients/web/index.html"
grep -q 'data-tab="me"' "$root/clients/web/index.html"
grep -q '@media(max-width:820px)' "$root/clients/web/styles.css"
echo "Web client assets and JavaScript syntax: ok"
