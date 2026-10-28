// Kanso — shared_note_live_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/sharing/state/shared_note_live/shared_note_live_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/settle.dart';
import '../../../helpers/sharing_fixtures.dart';

Either<ApplicationError, T> _ok<T>(T value) =>
    Right<ApplicationError, T>(value);

void main() {
  late MockSharedNotesRepository shared;

  setUpAll(() => registerFallbackValue(Object()));

  setUp(() {
    shared = MockSharedNotesRepository();
    when(
      () => shared.openLive(any(), any()),
    ).thenAnswer((_) async => _ok(unit));
    when(
      () => shared.closeLive(any(), any()),
    ).thenAnswer((_) async => _ok(unit));
  });

  SharedNoteLiveBloc build() => SharedNoteLiveBloc(shared);

  void isShared(String id, {required bool shared_}) {
    when(() => shared.info(id)).thenAnswer(
      (_) async => _ok<ShareInfo?>(shared_ ? shareInfo(id) : null),
    );
  }

  group('SharedNoteLiveBloc', () {
    blocTest<SharedNoteLiveBloc, SharedNoteLiveState>(
      'opening a shared note joins its room',
      setUp: () => isShared('n1', shared_: true),
      build: build,
      act: (bloc) => bloc.add(const SharedNoteLiveEvent.opened('n1')),
      expect: () => <SharedNoteLiveState>[
        const SharedNoteLiveState(noteId: 'n1'),
        const SharedNoteLiveState(noteId: 'n1', live: true),
      ],
      verify: (_) => verify(() => shared.openLive('n1', any())).called(1),
    );

    blocTest<SharedNoteLiveBloc, SharedNoteLiveState>(
      'a note that is not shared has no room',
      setUp: () => isShared('n1', shared_: false),
      build: build,
      act: (bloc) => bloc.add(const SharedNoteLiveEvent.opened('n1')),
      verify: (bloc) {
        verifyNever(() => shared.openLive(any(), any()));
        expect(bloc.state.live, isFalse);
      },
    );

    blocTest<SharedNoteLiveBloc, SharedNoteLiveState>(
      'a note whose sharing cannot be read (locked, signed out, offline) has '
      'no room either',
      setUp: () => when(() => shared.info('n1')).thenAnswer(
        (_) async => const Left<ApplicationError, ShareInfo?>(
          ApplicationError.locked(message: 'Locked.'),
        ),
      ),
      build: build,
      act: (bloc) => bloc.add(const SharedNoteLiveEvent.opened('n1')),
      verify: (_) => verifyNever(() => shared.openLive(any(), any())),
    );

    blocTest<SharedNoteLiveBloc, SharedNoteLiveState>(
      'a room that could not be opened is not live',
      setUp: () {
        isShared('n1', shared_: true);
        when(() => shared.openLive('n1', any())).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(
            ApplicationError.network(message: 'Off.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const SharedNoteLiveEvent.opened('n1')),
      verify: (bloc) => expect(bloc.state.live, isFalse),
    );

    blocTest<SharedNoteLiveBloc, SharedNoteLiveState>(
      'coming back to the front joins the room again',
      setUp: () => isShared('n1', shared_: true),
      build: build,
      act: (bloc) async {
        bloc.add(const SharedNoteLiveEvent.opened('n1'));
        await settle();
        bloc.add(const SharedNoteLiveEvent.resumed());
      },
      verify: (_) => verify(() => shared.openLive('n1', any())).called(2),
    );

    test('closing leaves the room that is open, and only that', () async {
      isShared('n1', shared_: true);
      final bloc = build()..add(const SharedNoteLiveEvent.opened('n1'));
      await settle();
      await bloc.close();
      verify(() => shared.closeLive('n1', any())).called(1);

      isShared('n2', shared_: false);
      final quiet = build()..add(const SharedNoteLiveEvent.opened('n2'));
      await settle();
      await quiet.close();
      verifyNever(() => shared.closeLive('n2', any()));
    });

    test('opening another note leaves the room of the first', () async {
      isShared('n1', shared_: true);
      isShared('n2', shared_: true);
      final bloc = build()..add(const SharedNoteLiveEvent.opened('n1'));
      await settle();
      bloc.add(const SharedNoteLiveEvent.opened('n2'));
      await settle();
      verifyInOrder(<void Function()>[
        () => shared.closeLive('n1', any()),
        () => shared.openLive('n2', any()),
      ]);
      await bloc.close();
    });

    test(
      'closing while the room is still opening leaves it once it is open',
      () async {
        isShared('n1', shared_: true);
        final gate = Completer<void>();
        when(() => shared.openLive('n1', any())).thenAnswer((_) async {
          await gate.future;
          return _ok(unit);
        });
        final bloc = build()..add(const SharedNoteLiveEvent.opened('n1'));
        await settle();
        unawaited(bloc.close());
        await settle();
        gate.complete();
        await settle();
        verify(() => shared.closeLive('n1', any())).called(1);
      },
    );

    test(
      'closing while the sharing is still being read leaves no room open',
      () async {
        final gate = Completer<void>();
        when(() => shared.info('n1')).thenAnswer((_) async {
          await gate.future;
          return _ok<ShareInfo?>(shareInfo('n1'));
        });
        final open = <String>{};
        when(() => shared.openLive(any(), any())).thenAnswer((call) async {
          open.add(call.positionalArguments.first as String);
          return _ok(unit);
        });
        when(() => shared.closeLive(any(), any())).thenAnswer((call) async {
          open.remove(call.positionalArguments.first as String);
          return _ok(unit);
        });
        final bloc = build()..add(const SharedNoteLiveEvent.opened('n1'));
        await settle();
        unawaited(bloc.close());
        gate.complete();
        await settle();
        expect(open, isEmpty);
      },
    );

    test('a resume that comes while a note is opening does not reopen the '
        'room of the note left for another', () async {
      final open = <String>{};
      final gate = Completer<void>();
      isShared('n1', shared_: true);
      isShared('n2', shared_: true);
      when(() => shared.openLive(any(), any())).thenAnswer((call) async {
        final id = call.positionalArguments.first as String;
        if (id == 'n1') {
          await gate.future;
        }
        open.add(id);
        return _ok(unit);
      });
      when(() => shared.closeLive(any(), any())).thenAnswer((call) async {
        open.remove(call.positionalArguments.first as String);
        return _ok(unit);
      });
      final bloc = build()
        ..add(const SharedNoteLiveEvent.opened('n1'))
        ..add(const SharedNoteLiveEvent.resumed())
        ..add(const SharedNoteLiveEvent.opened('n2'));
      await settle();
      gate.complete();
      await settle();
      expect(open, <String>{'n2'});
      await bloc.close();
      expect(open, isEmpty);
    });
  });
}
