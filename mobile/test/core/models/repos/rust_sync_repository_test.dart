// Kanso — rust_sync_repository_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/sync.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/impl/rust_sync_repository.dart';
import 'package:mocktail/mocktail.dart';

class _MockSync extends Mock implements KansoSync {}

void main() {
  late _MockSync sync;
  late RustSyncRepository repository;

  setUp(() {
    sync = _MockSync();
    repository = RustSyncRepository(sync);
  });

  group('RustSyncRepository, the app in front or not', () {
    test('tells the library which', () async {
      when(() => sync.setForeground(foreground: true)).thenAnswer((_) async {});
      when(
        () => sync.setForeground(foreground: false),
      ).thenAnswer((_) async {});
      expect(
        await repository.setForeground(foreground: true),
        const Right<ApplicationError, Unit>(unit),
      );
      expect(
        await repository.setForeground(foreground: false),
        const Right<ApplicationError, Unit>(unit),
      );
      verifyInOrder(<void Function()>[
        () => sync.setForeground(foreground: true),
        () => sync.setForeground(foreground: false),
      ]);
    });

    test('a library that refuses is an error', () async {
      when(() => sync.setForeground(foreground: true)).thenAnswer(
        (_) => Future<void>.error(const KansoError.internal(message: 'bug')),
      );
      final result = await repository.setForeground(foreground: true);
      expect(result.swap().toOption().toNullable(), isA<InternalError>());
    });
  });
}
