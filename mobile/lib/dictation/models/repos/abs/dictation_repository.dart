// Kanso — dictation_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/models/data/application_error.dart';
import '../../data/dictation_capability.dart';
import '../../data/dictation_permission.dart';
import '../../data/dictation_signal.dart';

/// The phone's speech recognizer. Speech is recognized on the phone; none of
/// it is sent anywhere by Kanso.
abstract interface class DictationRepository {
  /// What the recognizer for [localeTag] (a BCP 47 tag; the phone's language
  /// when null) can do now.
  Future<Either<ApplicationError, DictationCapability>> availability({
    String? localeTag,
  });

  /// Asks for the microphone and speech recognition if they are not asked
  /// yet; where that leaves them.
  Future<Either<ApplicationError, DictationPermission>> requestPermission();

  /// Starts listening. What it hears arrives on [signals].
  Future<Either<ApplicationError, Unit>> start({String? localeTag});

  /// Stops listening and lets the recognizer finish what it heard: a final
  /// signal may follow, then the end.
  Future<Either<ApplicationError, Unit>> stop();

  /// Stops at once, dropping what was not finished.
  Future<Either<ApplicationError, Unit>> cancel();

  /// Opens the phone's Settings page for Kanso.
  Future<Either<ApplicationError, Unit>> openSettings();

  /// What a session reports; the last signal of one is `ended`.
  Stream<DictationSignal> signals();
}
