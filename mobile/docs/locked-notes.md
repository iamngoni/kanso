# Locked notes on the phones (#48 step 9)

The contract between the Rust facade (`crates/kanso-mobile`, `mobile/rust`),
the native plugin (`mobile/plugins/kanso_lock`) and the Flutter screens. The
desktop is the reference: `crates/kanso-app/src/locked.rs` and
`notebook_lock.rs`. Every behaviour there has a counterpart here.

## What the person can do

| Desktop | Phone |
|---|---|
| Lock note… (sets the lock password the first time, typed twice; asks for it when the notes are closed; only confirms when they are open) | Note › More › Lock note… |
| Unlock note (Touch ID when on, else the password) | The locked note's card: Unlock with Face ID / Touch ID / fingerprint, and Use password |
| Remove lock… (confirm; refused in a locked notebook) | Note › More › Remove lock… |
| Lock notes now | Note › More › Lock notes now, and Settings › Locked notes › Lock notes now |
| Change lock password… (current, new, again; notes the old password doesn't open keep theirs and are counted) | Settings › Locked notes › Change lock password… |
| Lock notebook… (same three ways as a note) | A notebook's menu in the library › Lock notebook… |
| Opening a locked notebook asks once | Showing a locked notebook's list asks once (biometrics, else the password) |
| Remove notebook lock… (the notebook must be open) | A notebook's menu › Remove lock… |
| Turn on / off Touch ID; offered once after the first password unlock | Settings › Locked notes › Unlock with Face ID (Touch ID, fingerprint); offered once after the first password unlock |
| Auto-lock 10 minutes after unlocking or the last save of a locked note | The same (`kanso_engine::lock::AUTO_LOCK`), and at once when the app goes to the background |
| — | The app switcher shows a blurred cover while a locked note is open (iOS); `FLAG_SECURE` while a locked note is open (Android: no screenshots, blank recents thumbnail) |

There is no "reset" of the lock password on the desktop: Kanso can't recover
it. "Set a lock password" (the first lock) and "Change lock password" are the
two flows, here as there.

## Keys never reach Dart

Dart sees booleans, counts, tickets and outcomes. The lock key, the lock
password after it is typed, the biometric secret and the wrapped keys stay in
Rust and the native plugin.

## Biometric unlock

A biometric secret `B` (32 random bytes) lives in the platform, usable only
after a biometric check and invalidated when the enrolled biometrics change:

- **iOS:** a Keychain generic password (service `app.kanso.Kanso.biometric`,
  account the library ID) with `SecAccessControl` `.biometryCurrentSet` and
  `kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly`. Read through an
  `LAContext` that already passed `deviceOwnerAuthenticationWithBiometrics`,
  so there is one prompt. Gone after a biometrics change, or when the
  passcode is removed.
- **Android:** an AES-256-GCM Android Keystore key (alias
  `kanso.biometric.<library ID>`, StrongBox when present) with
  `setUserAuthenticationRequired(true)`, `BIOMETRIC_STRONG`, and
  `setInvalidatedByBiometricEnrollment(true)`. `B` is stored encrypted by it
  (ciphertext and IV in the plugin's private `SharedPreferences`), and is
  decrypted in `BiometricPrompt` with a `CryptoObject`. A biometrics change
  raises `KeyPermanentlyInvalidatedException`.

Rust keeps, in the library's normal secret store (Keychain
`AfterFirstUnlockThisDeviceOnly` / Keystore), the item
`lock-biometric-v1:<library ID>`:

- an X25519 key pair made at enrolment: the public key in clear, the private
  key sealed with `B` (XChaCha20-Poly1305, key = HKDF-SHA256(B), the library
  ID as associated data);
- the lock keys (`LockKey::to_secret`, the desktop's `LOCK_KEYS_V1` payload)
  sealed to the public key (ephemeral X25519, HKDF-SHA256, XChaCha20-Poly1305).

So the keys kept for biometrics are re-sealed with no prompt whenever the
session's keys change (a password opened another key, the password was
changed), as the desktop's `remember_keys` does, and opening them needs `B`.
If that re-seal fails, Rust forgets the item (biometrics read as off, the
password is the way in) rather than keep keys that are now the old ones.

### A lock wins over a late unlock

Rust counts how many times the notes were locked (`lock_gen`, changed under
the keys mutex by every manual, background and idle lock). A password unlock
or password change notes the count when it starts, before Argon2; a biometric
ticket notes it when it is issued. Keys are added or replaced only if the
count is unchanged. Otherwise they are dropped and the notes stay closed:
`unlock*` return `KansoError::Locked`, a biometric answer returns `-5`, and a
password change still re-seals the notes under the new password but holds no
key and turns biometrics off (their kept keys are the old ones).

### Which key seals a note

A new lock is sealed with a key chosen, never "the session's first key":

- `lock_note(id, password)` and `lock_notebook(id, Some(password))`: the key
  of the password just typed. It must open the library's existing lock
  (`KansoError::Auth` otherwise); with no lock at all, it is a new lock
  password, whatever stale keys the session holds. Locks made with other
  passwords on other devices are each tried, the most used first.
- `lock_note_with_session(id)` and `lock_notebook(id, None)`: of the held
  keys that open some existing lock, the one that opens the most notes and
  notebooks (ties: the one held first). `KansoError::Locked` when none does
  (nothing held, or only keys of locks that no longer exist), and the person
  is asked for the password.

The desktop (`locked.rs`, `notebook_lock.rs`) uses the session's first key
(`LockKeys::current`) in these places and shares the issue of picking a stale
key; this is the phones' fix.

### Tickets

A biometric check is a ticket, as passkeys are (step 8):

1. Dart asks Rust: `begin_biometric_enrollment()` (the notes must be open)
   or `begin_biometric_unlock()` (`None` when biometrics are off). Rust
   returns a ticket (`u64`), good for one answer within 5 minutes.
2. Dart asks the plugin: `enroll(ticket, reason)` or `unlock(ticket, reason)`.
3. The plugin runs the prompt and hands its answer straight to Rust:
   - C: `int32_t kanso_biometric_answer(uint64_t ticket, int32_t status, const uint8_t *secret, size_t len)`
   - JNI: `KansoNative.biometricAnswer(ticket: Long, status: Int, secret: ByteArray?): Int`,
     installed into the plugin by `KansoApplication` (the plugin can't see the
     app module), as step 8's `RustAnswer` is.
   - `status`: `0` ok (`secret` is `B`, 32 bytes), `1` cancelled, `2`
     invalidated (biometrics changed or removed), `3` unavailable, `4` failed.
   - Returns `1` applied, `0` no such ticket, `-1` the secret didn't open the
     kept keys (Rust forgets them: treated as invalidated), `-5` a fault
     (the secret store refused, or the notes were locked again between the
     ticket and the answer: the keys are dropped and the notes stay closed;
     the key the plugin holds is still good).
   - The plugin zeroes its copy of `B` after the call.
4. The plugin tells Dart only the outcome: `done`, `cancelled`,
   `invalidated`, `unavailable` or `failed`. On `invalidated` Rust has
   already forgotten the kept keys; the plugin deleted its own key or item.
   The plugin deletes its key or item only on `invalidated`, and, during
   enrolment, when Rust did not keep the secret (status ok, outcome not
   `done`); both platforms alike. A failed or faulted unlock leaves it.
   A key the platform can no longer recover (Android
   `UnrecoverableKeyException`) counts as gone (`invalidated`).

Turning biometrics off: Rust `turn_off_biometrics()` forgets the item, and the
plugin's `forget()` deletes the platform key or item.

## Auto-lock, background, cover

- Rust's timer closes the notes `AUTO_LOCK` after the last key or save
  (exists). `LockReason` gains `Background`.
- Dart locks with `LockReason::Background` when the app lifecycle reaches
  `hidden` (both platforms), before the app is suspended.
- The plugin's `setSecure(bool)`: Dart turns it on while a locked note's
  text is on screen (open), off otherwise.
  - iOS: the scene delegate puts a full-window blurred cover
    (`UIVisualEffectView`, `.systemMaterial`, the app's mark centred) on the
    window in `sceneWillResignActive` while secure is on, and takes it off in
    `sceneDidBecomeActive`. The app switcher's snapshot shows the cover.
    `setSecure(false)` takes a showing cover down only while the app is
    active; while it is not (Dart locks the notes as the app leaves), it
    only clears the flag and the did-become-active handler removes the
    cover, so it can't come down before the snapshot.
  - Android: `FLAG_SECURE` on the activity's window while on. It is cleared
    only while the activity is resumed; a `setSecure(false)` that arrives
    earlier waits for `onResume`, so the recents thumbnail stays blank.

## Provisional decisions (recorded on #48)

1. Biometrics wrap the lock keys with an X25519 pair whose private half is
   sealed by the platform's biometric secret, so keys are re-kept without a
   prompt (as the desktop's Touch ID does) and opening them needs a check.
2. "Lock after" in Settings shows the fixed 10 minutes, as the desktop has no
   such setting.
3. Lock on background happens at `hidden`, not `inactive`: pulling down
   Notification Centre or a system sheet does not close the notes, leaving
   the app does.
4. The secure flag (cover, `FLAG_SECURE`) is on only while an open locked
   note is shown, as the design says. A closed locked note shows nothing
   secret.
