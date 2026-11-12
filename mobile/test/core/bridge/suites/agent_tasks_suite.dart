// Kanso — agent_tasks_suite.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/agent_tasks.dart';

import 'open_library.dart';

/// Agent tasks through the bridge: the preferences, a start that AI being
/// off refuses, and the calls on a task that isn't there.
void agentTasksSuite(OpenLibrary open) {
  group('KansoAgentTasks', () {
    test('the preferences are saved and read back', () async {
      final tasks = (await open()).ai().agentTasks();
      final first = await tasks.prefs();
      expect(first.enabled, isTrue);
      final saved = await tasks.setPrefs(
        prefs: const AgentTaskPrefs(
          enabled: true,
          newNotes: true,
          edits: false,
          moves: true,
          merges: false,
          timeLimitMinutes: 5,
          maxNotes: 50,
          askBeforeReadingAll: false,
        ),
      );
      expect(saved.edits, isFalse);
      expect((saved.timeLimitMinutes, saved.maxNotes), (5, 50));
      expect((await tasks.prefs()).merges, isFalse);
    });

    test('a task does not start while AI is off', () async {
      final tasks = (await open()).ai().agentTasks();
      await expectLater(
        tasks.start(
          goal: 'Tidy',
          noteIds: <String>[],
          newNoteBook: '',
          confirmedAll: true,
        ),
        throwsA(anything),
      );
      expect(await tasks.tasks(), isEmpty);
    });

    test('a library with no tasks has no sessions', () async {
      final tasks = (await open()).ai().agentTasks();
      expect(await tasks.agentSessions(), isEmpty);
    });

    test('a session that is not there has no transcript', () async {
      final tasks = (await open()).ai().agentTasks();
      expect(await tasks.agentTranscript(key: 'NOPE'), isNull);
    });

    test('a message or an end for a session that is not there fails', () async {
      final tasks = (await open()).ai().agentTasks();
      await expectLater(
        tasks.agentFollowUp(key: 'NOPE', text: 'hi'),
        throwsA(anything),
      );
      await expectLater(tasks.agentEndSession(key: 'NOPE'), throwsA(anything));
    });

    test('a task that is not there has nothing to review or undo', () async {
      final tasks = (await open()).ai().agentTasks();
      await tasks.stop(id: 99);
      await tasks.reply(id: 99, text: 'yes');
      await tasks.discard(id: 99);
      expect(await tasks.canUndo(id: 99), isFalse);
      await expectLater(tasks.review(id: 99), throwsA(anything));
      await expectLater(
        tasks.apply(id: 99, ticked: <int>[0], keep: <int>[], nowMs: 1),
        throwsA(anything),
      );
      await expectLater(tasks.undo(id: 99, nowMs: 1), throwsA(anything));
    });
  });
}
