// Kanso — sharing_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';

import '../../../helpers/app_golden.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/sharing_fixtures.dart';

/// One screen of sharing, as a golden of the whole window.
class _Screen {
  const _Screen(this.name, this.shared, this.interact, {this.at = '/notes'});

  final String name;
  final String at;

  /// Makes the shared notes the screen needs.
  final FixtureSharedNotesRepository Function() shared;
  final Future<void> Function(WidgetTester tester, BrowseApp app) interact;
}

String _noteAt(String id) => '/notes/note/${Uri.encodeComponent(id)}';

Future<void> _openShare(WidgetTester tester, BrowseApp app) async {
  await tester.tap(find.bySemanticsLabel('Share'));
  await tester.pumpAndSettle();
}

Future<void> _open(WidgetTester tester, BrowseApp app, String path) async {
  unawaited(app.router.push<void>(path));
  await tester.pumpAndSettle();
}

/// The mock note, shared with people, a person to review and a link.
FixtureSharedNotesRepository _sharedNote() {
  final shared = FixtureSharedNotesRepository();
  shared.shares[MockIds.syncDesign] = shareInfo(
    MockIds.syncDesign,
    members: <MemberRow>[
      memberRow('ngoni', role: ShareRole.owner, me: true, verified: true),
      memberRow('ana', verified: true),
      memberRow('bo', role: ShareRole.viewer, invited: true),
      memberRow('cy', role: ShareRole.commenter, keyChanged: true),
    ],
    requests: <MemberRow>[
      memberRow('di', role: ShareRole.viewer, fingerprint: '55555 66666'),
    ],
    links: <ShareLinkRow>[
      ShareLinkRow(
        id: 'l_1',
        role: ShareRole.commenter,
        expiresMs: mockNow.millisecondsSinceEpoch + 6 * 86400000 - 3600000,
      ),
    ],
  );
  return shared;
}

final List<_Screen> _screens = <_Screen>[
  _Screen(
    'share_sheet_not_shared',
    FixtureSharedNotesRepository.new,
    _openShare,
    at: _noteAt(MockIds.syncDesign),
  ),
  _Screen(
    'share_sheet_shared',
    _sharedNote,
    _openShare,
    at: _noteAt(MockIds.syncDesign),
  ),
  _Screen(
    'share_sheet_locked',
    () => FixtureSharedNotesRepository()..locked.add(MockIds.privateJournal),
    _openShare,
    at: _noteAt(MockIds.privateJournal),
  ),
  _Screen(
    'share_sheet_fingerprint',
    () =>
        FixtureSharedNotesRepository()
          ..people['bo'] = PersonLookup.found(personRow('bo')),
    (tester, app) async {
      await _openShare(tester, app);
      await tester.enterText(find.byType(EditableText), 'bo');
      await tester.tap(find.text('Invite'));
      await tester.pumpAndSettle();
    },
    at: _noteAt(MockIds.syncDesign),
  ),
  _Screen(
    'share_sheet_signed_out',
    () => FixtureSharedNotesRepository()..signedOut = true,
    _openShare,
    at: _noteAt(MockIds.syncDesign),
  ),
  _Screen(
    'shared_with_you',
    () => FixtureSharedNotesRepository()
      ..invitationList = <InvitationRow>[
        invitationRow('s_1'),
        invitationRow('s_2', from: 'bo', role: ShareRole.viewer),
        invitationRow('s_3', from: 'cy', role: ShareRole.commenter),
      ],
    (tester, app) => _open(tester, app, '/shared'),
  ),
  _Screen(
    'shared_with_you_empty',
    FixtureSharedNotesRepository.new,
    (tester, app) => _open(tester, app, '/shared'),
  ),
  _Screen(
    'shared_with_you_failed',
    () =>
        FixtureSharedNotesRepository()
          ..failure = const ApplicationError.network(message: 'Off.'),
    (tester, app) => _open(tester, app, '/shared'),
  ),
  _Screen('join_link', FixtureSharedNotesRepository.new, (tester, app) async {
    await _open(tester, app, '/shared');
    await _open(tester, app, '/shared/join');
  }),
  _Screen('join_link_asked', FixtureSharedNotesRepository.new, (
    tester,
    app,
  ) async {
    await _open(tester, app, '/shared');
    await _open(tester, app, '/shared/join');
    await tester.enterText(
      find.byType(EditableText),
      'https://kansoapp.app/join/k3x9#secret',
    );
    await tester.tap(find.text('Join'));
    await tester.pumpAndSettle();
  }),
];

void main() {
  final viewports = <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ];
  for (final viewport in viewports) {
    for (final brightness in Brightness.values) {
      group('${viewport.slug}, ${brightness.name}', () {
        for (final screen in _screens) {
          testWidgets(screen.name, (tester) async {
            await expectAppGoldenOnBothPlatforms(
              tester,
              name: '${viewport.slug}_${screen.name}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              at: screen.at,
              shared: screen.shared,
              interact: screen.interact,
            );
          });
        }
      });
    }
  }
}
