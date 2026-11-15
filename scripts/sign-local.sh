#!/bin/bash
# Ky lai mot ban .app bang chung chi noi bo 'RLT Local Signing' (hoac RLT_SIGN_IDENTITY / APPLE_SIGNING_IDENTITY).
# Dung: scripts/sign-local.sh "<duong-dan>.app"
set -euo pipefail
APP="${1:?Dung: sign-local.sh <ten>.app}"
[ -d "$APP" ] || { echo "Khong thay $APP" >&2; exit 1; }
HERE="$(cd "$(dirname "$0")" && pwd)"
IDENT="${RLT_SIGN_IDENTITY:-${APPLE_SIGNING_IDENTITY:-RLT Local Signing}}"
BUNDLE_ID="${RLT_BUNDLE_ID:-com.mmola.recorderlivetranslate}"
ENT="${RLT_ENTITLEMENTS:-$HERE/../src-tauri/Entitlements.plist}"

xattr -cr "$APP"
codesign --force --deep --options runtime --timestamp=none \
  --sign "$IDENT" --identifier "$BUNDLE_ID" --entitlements "$ENT" "$APP"
codesign --verify --deep --strict --verbose=2 "$APP"
echo "--- codesign -dv"
codesign -dv --verbose=4 "$APP" 2>&1
echo "--- codesign -d -r-"
codesign -d -r- "$APP" 2>&1
echo "--- entitlements"
codesign -d --entitlements - "$APP" 2>&1
