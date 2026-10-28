// Kanso — share_note_view_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';
import 'package:kanso/sharing/state/share_note/share_note_bloc.dart';
import 'package:kanso/sharing/views/widgets/share_invite_bar.dart';
import 'package:kanso/sharing/views/widgets/share_link_maker.dart';
import 'package:kanso/sharing/views/widgets/share_member_menu.dart';
import 'package:kanso/sharing/views/widgets/share_note_view.dart';
import 'package:kanso/sharing/views/widgets/share_prompt_view.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/fixture_system_share_repository.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/sharing_fixtures.dart';

class _Sheet {
  _Sheet(this.shared, this.system, this.bloc);

  final FixtureSharedNotesRepository shared;
  final FixtureSystemShareRepository system;
  final ShareNoteBloc bloc;
}

Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 10; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

Future<_Sheet> _pump(
  WidgetTester tester, {
  FixtureSharedNotesRepository? shared,
  String noteId = 'n1',
}) async {
  final repository = shared ?? FixtureSharedNotesRepository();
  final system = FixtureSystemShareRepository();
  final bloc = ShareNoteBloc(repository, system)
    ..add(ShareNoteEvent.started(noteId));
  closeBlocLater(bloc);
  await pumpRouted(
    tester,
    BlocProvider<ShareNoteBloc>.value(
      value: bloc,
      child: const SizedBox(height: 820, child: ShareNoteView()),
    ),
  );
  await _settle(tester);
  return _Sheet(repository, system, bloc);
}

FixtureSharedNotesRepository _ownedWith(
  List<MemberRow> others, {
  List<MemberRow> requests = const <MemberRow>[],
  List<ShareLinkRow> links = const <ShareLinkRow>[],
}) {
  final shared = FixtureSharedNotesRepository();
  shared.shares['n1'] = shareInfo(
    'n1',
    members: <MemberRow>[
      memberRow('ngoni', role: ShareRole.owner, me: true, verified: true),
      ...others,
    ],
    requests: requests,
    links: links,
  );
  return shared;
}

Finder _inSheet(Finder finder) =>
    find.descendant(of: find.byType(ShareNoteView), matching: finder);

Future<void> _tapText(WidgetTester tester, String text) async {
  final target = find.text(text).first;
  await tester.ensureVisible(target);
  await tester.pump();
  await tester.tap(target);
  await _settle(tester);
}

void main() {
  group('ShareNoteView for a note that is not shared', () {
    testWidgets('invites the first person, who makes the note shared', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository()
        ..people['ana'] = PersonLookup.found(
          personRow('ana', verified: true),
        );
      await _pump(tester, shared: shared);
      expect(find.text('Share'), findsOneWidget);
      expect(find.textContaining('Invite people by their Kanso'), findsOne);
      expect(find.byType(ShareInviteBar), findsOneWidget);
      expect(find.text('People'), findsNothing);

      await tester.enterText(find.byType(EditableText), 'ana');
      await _tapText(tester, 'Invite');

      expect(shared.calls, contains('share n1 ana editor'));
      expect(
        find.text('Invited ana. The note shows for them once they accept.'),
        findsOneWidget,
      );
      expect(find.text('People'), findsOneWidget);
      expect(find.text('ana'), findsOneWidget);
      expect(
        find.text('Invited, hasn’t accepted yet · Fingerprint verified'),
        findsOneWidget,
      );
      // The field is ready for the next person.
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).controller.text,
        isEmpty,
      );
    });

    testWidgets('the role chosen is the role given', (tester) async {
      final shared = FixtureSharedNotesRepository()
        ..people['ana'] = PersonLookup.found(
          personRow('ana', verified: true),
        );
      await _pump(tester, shared: shared);
      await _tapText(tester, 'Can view');
      await tester.enterText(find.byType(EditableText), 'ana');
      await _tapText(tester, 'Invite');
      expect(shared.calls, contains('share n1 ana viewer'));
    });

    testWidgets('says to type a username when none was typed', (tester) async {
      final shared = FixtureSharedNotesRepository();
      await _pump(tester, shared: shared);
      await _tapText(tester, 'Invite');
      expect(
        find.text('Type the username of the person to invite.'),
        findsOneWidget,
      );
      expect(shared.calls.where((c) => c.startsWith('lookup')), isEmpty);
    });

    testWidgets('says why a username is not found, in the library’s words', (
      tester,
    ) async {
      await _pump(tester);
      await tester.enterText(find.byType(EditableText), 'zed');
      await _tapText(tester, 'Invite');
      expect(
        find.textContaining('There’s no Kanso account called zed'),
        findsOneWidget,
      );
    });

    testWidgets('asks to compare a new person’s fingerprint before inviting', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository()
        ..people['bo'] = PersonLookup.found(personRow('bo'));
      await _pump(tester, shared: shared);
      await tester.enterText(find.byType(EditableText), 'bo');
      await _tapText(tester, 'Invite');
      expect(
        find.textContaining('You’re sharing with bo for the first time'),
        findsOneWidget,
      );
      expect(find.text('11111'), findsOneWidget);
      expect(find.text('44444'), findsOneWidget);
      expect(shared.calls.where((c) => c.startsWith('share')), isEmpty);

      await _tapText(tester, 'Mark as verified and invite');
      expect(
        shared.calls,
        containsAllInOrder(<String>[
          'verify a_bo',
          'share n1 bo editor',
        ]),
      );
      expect(find.text('People'), findsOneWidget);
    });

    testWidgets('Back from the fingerprint question invites nobody', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository()
        ..people['bo'] = PersonLookup.found(personRow('bo'));
      await _pump(tester, shared: shared);
      await tester.enterText(find.byType(EditableText), 'bo');
      await _tapText(tester, 'Invite');
      await _tapText(tester, 'Back');
      expect(find.byType(ShareInviteBar), findsOneWidget);
      expect(shared.calls.where((c) => c.startsWith('share')), isEmpty);
    });

    testWidgets('shows keys that changed with a warning and the old and the '
        'new fingerprint', (tester) async {
      final shared = FixtureSharedNotesRepository()
        ..people['bo'] = PersonLookup.changed(
          pinned: personRow('bo', fingerprint: '11111 11111'),
          offered: personRow('bo', fingerprint: '99999 99999'),
        );
      await _pump(tester, shared: shared);
      await tester.enterText(find.byType(EditableText), 'bo');
      await _tapText(tester, 'Invite');
      expect(find.textContaining('bo’s keys changed'), findsOneWidget);
      expect(find.text('Before'), findsOneWidget);
      expect(find.text('Now'), findsOneWidget);
      expect(find.text('11111'), findsNWidgets(2));
      expect(find.text('99999'), findsNWidgets(2));

      await _tapText(tester, 'Trust the new keys');
      // Trusted is not verified: the new fingerprint is compared next, as
      // the desktop asks.
      expect(shared.calls, isNot(contains('share n1 bo editor')));
      expect(find.text('Mark as verified and invite'), findsOneWidget);
      expect(find.text('Back'), findsOneWidget);
      await _tapText(tester, 'Mark as verified and invite');
      expect(
        shared.calls,
        containsAllInOrder(<String>[
          'acceptNewKey bo 99999 99999',
          'verify a_bo',
          'share n1 bo editor',
        ]),
      );
    });

    testWidgets('invites someone new without comparing the fingerprint when '
        'the person chooses to', (tester) async {
      final shared = FixtureSharedNotesRepository()
        ..people['bo'] = PersonLookup.found(personRow('bo'));
      await _pump(tester, shared: shared);
      await tester.enterText(find.byType(EditableText), 'bo');
      await _tapText(tester, 'Invite');
      expect(find.textContaining('first time'), findsOneWidget);
      // The step's own Invite, under the fingerprint.
      await tester.tap(
        find.descendant(
          of: find.byType(SharePromptView),
          matching: find.text('Invite'),
        ),
      );
      await tester.pumpAndSettle();
      expect(shared.calls, isNot(contains('verify a_bo')));
      expect(shared.calls, contains('share n1 bo editor'));
    });

    testWidgets('Don’t invite leaves changed keys untrusted', (tester) async {
      final shared = FixtureSharedNotesRepository()
        ..people['bo'] = PersonLookup.changed(
          offered: personRow('bo', fingerprint: '99999 99999'),
        );
      await _pump(tester, shared: shared);
      await tester.enterText(find.byType(EditableText), 'bo');
      await _tapText(tester, 'Invite');
      expect(find.text('Before'), findsNothing);
      await _tapText(tester, 'Don’t invite');
      expect(shared.calls.where((c) => c.startsWith('acceptNewKey')), isEmpty);
      expect(find.byType(ShareInviteBar), findsOneWidget);
    });
  });

  group('ShareNoteView for a note that cannot be shared', () {
    testWidgets('a locked note says that and nothing else', (tester) async {
      final shared = FixtureSharedNotesRepository()..locked.add('n1');
      await _pump(tester, shared: shared);
      expect(
        find.text(
          'Locked notes, and notes in locked notebooks, can’t be shared. '
          'Remove the lock first.',
        ),
        findsOneWidget,
      );
      expect(find.byType(ShareInviteBar), findsNothing);
      expect(find.byType(EditableText), findsNothing);
      expect(find.byType(KansoButton), findsNothing);
      expect(find.text('Invite'), findsNothing);
      expect(find.text('People'), findsNothing);
      // Nothing about the note was asked beyond whether it can be shared.
      expect(
        shared.calls.where(
          (call) => call.startsWith('lookup') || call.startsWith('share '),
        ),
        isEmpty,
      );
    });

    testWidgets('without an account it offers to sign in', (tester) async {
      final shared = FixtureSharedNotesRepository()..signedOut = true;
      await _pump(tester, shared: shared);
      expect(find.textContaining('It needs a Kanso account'), findsOneWidget);
      expect(find.text('Sign in…'), findsOneWidget);
      expect(find.byType(ShareInviteBar), findsNothing);
    });

    testWidgets('a note the library refuses says why', (tester) async {
      final shared = FixtureSharedNotesRepository()
        ..unshareable['n1'] = 'The note is in the trash.';
      await _pump(tester, shared: shared);
      expect(find.text('The note is in the trash.'), findsOneWidget);
      expect(find.byType(ShareInviteBar), findsNothing);
    });

    testWidgets('a sharing that could not be read says so and tries again', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository()
        ..failure = const ApplicationError.network(message: 'Off.');
      await _pump(tester, shared: shared);
      expect(find.text('Sharing didn’t load'), findsOneWidget);
      expect(find.textContaining('Off.'), findsNothing);
      shared.failure = null;
      await _tapText(tester, 'Try again');
      expect(find.text('Sharing didn’t load'), findsNothing);
      expect(find.byType(ShareInviteBar), findsOneWidget);
    });

    testWidgets('shows still bars while the sharing is read', (tester) async {
      final gate = Completer<void>();
      addTearDown(() {
        if (!gate.isCompleted) {
          gate.complete();
        }
      });
      final shared = FixtureSharedNotesRepository()..gate = gate;
      final handle = tester.ensureSemantics();
      await _pump(tester, shared: shared);
      expect(find.bySemanticsLabel('Loading sharing'), findsOneWidget);
      expect(find.byType(ShareInviteBar), findsNothing);
      handle.dispose();
    });
  });

  group('ShareNoteView for a note the person shares', () {
    testWidgets('lists the people with their roles, the owner’s without a '
        'control', (tester) async {
      await _pump(
        tester,
        shared: _ownedWith(<MemberRow>[
          memberRow('ana', verified: true),
          memberRow('bo', role: ShareRole.viewer, invited: true),
          memberRow('cy', left: true),
        ]),
      );
      expect(find.text('People'), findsOneWidget);
      expect(find.text('ngoni'), findsOneWidget);
      expect(find.text('Owner'), findsOneWidget);
      expect(find.text('You'), findsOneWidget);
      expect(find.text('Fingerprint verified'), findsOneWidget);
      expect(find.text('Invited, hasn’t accepted yet'), findsOneWidget);
      expect(find.text('Left'), findsOneWidget);
      // Controls for ana and bo; none for the owner or for someone who left.
      expect(
        find.widgetWithText(KansoButton, 'Can edit'),
        findsOneWidget,
      );
      expect(find.widgetWithText(KansoButton, 'Can view'), findsOneWidget);
    });

    testWidgets('changes a role through the menu', (tester) async {
      final shared = _ownedWith(<MemberRow>[memberRow('ana')]);
      await _pump(tester, shared: shared);
      await tester.tap(find.widgetWithText(KansoButton, 'Can edit'));
      await _settle(tester);
      expect(find.byType(ShareMemberMenu), findsOneWidget);
      await tester.tap(
        find.descendant(
          of: find.byType(ShareMemberMenu),
          matching: find.text('Can view'),
        ),
      );
      await _settle(tester);
      expect(find.byType(ShareMemberMenu), findsNothing);
      expect(shared.calls, contains('setRole n1 a_ana viewer'));
      expect(find.widgetWithText(KansoButton, 'Can view'), findsOneWidget);
    });

    testWidgets('removes a person through the menu', (tester) async {
      final shared = _ownedWith(<MemberRow>[memberRow('ana')]);
      await _pump(tester, shared: shared);
      await tester.tap(find.widgetWithText(KansoButton, 'Can edit'));
      await _settle(tester);
      await tester.tap(find.text('Remove from note'));
      await _settle(tester);
      expect(shared.calls, contains('remove n1 a_ana'));
      expect(find.text('ana'), findsNothing);
    });

    testWidgets('lets in, or turns away, someone who asked through a link', (
      tester,
    ) async {
      final shared = _ownedWith(
        <MemberRow>[],
        requests: <MemberRow>[
          memberRow('cy', role: ShareRole.viewer),
          memberRow('di', role: ShareRole.commenter),
        ],
      );
      await _pump(tester, shared: shared);
      expect(find.text('Asked to join'), findsOneWidget);
      expect(find.text('cy asked to join'), findsOneWidget);
      expect(
        find.text('Can view · Fingerprint 12345 67890 24680 13579'),
        findsOneWidget,
      );
      await _tapText(tester, 'Approve');
      expect(shared.calls, contains('approve n1 a_cy'));
      expect(find.text('cy asked to join'), findsNothing);
      await _tapText(tester, 'Decline');
      expect(shared.calls, contains('declineRequest n1 a_di'));
      expect(find.text('Asked to join'), findsNothing);
    });

    testWidgets('makes a link with a role and days, shows it once, copies and '
        'sends it', (tester) async {
      final shared = _ownedWith(<MemberRow>[]);
      final sheet = await _pump(tester, shared: shared);
      expect(find.text('Invite links'), findsOneWidget);
      await tester.tap(
        find.descendant(
          of: find.byType(ShareLinkMaker),
          matching: find.text('Can view'),
        ),
      );
      await tester.tap(
        find.descendant(
          of: find.byType(ShareLinkMaker),
          matching: find.text('30 days'),
        ),
      );
      await tester.pump();
      await _tapText(tester, 'Make link');
      expect(shared.calls, contains('createLink n1 viewer 30'));
      expect(find.text(shared.linkAddress), findsOneWidget);
      expect(
        find.textContaining('Copy it now: it can’t be shown again.'),
        findsOneWidget,
      );
      expect(find.text('Can view, for 30 more days'), findsOneWidget);

      await _tapText(tester, 'Copy');
      expect(sheet.system.copied, <String>[shared.linkAddress]);
      expect(find.text('Copied'), findsOneWidget);

      await _tapText(tester, 'Send');
      expect(sheet.system.sent, <String>[shared.linkAddress]);
      expect(sheet.system.origins.single, isNotNull);
    });

    testWidgets('cancels a link', (tester) async {
      final shared = _ownedWith(
        <MemberRow>[],
        links: <ShareLinkRow>[
          ShareLinkRow(
            id: 'l_1',
            role: ShareRole.editor,
            expiresMs: clock.now().millisecondsSinceEpoch + 3 * 86400000,
          ),
        ],
      );
      await _pump(tester, shared: shared);
      expect(find.text('Can edit, for 3 more days'), findsOneWidget);
      await _tapText(tester, 'Cancel link');
      expect(shared.calls, contains('cancelLink n1 l_1'));
      expect(find.textContaining('more days'), findsNothing);
    });

    testWidgets('an expired link says so', (tester) async {
      final shared = _ownedWith(
        <MemberRow>[],
        links: <ShareLinkRow>[
          const ShareLinkRow(id: 'l_1', role: ShareRole.viewer, expiresMs: 1),
        ],
      );
      await _pump(tester, shared: shared);
      expect(find.text('Can view, expired'), findsOneWidget);
    });

    testWidgets('reviews the new keys of a member whose keys changed', (
      tester,
    ) async {
      final shared =
          _ownedWith(<MemberRow>[
              memberRow('ana', keyChanged: true),
            ])
            ..people['ana'] = PersonLookup.changed(
              pinned: personRow('ana', fingerprint: '11111 11111'),
              offered: personRow('ana', fingerprint: '55555 55555'),
            );
      await _pump(tester, shared: shared);
      expect(find.text('Can’t be checked'), findsOneWidget);
      await _tapText(tester, 'Accept the new key');
      expect(find.textContaining('ana’s keys changed'), findsOneWidget);
      expect(find.text('55555'), findsNWidgets(2));
      // The prompt of a member offers no invitation to turn down.
      expect(find.text('Don’t invite'), findsNothing);
      expect(find.text('Not now'), findsOneWidget);

      await _tapText(tester, 'Trust the new keys');
      expect(shared.calls, contains('acceptNewKey ana 55555 55555'));
      expect(find.byType(ShareInviteBar), findsOneWidget);
    });

    testWidgets('stops sharing after asking once', (tester) async {
      final shared = _ownedWith(<MemberRow>[memberRow('ana')]);
      await _pump(tester, shared: shared);
      await _tapText(tester, 'Stop sharing');
      expect(
        find.textContaining('Stopping sharing ends the note for everyone'),
        findsOneWidget,
      );
      expect(shared.calls.where((c) => c.startsWith('stop')), isEmpty);
      await _tapText(tester, 'Stop sharing for everyone');
      expect(shared.calls, contains('stop n1'));
      expect(
        find.text(
          'Sharing stopped. Everyone keeps a private copy of the note as it '
          'is.',
        ),
        findsOneWidget,
      );
      expect(find.text('People'), findsNothing);
    });

    testWidgets('Keep sharing leaves the note shared', (tester) async {
      final shared = _ownedWith(<MemberRow>[memberRow('ana')]);
      await _pump(tester, shared: shared);
      await _tapText(tester, 'Stop sharing');
      await _tapText(tester, 'Keep sharing');
      expect(find.text('Stop sharing'), findsOneWidget);
      expect(shared.calls.where((c) => c.startsWith('stop')), isEmpty);
    });
  });

  group('ShareNoteView for a note shared with the person', () {
    FixtureSharedNotesRepository sharedByAna(ShareRole role) {
      final shared = FixtureSharedNotesRepository();
      shared.shares['n1'] = shareInfo(
        'n1',
        owner: 'ana',
        myRole: role,
        members: <MemberRow>[
          memberRow('ana', role: ShareRole.owner, verified: true),
          memberRow('ngoni', role: role, me: true),
        ],
      );
      return shared;
    }

    testWidgets('has no invite field, links or role controls, and says what '
        'the person can do', (tester) async {
      await _pump(tester, shared: sharedByAna(ShareRole.commenter));
      expect(
        find.text(
          'ana shared this note with you. You can read it and comment on it.',
        ),
        findsOneWidget,
      );
      expect(find.byType(ShareInviteBar), findsNothing);
      expect(find.text('Invite links'), findsNothing);
      expect(_inSheet(find.byType(KansoButton)), findsOneWidget);
      expect(find.text('Leave'), findsOneWidget);
    });

    testWidgets('leaves the note after asking once', (tester) async {
      final shared = sharedByAna(ShareRole.editor);
      await _pump(tester, shared: shared);
      await _tapText(tester, 'Leave');
      expect(
        find.textContaining('Leaving takes you out of the note'),
        findsOneWidget,
      );
      await _tapText(tester, 'Leave the note');
      expect(shared.calls, contains('leave n1'));
      expect(
        find.text('You left the note. Your copy of it is private now.'),
        findsOneWidget,
      );
    });
  });
}
