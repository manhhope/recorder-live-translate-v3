# M1 - rlt-secrets: bao cao

Crate `crates/rlt-secrets`: trait `SecretStore` (set/get/delete), `KeychainSecretStore` (crate `keyring` 3, feature apple-native, chi macOS), `InMemorySecretStore` (test). Ba loai khoa: Soniox, Gemini, OpenAI. Service Keychain: `com.recorder.livetranslate.v3`. `SecretValue` che gia tri khi Debug va khong co Display. Test Keychain that dung service rieng `...v3.test.<pid>`, khoa gia, xoa sach sau khi chay.

## cargo test -p rlt-secrets
```
    Updating crates.io index
     Locking 29 packages to highest compatible versions
      Adding data-encoding v2.11.1
      Adding getrandom v0.2.17
      Adding ppv-lite86 v0.2.21
      Adding rand v0.9.5
      Adding rand_chacha v0.9.0
      Adding rand_core v0.9.5
      Adding ring v0.17.14
      Adding rustls v0.23.46
      Adding rustls-pki-types v1.16.0
      Adding rustls-webpki v0.103.17
      Adding sha1 v0.10.7
      Adding subtle v2.6.1
      Adding tokio-macros v2.7.2
      Adding tokio-rustls v0.26.6
      Adding tokio-tungstenite v0.29.0 (available: v0.30.0)
      Adding tungstenite v0.29.0
      Adding untrusted v0.9.0
      Adding webpki-roots v0.26.11
      Adding webpki-roots v1.0.12
      Adding windows-sys v0.52.0
      Adding windows-targets v0.52.6
      Adding windows_aarch64_gnullvm v0.52.6
      Adding windows_aarch64_msvc v0.52.6
      Adding windows_i686_gnu v0.52.6
      Adding windows_i686_gnullvm v0.52.6
      Adding windows_i686_msvc v0.52.6
      Adding windows_x86_64_gnu v0.52.6
      Adding windows_x86_64_gnullvm v0.52.6
      Adding windows_x86_64_msvc v0.52.6
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.12s
     Running unittests src/lib.rs (target/debug/build/rlt-secrets/0d24840cc885f5d3/out/rlt_secrets-0d24840cc885f5d3)
running 8 tests
test tests::debug_redacts_value ... ok
test tests::empty_value_rejected ... ok
test tests::debug_of_containers_redacts_value ... ok
test tests::kinds_are_independent ... ok
test tests::error_display_has_no_value ... ok
test tests::in_memory_roundtrip_three_kinds ... ok
test tests::service_is_v3_specific ... ok
test tests::real_keychain_roundtrip_fake_key_cleaned_up ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
   Doc-tests rlt_secrets
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## cargo clippy -p rlt-secrets --all-targets -- -D warnings
```
    Checking rlt-secrets v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-secrets)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
```

## Grep chung minh khong lo khoa
Khoa gia dung trong test: `FAKE-TEST-KEY-0000`.
```
$ grep -rn RecorderLiveTranslate crates/rlt-secrets/
exit=1
$ grep -rn "FAKE" crates/rlt-secrets/src | grep -v tests (phan ngoai mod tests: dong < dong cua mod tests)
exit=1 (1 = khong thay)
$ cargo test -p rlt-secrets -- --nocapture 2>&1 | grep -c FAKE-TEST-KEY
0
$ grep -c FAKE-TEST-KEY target/debug/librlt_secrets.rlib
$ strings target/debug/librlt_secrets.rlib | grep -c FAKE-TEST-KEY
0
$ dung Keychain that sau test: security find-generic-password -s com.recorder.livetranslate.v3.test
security: SecKeychainSearchCopyNext: The specified item could not be found in the keychain.
```
