// Kanso — comments_goldens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

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

/// One screen of a shared note's comments, as a golden of the whole window.
class _Screen {
  const _Screen(this.name, this.shared, this.interact);

  final String name;

  /// Makes the shared notes the screen needs.
  final FixtureSharedNotesRepository Function() shared;
  final Future<void> Function(WidgetTester tester, BrowseApp app) interact;
}

final String _note = '/notes/note/${Uri.encodeComponent(MockIds.syncDesign)}';

int _ago(Duration ago) => mockNow.subtract(ago).millisecondsSinceEpoch;

/// The mock note, shared, with [comments] on it.
FixtureSharedNotesRepository _withComments(
  List<CommentRow> comments, {
  ShareRole role = ShareRole.owner,
}) {
  final shared = FixtureSharedNotesRepository();
  shared.shares[MockIds.syncDesign] = shareInfo(
    MockIds.syncDesign,
    myRole: role,
    owner: role == ShareRole.owner ? 'ngoni' : 'ana',
  );
  shared.commentLists[MockIds.syncDesign] = comments;
  return shared;
}

/// Open, resolved, mine, edited and orphaned: one of each.
List<CommentRow> _someComments() => <CommentRow>[
  commentRow(
    'c1',
    author: 'ana',
    body: '@ngoni should this wait for the next release?',
    quote: 'Sync design',
    end: 11,
    createdMs: _ago(const Duration(minutes: 12)),
    mentions: const <MentionSpan>[MentionSpan(start: 0, end: 6)],
  ),
  commentRow(
    'c2',
    author: 'ngoni',
    body: 'Rewrote this paragraph after the review.',
    quote: 'conflict resolution',
    start: 40,
    end: 59,
    mine: true,
    createdMs: _ago(const Duration(hours: 3)),
    editedMs: _ago(const Duration(hours: 2)),
  ),
  commentRow(
    'c3',
    body: 'Agreed, done.',
    quote: 'merge order',
    start: 70,
    end: 81,
    resolved: true,
    createdMs: _ago(const Duration(days: 1)),
  ),
  commentRow(
    'c4',
    author: 'cy',
    body: 'The words this was on are gone.',
    quote: 'an old heading that was deleted',
    start: 90,
    orphaned: true,
    createdMs: _ago(const Duration(days: 3)),
  ),
];

Future<void> _openComments(WidgetTester tester, BrowseApp app) async {
  await tester.tap(find.bySemanticsLabel(RegExp('^Comments')));
  await tester.pumpAndSettle();
}

final List<_Screen> _screens = <_Screen>[
  _Screen(
    'comments_button',
    () => _withComments(_someComments()),
    (tester, app) async {},
  ),
  _Screen(
    'comments_sheet',
    () => _withComments(_someComments()),
    _openComments,
  ),
  _Screen(
    'comments_sheet_empty',
    () => _withComments(const <CommentRow>[]),
    _openComments,
  ),
  _Screen(
    'comments_sheet_reader',
    () => _withComments(const <CommentRow>[], role: ShareRole.viewer),
    _openComments,
  ),
  _Screen(
    'comments_sheet_failed',
    () =>
        _withComments(_someComments())
          ..commentFailure = const ApplicationError.network(
            message: 'Kanso could not reach the sync server.',
          ),
    (tester, app) async {
      await _openComments(tester, app);
      await tester.tap(find.text('Resolve').first);
      await tester.pumpAndSettle();
    },
  ),
  _Screen('comment_edit', () => _withComments(_someComments()), (
    tester,
    app,
  ) async {
    await _openComments(tester, app);
    await tester.tap(find.text('Edit'));
    await tester.pumpAndSettle();
  }),
  _Screen('comment_delete', () => _withComments(_someComments()), (
    tester,
    app,
  ) async {
    await _openComments(tester, app);
    await tester.tap(find.text('Delete'));
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
              at: _note,
              shared: screen.shared,
              interact: screen.interact,
            );
          });
        }
      });
    }
  }
}
