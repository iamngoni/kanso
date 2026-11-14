// Kanso — book_notice_inbox_test.dart
// Created 2026-11-14.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, IntAnys, ListAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/sharing/services/book_notice_inbox.dart';

import '../../helpers/fixture_shared_notes_repository.dart';
import '../../helpers/property.dart';

SharedEvent _notices(List<String> notices) => SharedEvent(
  noteIds: const <String>[],
  invitations: 0,
  commentNotes: const <String>[],
  liveNotes: const <String>[],
  notebooks: const <String>[],
  bookInvitations: 0,
  bookNotices: notices,
);

void main() {
  test('hears nothing until it is started, and starts once', () async {
    final shared = FixtureSharedNotesRepository();
    final inbox = BookNoticeInbox(shared);
    addTearDown(inbox.close);
    shared.eventsController.add(_notices(<String>['early']));
    await pumpEventQueue();
    expect(inbox.notices, isEmpty);
    inbox
      ..start()
      ..start();
    shared.eventsController.add(_notices(<String>['one']));
    await pumpEventQueue();
    expect(inbox.notices, <String>['one'], reason: 'heard once, not twice');
  });

  // Each step is a sync that says 1 to 3 notices, or (0) the first notice
  // read: the inbox holds exactly what was said and not read, in order, and
  // tells its listeners each change.
  Glados<List<int>>(
    any.listWithLengthInRange(0, 30, any.intInRange(0, 4)),
    kansoExploreConfig(),
  ).test('holds every notice said and not read, oldest first', (steps) async {
    final shared = FixtureSharedNotesRepository();
    final inbox = BookNoticeInbox(shared)..start();
    final told = <List<String>>[];
    final listening = inbox.changes.listen(told.add);
    final model = <String>[];
    var said = 0;
    for (final step in steps) {
      if (step == 0) {
        inbox.dismissFirst();
        if (model.isNotEmpty) {
          model.removeAt(0);
        }
      } else {
        final batch = <String>[for (var i = 0; i < step; i++) 'n${said++}'];
        model.addAll(batch);
        shared.eventsController.add(_notices(batch));
      }
      await pumpEventQueue();
      expect(inbox.notices, model);
      if (told.isNotEmpty) {
        expect(told.last, model);
      }
    }
    await listening.cancel();
    await inbox.close();
  });
}
