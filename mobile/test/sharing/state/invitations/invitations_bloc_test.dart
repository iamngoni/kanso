// Kanso — invitations_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/sharing/models/data/share_problem.dart';
import 'package:kanso/sharing/state/invitations/invitations_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/settle.dart';
import '../../../helpers/sharing_fixtures.dart';

Either<ApplicationError, T> _ok<T>(T value) =>
    Right<ApplicationError, T>(value);

void main() {
  late MockSharedNotesRepository shared;
  late StreamController<SharedEvent> events;
  final first = invitationRow('s_1');
  final second = invitationRow('s_2', from: 'bo', role: ShareRole.viewer);

  setUp(() {
    shared = MockSharedNotesRepository();
    events = StreamController<SharedEvent>.broadcast();
    addTearDown(events.close);
    when(shared.events).thenAnswer((_) => events.stream);
  });

  InvitationsBloc build() => InvitationsBloc(shared);

  group('InvitationsBloc', () {
    test('starts loading, with nothing waiting', () {
      final state = build().state;
      expect(state.status, LoadStatus.loading);
      expect(state.invitations, isEmpty);
      expect(state.signedOut, isFalse);
    });

    blocTest<InvitationsBloc, InvitationsState>(
      'started reads the invitations',
      setUp: () => when(
        shared.invitations,
      ).thenAnswer((_) async => _ok(<InvitationRow>[first, second])),
      build: build,
      act: (bloc) => bloc.add(const InvitationsEvent.started()),
      expect: () => <InvitationsState>[
        InvitationsState(
          status: LoadStatus.ready,
          invitations: <InvitationRow>[first, second],
        ),
      ],
    );

    test('an invitation accepted while a read is on its way does not come '
        'back with that read', () async {
      final late = Completer<Either<ApplicationError, List<InvitationRow>>>();
      var reads = 0;
      when(shared.invitations).thenAnswer((_) {
        reads++;
        return reads == 1
            ? Future<Either<ApplicationError, List<InvitationRow>>>.value(
                _ok(<InvitationRow>[first, second]),
              )
            : late.future;
      });
      when(() => shared.accept('s_1')).thenAnswer((_) async => _ok('n_1'));
      final bloc = build()..add(const InvitationsEvent.started());
      addTearDown(bloc.close);
      await settle();
      // A sync's news starts a read; the person accepts before it answers.
      bloc.add(const InvitationsEvent.refreshed());
      await settle();
      bloc.add(const InvitationsEvent.accepted('s_1'));
      await settle();
      expect(bloc.state.invitations, <InvitationRow>[second]);
      late.complete(_ok(<InvitationRow>[first, second]));
      await settle();
      expect(bloc.state.invitations, <InvitationRow>[second]);
    });

    test('nor does one declined', () async {
      final late = Completer<Either<ApplicationError, List<InvitationRow>>>();
      var reads = 0;
      when(shared.invitations).thenAnswer((_) {
        reads++;
        return reads == 1
            ? Future<Either<ApplicationError, List<InvitationRow>>>.value(
                _ok(<InvitationRow>[first, second]),
              )
            : late.future;
      });
      when(() => shared.decline('s_2')).thenAnswer((_) async => _ok(unit));
      final bloc = build()..add(const InvitationsEvent.started());
      addTearDown(bloc.close);
      await settle();
      bloc.add(const InvitationsEvent.refreshed());
      await settle();
      bloc.add(const InvitationsEvent.declined('s_2'));
      await settle();
      late.complete(_ok(<InvitationRow>[first, second]));
      await settle();
      expect(bloc.state.invitations, <InvitationRow>[first]);
    });

    blocTest<InvitationsBloc, InvitationsState>(
      'reads them again each time a sync of shared notes reports news',
      setUp: () {
        var reads = 0;
        when(shared.invitations).thenAnswer((_) async {
          reads++;
          return _ok(reads == 1 ? <InvitationRow>[] : <InvitationRow>[first]);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const InvitationsEvent.started());
        await settle();
        events.add(
          const SharedEvent(
            noteIds: <String>[],
            invitations: 1,
            commentNotes: <String>[],
            liveNotes: <String>[],
          ),
        );
        await settle();
      },
      verify: (bloc) {
        verify(shared.invitations).called(2);
        expect(bloc.state.invitations, <InvitationRow>[first]);
      },
    );

    blocTest<InvitationsBloc, InvitationsState>(
      'fails when the first read does, and recovers when asked again',
      setUp: () {
        var reads = 0;
        when(shared.invitations).thenAnswer((_) async {
          reads++;
          return reads == 1
              ? const Left<ApplicationError, List<InvitationRow>>(
                  ApplicationError.network(message: 'Off.'),
                )
              : _ok(<InvitationRow>[first]);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const InvitationsEvent.started());
        await settle();
        expect(bloc.state.status, LoadStatus.failed);
        bloc.add(const InvitationsEvent.refreshed());
      },
      verify: (bloc) {
        expect(bloc.state.status, LoadStatus.ready);
        expect(bloc.state.invitations, <InvitationRow>[first]);
      },
    );

    blocTest<InvitationsBloc, InvitationsState>(
      'a read that fails after one worked keeps the list and says why',
      setUp: () {
        var reads = 0;
        when(shared.invitations).thenAnswer((_) async {
          reads++;
          return reads == 1
              ? _ok(<InvitationRow>[first])
              : const Left<ApplicationError, List<InvitationRow>>(
                  ApplicationError.network(message: 'Off.'),
                );
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const InvitationsEvent.started());
        await settle();
        bloc.add(const InvitationsEvent.refreshed());
      },
      verify: (bloc) {
        expect(bloc.state.status, LoadStatus.ready);
        expect(bloc.state.invitations, <InvitationRow>[first]);
        expect(bloc.state.problem, const ShareProblem.offline());
      },
    );

    blocTest<InvitationsBloc, InvitationsState>(
      'without an account there is nothing to wait for',
      setUp: () => when(shared.invitations).thenAnswer(
        (_) async => const Left<ApplicationError, List<InvitationRow>>(
          ApplicationError.signedOut(message: 'Not signed in.'),
        ),
      ),
      build: build,
      act: (bloc) => bloc.add(const InvitationsEvent.started()),
      expect: () => <InvitationsState>[
        const InvitationsState(status: LoadStatus.ready, signedOut: true),
      ],
    );

    blocTest<InvitationsBloc, InvitationsState>(
      'accepting takes the invitation out and opens the note it brought',
      setUp: () {
        when(
          shared.invitations,
        ).thenAnswer((_) async => _ok(<InvitationRow>[first, second]));
        when(() => shared.accept('s_1')).thenAnswer((_) async => _ok('n_7'));
      },
      build: build,
      act: (bloc) async {
        bloc.add(const InvitationsEvent.started());
        await settle();
        bloc.add(const InvitationsEvent.accepted('s_1'));
      },
      verify: (bloc) {
        expect(bloc.state.invitations, <InvitationRow>[second]);
        expect(bloc.state.opened, 'n_7');
        expect(bloc.state.working, isEmpty);
      },
    );

    blocTest<InvitationsBloc, InvitationsState>(
      'an invitation that cannot be accepted stays, with the reason',
      setUp: () {
        when(
          shared.invitations,
        ).thenAnswer((_) async => _ok(<InvitationRow>[first]));
        when(() => shared.accept('s_1')).thenAnswer(
          (_) async => const Left<ApplicationError, String>(
            ApplicationError.invalid(message: 'That invitation is gone.'),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const InvitationsEvent.started());
        await settle();
        bloc.add(const InvitationsEvent.accepted('s_1'));
      },
      verify: (bloc) {
        expect(bloc.state.invitations, <InvitationRow>[first]);
        expect(bloc.state.opened, isNull);
        expect(
          bloc.state.problem,
          const ShareProblem.words('That invitation is gone.'),
        );
        expect(bloc.state.working, isEmpty);
      },
    );

    blocTest<InvitationsBloc, InvitationsState>(
      'declining takes the invitation out and opens nothing',
      setUp: () {
        when(
          shared.invitations,
        ).thenAnswer((_) async => _ok(<InvitationRow>[first, second]));
        when(() => shared.decline('s_2')).thenAnswer((_) async => _ok(unit));
      },
      build: build,
      act: (bloc) async {
        bloc.add(const InvitationsEvent.started());
        await settle();
        bloc.add(const InvitationsEvent.declined('s_2'));
      },
      verify: (bloc) {
        expect(bloc.state.invitations, <InvitationRow>[first]);
        expect(bloc.state.opened, isNull);
      },
    );

    test('an answer already being sent is not sent twice', () async {
      when(
        shared.invitations,
      ).thenAnswer((_) async => _ok(<InvitationRow>[first]));
      final gate = Completer<void>();
      when(() => shared.accept('s_1')).thenAnswer((_) async {
        await gate.future;
        return _ok('n_7');
      });
      final bloc = build()..add(const InvitationsEvent.started());
      await settle();
      bloc
        ..add(const InvitationsEvent.accepted('s_1'))
        ..add(const InvitationsEvent.accepted('s_1'));
      await settle();
      expect(bloc.state.working, <String>{'s_1'});
      gate.complete();
      await settle();
      verify(() => shared.accept('s_1')).called(1);
      await bloc.close();
    });
  });
}
