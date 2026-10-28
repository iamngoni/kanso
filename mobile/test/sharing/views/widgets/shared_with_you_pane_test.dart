// Kanso — shared_with_you_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/sharing/state/invitations/invitations_bloc.dart';
import 'package:kanso/sharing/state/join_link/join_link_bloc.dart';
import 'package:kanso/sharing/views/widgets/invitation_tile.dart';
import 'package:kanso/sharing/views/widgets/join_link_pane.dart';
import 'package:kanso/sharing/views/widgets/shared_with_you_pane.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/sharing_fixtures.dart';

Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 8; i++) {
    await tester.pump(const Duration(milliseconds: 30));
  }
}

Future<GoRouter> _pumpInvitations(
  WidgetTester tester,
  FixtureSharedNotesRepository shared,
) async {
  final bloc = InvitationsBloc(shared)..add(const InvitationsEvent.started());
  closeBlocLater(bloc);
  final router = await pumpRouted(
    tester,
    BlocProvider<InvitationsBloc>.value(
      value: bloc,
      child: const SharedWithYouPane(),
    ),
  );
  await _settle(tester);
  return router;
}

/// A shared notes fixture that holds back the invitations.
class _SlowShared extends FixtureSharedNotesRepository {
  _SlowShared(this.until);

  final Future<void> until;

  @override
  Future<Either<ApplicationError, List<InvitationRow>>> invitations() async {
    await until;
    return super.invitations();
  }
}

void main() {
  group('SharedWithYouPane', () {
    testWidgets('lists the invitations by who they are from, never by title', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository()
        ..invitationList = <InvitationRow>[
          invitationRow('s_1'),
          invitationRow('s_2', from: 'bo', role: ShareRole.viewer),
        ];
      await _pumpInvitations(tester, shared);
      expect(find.text('Shared with you'), findsOneWidget);
      expect(find.text('A note from ana'), findsOneWidget);
      expect(find.text('A note from bo'), findsOneWidget);
      expect(find.text('Can edit'), findsOneWidget);
      expect(find.text('Can view'), findsOneWidget);
      expect(find.text('Accept'), findsNWidgets(2));
      expect(find.text('Decline'), findsNWidgets(2));
      expect(find.text('Join with a link'), findsOneWidget);
    });

    testWidgets('accepting an invitation opens the note it brings', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository()
        ..invitationList = <InvitationRow>[invitationRow('s_1')];
      final router = await _pumpInvitations(tester, shared);
      await tester.tap(find.text('Accept'));
      await _settle(tester);
      expect(shared.calls, contains('accept s_1'));
      expect(whereIs(router), '/notes/note/n_from_s_1');
    });

    testWidgets('declining an invitation takes it out and opens nothing', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository()
        ..invitationList = <InvitationRow>[
          invitationRow('s_1'),
          invitationRow('s_2', from: 'bo'),
        ];
      final router = await _pumpInvitations(tester, shared);
      await tester.tap(find.text('Decline').first);
      await _settle(tester);
      expect(shared.calls, contains('decline s_1'));
      expect(find.text('A note from ana'), findsNothing);
      expect(find.text('A note from bo'), findsOneWidget);
      expect(whereIs(router), '/start');
    });

    testWidgets('says so when there is nothing, and offers to join with a '
        'link', (tester) async {
      final router = await _pumpInvitations(
        tester,
        FixtureSharedNotesRepository(),
      );
      expect(find.text('Nothing shared with you'), findsOneWidget);
      expect(
        find.text(
          'When someone shares a note with you, it waits here until you '
          'accept it.',
        ),
        findsOneWidget,
      );
      expect(find.byType(InvitationTile), findsNothing);
      await tester.tap(find.text('Join with a link'));
      await tester.pumpAndSettle();
      expect(topOf(router), '/shared/join');
    });

    testWidgets('says it needs an account when none is signed in', (
      tester,
    ) async {
      await _pumpInvitations(
        tester,
        FixtureSharedNotesRepository()..signedOut = true,
      );
      expect(find.text('Sign in to get shared notes'), findsOneWidget);
      expect(find.text('Join with a link'), findsNothing);
      expect(find.text('Sign in…'), findsOneWidget);
    });

    testWidgets('says plainly when they could not be read and tries again', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository()
        ..failure = const ApplicationError.network(message: 'Off.');
      await _pumpInvitations(tester, shared);
      expect(find.text('Shared notes didn’t load'), findsOneWidget);
      expect(find.textContaining('Off.'), findsNothing);
      shared
        ..failure = null
        ..invitationList = <InvitationRow>[invitationRow('s_1')];
      await tester.tap(find.text('Try again'));
      await _settle(tester);
      expect(find.text('A note from ana'), findsOneWidget);
    });

    testWidgets('shows still bars while they are read', (tester) async {
      final gate = Completer<void>();
      final shared = _SlowShared(gate.future);
      addTearDown(() {
        if (!gate.isCompleted) {
          gate.complete();
        }
      });
      final handle = tester.ensureSemantics();
      await _pumpInvitations(tester, shared);
      expect(find.bySemanticsLabel('Loading shared notes'), findsOneWidget);
      handle.dispose();
    });

    testWidgets('reads them again on the refresh button', (tester) async {
      final shared = FixtureSharedNotesRepository();
      await _pumpInvitations(tester, shared);
      shared.invitationList = <InvitationRow>[invitationRow('s_9', from: 'di')];
      final handle = tester.ensureSemantics();
      await tester.tap(find.bySemanticsLabel('Look for new shared notes'));
      await _settle(tester);
      handle.dispose();
      expect(find.text('A note from di'), findsOneWidget);
    });

    testWidgets('a sync that finds a new invitation shows it at once', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository();
      await _pumpInvitations(tester, shared);
      expect(find.byType(InvitationTile), findsNothing);
      shared.invitationList = <InvitationRow>[invitationRow('s_4', from: 'ed')];
      shared.eventsController.add(
        const SharedEvent(
          noteIds: <String>[],
          invitations: 1,
          commentNotes: <String>[],
          liveNotes: <String>[],
        ),
      );
      await _settle(tester);
      expect(find.text('A note from ed'), findsOneWidget);
    });
  });

  group('JoinLinkPane', () {
    Future<FixtureSharedNotesRepository> pumpJoin(
      WidgetTester tester, {
      FixtureSharedNotesRepository? shared,
    }) async {
      final repository = shared ?? FixtureSharedNotesRepository();
      final bloc = JoinLinkBloc(repository);
      closeBlocLater(bloc);
      await pumpRouted(
        tester,
        BlocProvider<JoinLinkBloc>.value(
          value: bloc,
          child: const JoinLinkPane(),
        ),
      );
      return repository;
    }

    testWidgets('asks to join with the link typed and says who was asked', (
      tester,
    ) async {
      final shared = await pumpJoin(tester);
      expect(find.text('Join with a link'), findsOneWidget);
      expect(find.text('Invite link'), findsOneWidget);
      await tester.enterText(
        find.byType(EditableText),
        ' https://kansoapp.app/join/k3x9#secret ',
      );
      await tester.tap(find.text('Join'));
      await _settle(tester);
      expect(shared.calls, <String>[
        'join https://kansoapp.app/join/k3x9#secret',
      ]);
      expect(find.text('Request sent'), findsOneWidget);
      expect(
        find.text(
          'Asked ana to let you in. The note shows in your library once '
          'they approve.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('joining another note starts from an empty field', (
      tester,
    ) async {
      await pumpJoin(tester);
      await tester.enterText(find.byType(EditableText), 'https://k/j');
      await tester.tap(find.text('Join'));
      await _settle(tester);
      await tester.tap(find.text('Join another note'));
      await _settle(tester);
      expect(find.text('Request sent'), findsNothing);
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).controller.text,
        isEmpty,
      );
    });

    testWidgets('says to paste a link when none is', (tester) async {
      final shared = await pumpJoin(tester);
      await tester.tap(find.text('Join'));
      await _settle(tester);
      expect(find.text('Paste the invite link first.'), findsOneWidget);
      expect(shared.calls, isEmpty);
    });

    testWidgets('says why a link did not work, in the library words', (
      tester,
    ) async {
      await pumpJoin(
        tester,
        shared: FixtureSharedNotesRepository()
          ..failure = const ApplicationError.invalid(
            message: 'That invite link has expired.',
          ),
      );
      await tester.enterText(find.byType(EditableText), 'https://k/j');
      await tester.tap(find.text('Join'));
      await _settle(tester);
      expect(find.text('That invite link has expired.'), findsOneWidget);
      expect(find.text('Request sent'), findsNothing);
    });
  });
}
