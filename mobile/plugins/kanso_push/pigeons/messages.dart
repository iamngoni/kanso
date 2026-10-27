// Kanso — messages.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// ignore_for_file: one_member_abstracts

// The typed channel between Dart and the native push code. Regenerate with
// `dart run pigeon --input pigeons/messages.dart` from the plugin's folder.

import 'package:pigeon/pigeon.dart';

@ConfigurePigeon(
  PigeonOptions(
    dartOut: 'lib/src/messages.g.dart',
    swiftOut: 'ios/kanso_push/Sources/kanso_push/Messages.g.swift',
    kotlinOut: 'android/src/main/kotlin/app/kanso/kanso_push/Messages.g.kt',
    kotlinOptions: KotlinOptions(package: 'app.kanso.kanso_push'),
    copyrightHeader: 'pigeons/copyright.txt',
  ),
)
/// What Dart asks of the native side.
@HostApi()
abstract class KansoPushHostApi {
  /// Asks the OS for this phone's push token. Returns at once; the token
  /// arrives through [KansoPushFlutterApi.onToken], now and each time the OS
  /// issues a new one.
  void register();

  /// Why push can't work on this device, in words for a log; null when it
  /// can. On Android, no Firebase configuration in the build.
  String? unavailableReason();
}

/// What the native side tells Dart.
@FlutterApi()
abstract class KansoPushFlutterApi {
  /// The OS handed out [token]. [platform] is `apns` or `fcm`; [environment]
  /// is `production` or `development` (a debug iOS build is sandboxed).
  void onToken(String platform, String token, String environment);
}
