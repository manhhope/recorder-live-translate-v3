#!/bin/bash
# Tao chung chi ky noi bo "RLT Local Signing" (tu ky, Code Signing) trong login keychain.
# Idempotent: da co thi bao va thoat 0. Khong in mat khau/khoa, xoa file tam.
# Chi co tac dung tren may nay. Nguoi dung khac van can Developer ID.
set -euo pipefail

NAME="${RLT_SIGN_NAME:-RLT Local Signing}"
KC="${RLT_KEYCHAIN:-$HOME/Library/Keychains/login.keychain-db}"
# Tuy chon: mat khau login keychain qua bien moi truong de chay set-key-partition-list khong tuong tac.
# (khong bao gio duoc in ra). Neu khong dat thi script chi nhac lenh can chay.
KC_PASS="${RLT_KEYCHAIN_PASSWORD:-}"

if security find-identity -v -p codesigning | grep -q "\"$NAME\""; then
  echo "Da co chung chi hop le '$NAME' (security find-identity -v -p codesigning). Khong lam gi them."
  exit 0
fi

TMP="$(mktemp -d)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT
umask 077

if ! security find-certificate -c "$NAME" "$KC" >/dev/null 2>&1; then
  P12PASS="$(openssl rand -hex 16)"
  export P12PASS
  cat > "$TMP/openssl.cnf" <<CNF
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = $NAME
[ext]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
subjectKeyIdentifier = hash
CNF
  openssl req -x509 -newkey rsa:2048 -nodes -days 3650 -sha256 \
    -config "$TMP/openssl.cnf" -keyout "$TMP/key.pem" -out "$TMP/cert.pem" 2>/dev/null
  # PBE-SHA1-3DES + sha1 MAC: dinh dang macOS 'security import' doc duoc.
  openssl pkcs12 -export -inkey "$TMP/key.pem" -in "$TMP/cert.pem" -name "$NAME" \
    -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1 \
    -passout env:P12PASS -out "$TMP/id.p12"
  security import "$TMP/id.p12" -k "$KC" -P "$P12PASS" -T /usr/bin/codesign -T /usr/bin/security >/dev/null
  unset P12PASS
  cp "$TMP/cert.pem" "$TMP/trust.pem"
  echo "Da nhap chung chi '$NAME' vao $KC."
else
  echo "Chung chi '$NAME' da nam trong keychain nhung chua duoc tin cay; thu dat trust."
  security find-certificate -c "$NAME" -p "$KC" > "$TMP/trust.pem"
fi

# Cho codesign dung khoa ma khong hoi lai (can mat khau keychain).
if [ -n "$KC_PASS" ]; then
  security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KC_PASS" "$KC" >/dev/null 2>&1 \
    && echo "Da dat partition list cho khoa." || echo "CANH BAO: set-key-partition-list that bai."
else
  echo "Bo qua set-key-partition-list (khong co RLT_KEYCHAIN_PASSWORD)."
fi

# Dat trust cho Code Signing (cho nguoi dung: co the hien hop thoai mat khau).
if ! security add-trusted-cert -r trustRoot -p codeSign -k "$KC" "$TMP/trust.pem" 2>"$TMP/trust.err"; then
  echo "DUNG: khong dat duoc trust (can hop thoai mat khau, khong co phien do hoa)." >&2
  echo "Chu hay chay trong Terminal (se hoi mat khau dang nhap):" >&2
  echo "  security find-certificate -c '$NAME' -p '$KC' > /tmp/rlt-local.pem && security add-trusted-cert -r trustRoot -p codeSign -k '$KC' /tmp/rlt-local.pem && rm /tmp/rlt-local.pem" >&2
  echo "  security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k '<mat khau login>' '$KC'   # tuy chon, tranh hoi khi ky" >&2
  echo "Sau do chay lai script nay de kiem tra." >&2
  exit 2
fi

if security find-identity -v -p codesigning | grep -q "\"$NAME\""; then
  echo "Xong: '$NAME' hop le cho codesigning."
else
  echo "Da dat trust nhung find-identity chua thay '$NAME'. Kiem tra lai." >&2
  exit 3
fi
