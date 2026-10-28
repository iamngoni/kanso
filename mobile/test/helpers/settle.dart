// Kanso — settle.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Lets the calls a BLoC made to a fake repository finish: the fakes answer
/// at once, so a few turns of the event loop run the whole chain of awaits.
Future<void> settle() async {
  for (var i = 0; i < 12; i++) {
    await Future<void>.delayed(Duration.zero);
  }
}
