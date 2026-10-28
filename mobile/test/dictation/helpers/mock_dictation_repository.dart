// Kanso — mock_dictation_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/dictation/models/data/dictation_capability.dart';
import 'package:kanso/dictation/models/data/dictation_permission.dart';
import 'package:kanso/dictation/models/data/dictation_signal.dart';
import 'package:kanso/dictation/models/repos/abs/dictation_repository.dart';
import 'package:mocktail/mocktail.dart';

class MockDictationRepository extends Mock implements DictationRepository {}

/// A mock repository that can hear: it answers `capability`, takes the
/// permission `asked` for, and brings what a test adds to `signals` to
/// whoever listens.
class SpokenRepository {
  SpokenRepository({
    DictationCapability capability = const DictationCapability(
      available: true,
      onDevice: true,
      permission: DictationPermission.granted,
    ),
    DictationPermission asked = DictationPermission.granted,
  }) {
    when(repository.signals).thenAnswer((_) => signals.stream);
    when(
      () => repository.availability(localeTag: any(named: 'localeTag')),
    ).thenAnswer(
      (_) async => Right<ApplicationError, DictationCapability>(capability),
    );
    when(repository.requestPermission).thenAnswer(
      (_) async => Right<ApplicationError, DictationPermission>(asked),
    );
    when(
      () => repository.start(localeTag: any(named: 'localeTag')),
    ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
    when(
      repository.stop,
    ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
    when(
      repository.cancel,
    ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
    when(
      repository.openSettings,
    ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
  }

  final MockDictationRepository repository = MockDictationRepository();
  final StreamController<DictationSignal> signals =
      StreamController<DictationSignal>.broadcast();

  /// Closes the stream without waiting: a BLoC that still listens would
  /// keep the wait from ending.
  void close() => unawaited(signals.close());
}
