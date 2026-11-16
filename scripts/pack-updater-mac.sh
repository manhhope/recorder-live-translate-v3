#!/bin/bash
# Dong goi goi cap nhat macOS (US-33) SAU khi da ky .app bang 'RLT Local Signing' (scripts/sign-local.sh):
# `tauri build` tu tao .app.tar.gz TRUOC buoc ky lai, nen goi do chua .app cu. O day nen lai tu .app da ky, roi ky minisign.
# Khoa rieng doc tu bien moi truong TAURI_SIGNING_PRIVATE_KEY (hoac _PATH) va TAURI_SIGNING_PRIVATE_KEY_PASSWORD, khong bao gio tu tham so/tep trong repo.
# Dung: scripts/pack-updater-mac.sh "<duong-dan>.app" <thu-muc-ra>   ->  <ra>/<ten>.app.tar.gz va <ten>.app.tar.gz.sig
set -euo pipefail
APP="${1:?Dung: pack-updater-mac.sh <ten>.app <thu-muc-ra>}"
OUT="${2:?Thieu thu muc ra}"
[ -d "$APP" ] || { echo "Khong thay $APP" >&2; exit 1; }
[ -n "${TAURI_SIGNING_PRIVATE_KEY:-}${TAURI_SIGNING_PRIVATE_KEY_PATH:-}" ] || { echo "Thieu TAURI_SIGNING_PRIVATE_KEY (hoac _PATH)" >&2; exit 1; }
codesign --verify --deep --strict "$APP"   # goi chi duoc ky minisign khi .app da qua codesign
mkdir -p "$OUT"
NAME="$(basename "$APP")"
TAR="$OUT/$NAME.tar.gz"
rm -f "$TAR" "$TAR.sig"
# COPYFILE_DISABLE: khong them tep ._* (AppleDouble) lam vo chu ky codesign sau khi giai nen.
COPYFILE_DISABLE=1 tar -czf "$TAR" -C "$(dirname "$APP")" "$NAME"
cargo tauri signer sign "$TAR"
ls -l "$TAR" "$TAR.sig"
