# Ky so noi bo - bao cao va huong dan

## Hien trang (may chu)
- Chung chi tu ky `RLT Local Signing` (RSA 2048, 10 nam, EKU codeSigning) trong login keychain, dat trust codeSign. Tao bang `scripts/make-local-signing-cert.sh` (chay lai: bao "da co", khong lam gi them). Khoa/chung chi khong nam trong repo; file tam .p12/.key bi xoa khi script ket thuc.
- Ky mot ban `.app`: `scripts/sign-local.sh "<ban>.app"` (hardened runtime, `--identifier com.mmola.recorderlivetranslate`, entitlements `src-tauri/Entitlements.plist`, roi `codesign --verify --deep --strict` va in `-dv`, `-d -r-`).
- Entitlements toi thieu: chi `com.apple.security.device.audio-input` (hardened runtime chan micro neu thieu). Khong can: screen capture (TCC, khong phai entitlement vi app khong sandbox), network, JIT (WKWebView chay tien trinh WebContent rieng), library validation (khong nap dylib ngoai).
- Designated requirement gan voi chung chi, khong chi cdhash:
  `identifier "com.mmola.recorderlivetranslate" and certificate leaf = H"347574eddafeb68f0832b73376828200ed6d6494"`
  Moi build ky bang cung chung chi co cung requirement, nen Keychain/TCC nhan ra la cung mot app qua cac ban.
- `tauri.conf.json` co `bundle.macOS.entitlements = Entitlements.plist`. Tauri tu doc bien moi truong `APPLE_SIGNING_IDENTITY` khi `cargo tauri build`; khong dat bien thi build van ad-hoc roi chay `sign-local.sh` de ky lai.

## Sau lan dung dau tien (chu doc)
1. **Khoa trong Keychain** (Soniox/DeepL/Gemini): muc do ban ad-hoc hoac CLI tao co ACL khong thuoc app moi. Lan dau app doc co the hien hoi mat khau: bam **Always Allow** mot lan. Neu van loi hoac khong co, luu lai khoa trong Cai dat mot lan. Tu cac ban sau (cung chung chi) khong hoi lai.
2. **Screen Recording / Microphone**: danh tinh app doi tu ad-hoc sang chung chi nen cap lai quyen mot lan trong System Settings > Privacy & Security. Neu bi ket (da bat nhung van bao thieu quyen):
   `tccutil reset ScreenCapture com.mmola.recorderlivetranslate` (tuong tu `Microphone`), mo lai app va cap lai.
3. **Chi hieu luc tren may chu.** Chung chi tu ky khong duoc Apple tin tren may khac: nguoi dung khac van thay canh bao Gatekeeper va can ban ky Developer ID + notarize.
4. Doi chung chi (ky lai sau 10 nam, hoac tao lai) la doi danh tinh: lap lai muc 1 va 2.

## Khi co Apple Developer (chi chua cho, chua lam)
1. Dang ky Apple Developer Program, tao chung chi **Developer ID Application**, nhap vao keychain (xem `security find-identity -v -p codesigning`).
2. Thay mot bien: `export APPLE_SIGNING_IDENTITY="Developer ID Application: <Ten> (<TEAMID>)"` roi `cargo tauri build --bundles app`. Tauri ky voi hardened runtime va `Entitlements.plist` san co. Hoac `RLT_SIGN_IDENTITY=... scripts/sign-local.sh <app>`.
3. Hardened runtime da bat (`--options runtime`), giu entitlements toi thieu nhu tren; them `--timestamp` (bo `--timestamp=none` trong `sign-local.sh`) vi notarize bat buoc dau thoi gian.
4. Notarize: dong goi `ditto -c -k --keepParent App.app App.zip`, roi `xcrun notarytool submit App.zip --keychain-profile <profile> --wait` (tao profile bang `xcrun notarytool store-credentials`).
5. Staple: `xcrun stapler staple App.app`, kiem tra `spctl -a -vv App.app` va `xcrun stapler validate App.app`.
6. Designated requirement doi sang Team ID/Developer ID: Keychain ACL va TCC cua ban cu (RLT Local Signing) mat hieu luc mot lan, lap lai muc "Sau lan dung dau tien".
