// Kanso — kanso_push.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Kanso's push wake-up. A push carries nothing: it tells the app to sync.
///
/// Dart asks for a token through `KansoPushHostApi.register` and hears of
/// each token, now and when the OS issues a new one, by implementing
/// `KansoPushFlutterApi` and passing it to `KansoPushFlutterApi.setUp`.
/// The silent pushes themselves are answered natively (`KansoPush.onWake` on
/// iOS and Android), without a Dart isolate.
library;

export 'src/messages.g.dart' show KansoPushFlutterApi, KansoPushHostApi;
