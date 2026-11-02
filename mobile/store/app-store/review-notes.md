# Notes for App Review

(App Store Connect › App Review Information › Notes.)

Kanso is a Markdown notes app. Every feature works with no account: open the app
and write. Sync is optional; to try it, create an account from Settings › Sync
with a username and password (no email is asked for).

Encryption: the app seals notes end to end with standard algorithms (Argon2id,
HKDF-SHA256, XChaCha20-Poly1305, X25519, Ed25519) before they are synced. Export
compliance is answered in Info.plist (`ITSAppUsesNonExemptEncryption`).

Sign in with Apple is offered beside Google and GitHub (guideline 4.8).

Background audio (`UIBackgroundModes` = `audio`): used only for voice notes the
user starts on purpose from the Action Button, Control Center, a Home or Lock
Screen widget, or Siri. Pressing it records the microphone with Kanso in the
background, with a Live Activity on the Lock Screen and in the Dynamic Island
showing a running timer and a Stop button the whole time. A second press, or
Stop, saves the note; a recording ends by itself after 30 minutes. Nothing is
recorded without that visible Live Activity (if Live Activities are off, the
action refuses and says so), and the microphone is never used in the background
for anything else. Notes are written to Kanso's inbox and turned into text on
the device the next time the app is open. To try it: Settings > Action Button >
Controls (or add the voice note control in Control Center), allow the
microphone once in Kanso, then press it.
