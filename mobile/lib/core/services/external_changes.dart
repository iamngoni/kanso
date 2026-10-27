// Kanso — external_changes.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:injectable/injectable.dart';

/// Tells the lists that the library may have changed behind the app's back.
///
/// The library's change stream only carries what this process writes. The
/// share extension and the shortcuts write from another process, so when the
/// app comes back to the front [announce] says "read again", and
/// `LibraryRepository.changes` carries it to the same listeners as any other
/// change.
@lazySingleton
class ExternalChanges {
  final StreamController<List<String>> _controller =
      StreamController<List<String>>.broadcast();

  /// What was announced, to listeners from now on.
  Stream<List<String>> get stream => _controller.stream;

  /// Says that [ids] changed; none means "anything may have".
  void announce([List<String> ids = const <String>[]]) {
    if (!_controller.isClosed) {
      _controller.add(ids);
    }
  }

  /// Ends the stream.
  Future<void> close() => _controller.close();
}
