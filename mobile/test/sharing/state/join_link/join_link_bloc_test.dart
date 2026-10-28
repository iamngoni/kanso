// Kanso — join_link_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/sharing/models/data/share_problem.dart';
import 'package:kanso/sharing/state/join_link/join_link_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/settle.dart';

void main() {
  late MockSharedNotesRepository shared;
  const asked = JoinRow(shareId: 's_9', owner: 'ana', role: ShareRole.editor);

  setUp(() => shared = MockSharedNotesRepository());

  JoinLinkBloc build() => JoinLinkBloc(shared);

  group('JoinLinkBloc', () {
    blocTest<JoinLinkBloc, JoinLinkState>(
      'asks to join with the link, trimmed, and says who was asked',
      setUp: () =>
          when(() => shared.join('https://k.invalid/j/x#k')).thenAnswer(
            (_) async => const Right<ApplicationError, JoinRow>(asked),
          ),
      build: build,
      act: (bloc) => bloc.add(
        const JoinLinkEvent.submitted('  https://k.invalid/j/x#k \n'),
      ),
      expect: () => <JoinLinkState>[
        const JoinLinkState(joining: true),
        const JoinLinkState(asked: asked),
      ],
    );

    blocTest<JoinLinkBloc, JoinLinkState>(
      'an empty field asks nobody',
      build: build,
      act: (bloc) => bloc.add(const JoinLinkEvent.submitted('   ')),
      expect: () => <JoinLinkState>[
        const JoinLinkState(problem: ShareProblem.linkMissing()),
      ],
      verify: (_) => verifyNever(() => shared.join(any())),
    );

    blocTest<JoinLinkBloc, JoinLinkState>(
      'a link that does not work says why in the library’s words',
      setUp: () => when(() => shared.join(any())).thenAnswer(
        (_) async => const Left<ApplicationError, JoinRow>(
          ApplicationError.invalid(message: 'That invite link has expired.'),
        ),
      ),
      build: build,
      act: (bloc) => bloc.add(const JoinLinkEvent.submitted('https://k/j')),
      expect: () => <JoinLinkState>[
        const JoinLinkState(joining: true),
        const JoinLinkState(
          problem: ShareProblem.words('That invite link has expired.'),
        ),
      ],
    );

    blocTest<JoinLinkBloc, JoinLinkState>(
      'a request is not sent twice while one is going',
      setUp: () {
        final gate = Completer<void>();
        when(() => shared.join(any())).thenAnswer((_) async {
          await gate.future;
          return const Right<ApplicationError, JoinRow>(asked);
        });
        addTearDown(() {
          if (!gate.isCompleted) {
            gate.complete();
          }
        });
      },
      build: build,
      act: (bloc) async {
        bloc
          ..add(const JoinLinkEvent.submitted('https://k/j'))
          ..add(const JoinLinkEvent.submitted('https://k/j'));
        await settle();
      },
      verify: (_) => verify(() => shared.join('https://k/j')).called(1),
    );

    blocTest<JoinLinkBloc, JoinLinkState>(
      'joining another note starts from an empty field',
      setUp: () => when(() => shared.join(any())).thenAnswer(
        (_) async => const Right<ApplicationError, JoinRow>(asked),
      ),
      build: build,
      act: (bloc) async {
        bloc.add(const JoinLinkEvent.submitted('https://k/j'));
        await settle();
        bloc.add(const JoinLinkEvent.restarted());
      },
      verify: (bloc) => expect(bloc.state, const JoinLinkState()),
    );
  });
}
