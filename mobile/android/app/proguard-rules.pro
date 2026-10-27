# The native library finds these by name through JNI, which shrinking cannot
# see.
-keep class app.kanso.kanso.KansoNative { *; }
-keep class app.kanso.kanso.KansoSecrets { *; }
# The plugins' WorkManager job is created by name; the auth callback is
# started by the system from the manifest.
-keep class app.kanso.kanso_background.SyncWorker { *; }
-keep class app.kanso.kanso_auth_session.AuthCallbackActivity { *; }
# The certificate verifier's Kotlin component, called from Rust.
-keep, includedescriptorclasses class org.rustls.platformverifier.** { *; }
# The share activity and the widget are started by the system by name; the
# manifest keeps them, and captureInbox is covered by KansoNative's rule above.
