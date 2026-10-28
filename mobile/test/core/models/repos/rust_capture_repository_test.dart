// Kanso — rust_capture_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/library_folder.dart';
import 'package:kanso/core/models/repos/impl/rust_capture_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_library_repository.dart';
import 'package:kanso/core/services/external_changes.dart';
import 'package:mocktail/mocktail.dart';

class _MockLibrary extends Mock implements KansoLibrary {}

void main() {
  late _MockLibrary library;
  late ExternalChanges external;
  late RustCaptureRepository repository;

  setUpAll(
    () => registerFallbackValue(
      const LocalTime(
        year: 2026,
        month: 1,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
      ),
    ),
  );

  setUp(() {
    library = _MockLibrary();
    external = ExternalChanges();
    addTearDown(external.close);
    repository = RustCaptureRepository(
      library,
      const LibraryFolder('/library/kanso'),
      external,
    );
  });

  group('RustCaptureRepository', () {
    test("saves the drops in the library folder's inbox", () async {
      const saved = DropsSaved(
        notes: <String>['note:cap-a'],
        problems: <String>['x'],
        kept: 0,
      );
      when(
        () => library.saveDrops(inbox: any(named: 'inbox')),
      ).thenAnswer((_) async => saved);
      expect(
        await repository.saveDrops(),
        const Right<Object, DropsSaved>(saved),
      );
      verify(() => library.saveDrops(inbox: '/library/kanso/inbox')).called(1);
    });

    test('tells the lists to read again after a save', () async {
      when(
        () => library.saveDrops(inbox: any(named: 'inbox')),
      ).thenAnswer(
        (_) async => const DropsSaved(notes: [], problems: [], kept: 0),
      );
      final heard = <List<String>>[];
      final subscription = external.stream.listen(heard.add);
      addTearDown(subscription.cancel);
      await repository.saveDrops();
      await Future<void>.delayed(Duration.zero);
      expect(heard, hasLength(1));
    });

    test('a save that fails is an error and tells no one', () async {
      when(() => library.saveDrops(inbox: any(named: 'inbox'))).thenThrow(
        const KansoError.storage(message: 'The folder could not be read.'),
      );
      final heard = <List<String>>[];
      final subscription = external.stream.listen(heard.add);
      addTearDown(subscription.cancel);
      final result = await repository.saveDrops();
      await Future<void>.delayed(Duration.zero);
      expect(
        result,
        const Left<ApplicationError, DropsSaved>(
          ApplicationError.storage(message: 'The folder could not be read.'),
        ),
      );
      expect(heard, isEmpty);
    });

    test("asks for today's note with the phone's date and time", () async {
      when(
        () => library.todayNote(now: any(named: 'now')),
      ).thenAnswer((_) async => 'note:daily-2026-10-26');
      final result = await repository.todayNote(
        DateTime(2026, 10, 26, 8, 5, 9),
      );
      expect(
        result,
        const Right<ApplicationError, String>('note:daily-2026-10-26'),
      );
      verify(
        () => library.todayNote(
          now: const LocalTime(
            year: 2026,
            month: 10,
            day: 26,
            hour: 8,
            minute: 5,
            second: 9,
          ),
        ),
      ).called(1);
    });

    test('a failure to open it is an error', () async {
      when(
        () => library.todayNote(now: any(named: 'now')),
      ).thenThrow(const KansoError.invalid(message: 'That day is not real.'));
      final result = await repository.todayNote(DateTime(2026, 2, 3));
      expect(result.isLeft(), isTrue);
    });
  });

  group('RustLibraryRepository.changes', () {
    test("carries the library's changes and the announced ones", () async {
      final own = StreamController<List<String>>();
      addTearDown(own.close);
      when(() => library.changes()).thenAnswer((_) => own.stream);
      final changes = RustLibraryRepository(library, external).changes();
      final heard = <List<String>>[];
      final subscription = changes.listen(heard.add);
      addTearDown(subscription.cancel);
      own.add(<String>['note:a']);
      await Future<void>.delayed(Duration.zero);
      external.announce();
      await Future<void>.delayed(Duration.zero);
      expect(heard, <List<String>>[
        <String>['note:a'],
        <String>[],
      ]);
    });

    test('lets go of the announcements when it is cancelled', () async {
      final own = StreamController<List<String>>();
      addTearDown(own.close);
      when(() => library.changes()).thenAnswer((_) => own.stream);
      final heard = <List<String>>[];
      final subscription = RustLibraryRepository(
        library,
        external,
      ).changes().listen(heard.add);
      await Future<void>.delayed(Duration.zero);
      await subscription.cancel();
      external.announce();
      await Future<void>.delayed(Duration.zero);
      expect(heard, isEmpty);
    });

    test(
      'a repository without the announcements still reports the library',
      () async {
        when(() => library.changes()).thenAnswer(
          (_) => Stream<List<String>>.value(<String>['note:a']),
        );
        expect(await RustLibraryRepository(library).changes().first, <String>[
          'note:a',
        ]);
      },
    );
  });
}
