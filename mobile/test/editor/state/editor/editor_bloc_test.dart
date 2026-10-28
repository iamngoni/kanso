// Kanso — editor_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:typed_data';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart' show Left, Right, Unit;
import 'package:flutter/widgets.dart' show AppLifecycleState;
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/models/data/media_source.dart';
import 'package:kanso/core/bridge/api/types.dart' as core;
import 'package:kanso/core/bridge/api/types.dart' hide EditorEvent;
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/editor/models/data/editor_destination.dart';
import 'package:kanso/editor/models/data/editor_status.dart';
import 'package:kanso/editor/models/data/flush_reason.dart';
import 'package:kanso/editor/state/editor/editor_bloc.dart';

import '../../../helpers/fake_editor_document.dart';
import '../../../helpers/fake_editor_repository.dart';
import '../../../helpers/fake_media_picker_repository.dart';
import '../../../helpers/fake_web_link_repository.dart';

const _noHistory = HistoryState(canUndo: false, canRedo: false);
const _context = EditorContext(
  inList: true,
  inTable: false,
  inCode: false,
  completionHere: false,
  history: _noHistory,
);
const _completing = EditorContext(
  inList: false,
  inTable: false,
  inCode: false,
  completionHere: true,
  history: _noHistory,
);
const _completion = Completion(
  kind: CompletionKind.wiki,
  replace: Range16(start: 0, end: 2),
  items: <CompletionItem>[
    CompletionItem(label: 'Plan', insert: '[[Plan]]', newNote: false),
  ],
);
const _property = Property(
  key: 'owner',
  kind: PropertyKind.text,
  text: 'Ada',
  items: <String>[],
);

ShareFile _file(String name) => ShareFile(
  name: name,
  bytes: Uint8List.fromList(<int>[1]),
);

/// The revision a state says was saved, if it says so.
String? _savedRev(EditorState state) {
  final save = state.save;
  return save is SaveState_Saved ? save.rev : null;
}

/// Whether a state asks about a change made elsewhere; true for a deletion.
bool? _conflictDeleted(EditorState state) {
  final save = state.save;
  return save is SaveState_Conflict ? save.deleted : null;
}

void main() {
  late FakeEditorDocument document;
  late FakeMediaPickerRepository picker;
  late FakeWebLinkRepository links;

  setUp(() {
    links = FakeWebLinkRepository();
    picker = FakeMediaPickerRepository();
    document = FakeEditorDocument(text: 'the text', title: 'A title')
      ..properties = const <Property>[_property];
    addTearDown(document.events_.close);
  });

  EditorBloc build({ApplicationError? error}) {
    final bloc = EditorBloc(
      FakeEditorRepository(document: document, error: error),
      links,
      picker,
    );
    addTearDown(bloc.close);
    return bloc;
  }

  /// A bloc that has opened the note.
  Future<EditorBloc> ready() async {
    final bloc = build()..add(const EditorEvent.started('note:fake'));
    await bloc.stream.firstWhere((s) => s.status == EditorStatus.ready);
    return bloc;
  }

  group('EditorBloc started', () {
    test('opens with the note goal and resolved library default', () async {
      document
        ..wordGoal = const GoalSetting.default_()
        ..defaultGoal = 500;
      final bloc = await ready();

      expect(bloc.state.wordGoalSetting, const GoalSetting.default_());
      expect(bloc.state.defaultWordGoal, 500);
      expect(bloc.state.effectiveWordGoal, 500);
    });

    test('a chosen goal updates the document and progress state', () async {
      final bloc = await ready();
      final changed = bloc.stream.firstWhere(
        (state) => state.wordGoalSetting == const GoalSetting.words(1200),
      );

      bloc.add(const EditorEvent.wordGoalChanged(GoalSetting.words(1200)));
      final state = await changed;

      expect(state.effectiveWordGoal, 1200);
      expect(document.wordGoal, const GoalSetting.words(1200));
      expect(
        document.calls,
        contains('setWordGoal GoalSetting.words(field0: 1200)'),
      );
    });

    blocTest<EditorBloc, EditorState>(
      'opens the note and takes its text, properties and save state',
      build: build,
      act: (bloc) => bloc.add(const EditorEvent.started('note:fake')),
      wait: const Duration(milliseconds: 50),
      verify: (bloc) {
        final state = bloc.state;
        expect(state.status, EditorStatus.ready);
        expect(state.noteId, 'note:fake');
        expect(state.document, same(document));
        expect(state.snapshot?.text, 'the text');
        expect(state.snapshot?.title, 'A title');
        expect(state.save, isA<SaveState_Saved>());
        expect(state.properties, const <Property>[_property]);
      },
    );

    blocTest<EditorBloc, EditorState>(
      'a note locked and closed opens to its lock, with no text',
      build: () => build(error: const ApplicationError.locked(message: 'x')),
      act: (bloc) => bloc.add(const EditorEvent.started('note:fake')),
      wait: const Duration(milliseconds: 50),
      verify: (bloc) {
        expect(bloc.state.status, EditorStatus.locked);
        expect(bloc.state.snapshot, isNull);
        expect(bloc.state.document, isNull);
      },
    );

    blocTest<EditorBloc, EditorState>(
      'a note that is not there is not found',
      build: () => build(error: const ApplicationError.notFound(message: 'x')),
      act: (bloc) => bloc.add(const EditorEvent.started('note:fake')),
      wait: const Duration(milliseconds: 50),
      verify: (bloc) => expect(bloc.state.status, EditorStatus.notFound),
    );

    blocTest<EditorBloc, EditorState>(
      'any other error is a failure, with the error',
      build: () => build(error: const ApplicationError.storage(message: 'x')),
      act: (bloc) => bloc.add(const EditorEvent.started('note:fake')),
      wait: const Duration(milliseconds: 50),
      verify: (bloc) {
        expect(bloc.state.status, EditorStatus.failed);
        expect(bloc.state.error, isA<StorageError>());
      },
    );

    blocTest<EditorBloc, EditorState>(
      'letting go of a session whose text could not be read',
      build: build,
      setUp: () =>
          document.failures['snapshot'] = const ApplicationError.storage(
            message: 'x',
          ),
      act: (bloc) => bloc.add(const EditorEvent.started('note:fake')),
      wait: const Duration(milliseconds: 50),
      verify: (bloc) {
        expect(bloc.state.status, EditorStatus.failed);
        expect(document.calls, contains('close'));
      },
    );
  });

  group('EditorBloc signalReceived', () {
    test('the signals are listened to once the note is open', () async {
      await ready();
      expect(document.listening, isTrue);
    });

    test('Saved says the note is saved', () async {
      final bloc = await ready();
      document.events_.add(const core.EditorEvent.saved(rev: 'r9', atMs: 5));
      final state = await bloc.stream.firstWhere(
        (s) => _savedRev(s) == 'r9',
      );
      expect(state.save, const SaveState.saved(rev: 'r9', atMs: 5));
    });

    test(
      'Incoming asks about a change made elsewhere, or a deletion',
      () async {
        final bloc = await ready();
        document.events_.add(const core.EditorEvent.incoming(deleted: false));
        await bloc.stream.firstWhere((s) => s.save is SaveState_Conflict);
        expect(_conflictDeleted(bloc.state), isFalse);
        document.events_.add(const core.EditorEvent.incoming(deleted: true));
        await bloc.stream.firstWhere(
          (s) => _conflictDeleted(s) ?? false,
        );
      },
    );

    test('Reloaded takes the new text and asks the field to show it', () async {
      final bloc = await ready();
      const reloaded = EditorSnapshot(
        id: 'note:fake',
        title: 'New title',
        text: 'from elsewhere',
        rev: 'r2',
        lock: LockState.plain(),
        properties: <Property>[],
        version: 9,
        selection: Range16(start: 0, end: 0),
        history: _noHistory,
        save: SaveState.saved(rev: 'r2', atMs: 1),
        wordGoal: GoalSetting.default_(),
      );
      document.events_.add(const core.EditorEvent.reloaded(snapshot: reloaded));
      final state = await bloc.stream.firstWhere((s) => s.reloadSeq == 1);
      expect(state.snapshot, reloaded);
      expect(state.properties, isEmpty);
      expect(state.completion, isNull);
    });

    test('Failed says saving failed, with why', () async {
      final bloc = await ready();
      document.events_.add(const core.EditorEvent.failed(message: 'disk'));
      final state = await bloc.stream.firstWhere(
        (s) => s.save is SaveState_Failed,
      );
      expect(state.save, const SaveState.failed(message: 'disk'));
    });

    test(
      'Closed keeps nothing of the note: not its text, not its document',
      () async {
        final bloc = await ready();
        document.events_.add(const core.EditorEvent.closed());
        final state = await bloc.stream.firstWhere(
          (s) => s.status == EditorStatus.locked,
        );
        expect(state.snapshot, isNull);
        expect(state.document, isNull);
        expect(state.properties, isEmpty);
        expect(state.save, isNull);
        expect(state.completion, isNull);
        expect(state.noteId, 'note:fake');
      },
    );

    test('nothing is listened to once the editor is closed', () async {
      final bloc = await ready();
      await bloc.close();
      expect(document.listening, isFalse);
    });
  });

  group('EditorBloc edited', () {
    blocTest<EditorBloc, EditorState>(
      'says there is something unsaved',
      build: build,
      act: (bloc) async {
        bloc.add(const EditorEvent.started('note:fake'));
        await Future<void>.delayed(const Duration(milliseconds: 20));
        bloc.add(const EditorEvent.edited());
      },
      verify: (bloc) => expect(bloc.state.save, isA<SaveState_Pending>()),
    );

    test('does not hide the question about a change made elsewhere', () async {
      final bloc = await ready();
      document.events_.add(const core.EditorEvent.incoming(deleted: false));
      await bloc.stream.firstWhere((s) => s.save is SaveState_Conflict);
      bloc.add(const EditorEvent.edited());
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(bloc.state.save, isA<SaveState_Conflict>());
    });
  });

  group('EditorBloc attachRequested', () {
    blocTest<EditorBloc, EditorState>(
      'a picker failure is shown as an attachment problem',
      setUp: () =>
          picker.answer = const Left<ApplicationError, List<ShareFile>>(
            ApplicationError.storage(message: 'No picker.'),
          ),
      build: build,
      act: (bloc) async {
        bloc.add(const EditorEvent.started('note:fake'));
        await bloc.stream.firstWhere((s) => s.status == EditorStatus.ready);
        bloc.add(const EditorEvent.attachRequested(MediaSource.photos));
        await bloc.stream.firstWhere(
          (state) => state.attachProblems.isNotEmpty,
        );
      },
      skip: 2,
      expect: () => <Matcher>[
        isA<EditorState>().having(
          (state) => state.attachProblems,
          'attachProblems',
          const <ApplicationError>[
            ApplicationError.storage(message: 'No picker.'),
          ],
        ),
      ],
      verify: (_) {
        expect(picker.sources, <MediaSource>[MediaSource.photos]);
        expect(document.attachedFiles, isEmpty);
      },
    );

    blocTest<EditorBloc, EditorState>(
      'attaches each file in turn and counts each one',
      setUp: () => picker.answer = Right<ApplicationError, List<ShareFile>>(
        <ShareFile>[_file('one.png'), _file('two.pdf')],
      ),
      build: build,
      act: (bloc) async {
        bloc.add(const EditorEvent.started('note:fake'));
        await bloc.stream.firstWhere((s) => s.status == EditorStatus.ready);
        bloc.add(const EditorEvent.attachRequested(MediaSource.photos));
        await bloc.stream.firstWhere((s) => s.attachSeq == 2);
      },
      wait: const Duration(milliseconds: 50),
      skip: 2,
      expect: () => <Matcher>[
        isA<EditorState>()
            .having((state) => state.attachSeq, 'attachSeq', 1)
            .having((state) => state.attached, 'attached', isNotNull),
        isA<EditorState>()
            .having((state) => state.attachSeq, 'attachSeq', 2)
            .having((state) => state.attached, 'attached', isNotNull),
      ],
      verify: (_) {
        expect(document.attachedFiles, <String>['one.png', 'two.pdf']);
        expect(document.text, contains('one.png'));
        expect(document.text, contains('two.pdf'));
      },
    );

    blocTest<EditorBloc, EditorState>(
      'keeps the one that worked and says why the other did not',
      setUp: () {
        picker.answer = Right<ApplicationError, List<ShareFile>>(
          <ShareFile>[_file('one.png'), _file('two.pdf')],
        );
        document.attachFailures['two.pdf'] = const ApplicationError.storage(
          message: 'Not an image or a PDF.',
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const EditorEvent.started('note:fake'));
        await bloc.stream.firstWhere((s) => s.status == EditorStatus.ready);
        bloc.add(const EditorEvent.attachRequested(MediaSource.photos));
        await bloc.stream.firstWhere((s) => s.attachProblems.isNotEmpty);
      },
      wait: const Duration(milliseconds: 50),
      skip: 2,
      expect: () => <Matcher>[
        isA<EditorState>()
            .having((state) => state.attachSeq, 'attachSeq', 1)
            .having((state) => state.attachProblems, 'problems', isEmpty),
        isA<EditorState>()
            .having((state) => state.attachSeq, 'attachSeq', 1)
            .having(
              (state) => state.attachProblems,
              'problems',
              const <ApplicationError>[
                ApplicationError.storage(
                  message: 'Not an image or a PDF.',
                ),
              ],
            ),
      ],
      verify: (_) {
        expect(document.attachedFiles, <String>['one.png']);
      },
    );

    blocTest<EditorBloc, EditorState>(
      'a locked attachment is shown as a locked problem',
      setUp: () {
        picker.answer = Right<ApplicationError, List<ShareFile>>(
          <ShareFile>[_file('locked.pdf')],
        );
        document.attachFailures['locked.pdf'] = const ApplicationError.locked(
          message: 'Locked.',
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const EditorEvent.started('note:fake'));
        await bloc.stream.firstWhere((s) => s.status == EditorStatus.ready);
        bloc.add(const EditorEvent.attachRequested(MediaSource.pdf));
        await bloc.stream.firstWhere(
          (state) => state.attachProblems.isNotEmpty,
        );
      },
      skip: 2,
      expect: () => <Matcher>[
        isA<EditorState>().having(
          (state) => state.attachProblems,
          'attachProblems',
          const <ApplicationError>[ApplicationError.locked(message: 'Locked.')],
        ),
      ],
      verify: (_) => expect(document.attachedFiles, isEmpty),
    );

    blocTest<EditorBloc, EditorState>(
      'dismissing the notice clears the attachment problems',
      setUp: () =>
          picker.answer = const Left<ApplicationError, List<ShareFile>>(
            ApplicationError.storage(message: 'No picker.'),
          ),
      build: build,
      act: (bloc) async {
        bloc.add(const EditorEvent.started('note:fake'));
        await bloc.stream.firstWhere((s) => s.status == EditorStatus.ready);
        bloc.add(const EditorEvent.attachRequested(MediaSource.photos));
        await bloc.stream.firstWhere((s) => s.attachProblems.isNotEmpty);
        bloc.add(const EditorEvent.attachNoticeDismissed());
      },
      skip: 2,
      expect: () => <Matcher>[
        isA<EditorState>().having(
          (state) => state.attachProblems,
          'problems',
          hasLength(1),
        ),
        isA<EditorState>().having(
          (state) => state.attachProblems,
          'problems',
          isEmpty,
        ),
      ],
    );
  });

  group('EditorBloc caretMoved', () {
    test('keeps what is under the caret', () async {
      final bloc = await ready();
      bloc.add(const EditorEvent.caretMoved(_context));
      final state = await bloc.stream.firstWhere((s) => s.context != null);
      expect(state.context?.inList, isTrue);
      expect(document.calls, isNot(contains('completion')));
    });

    test('asks for completions where there are some', () async {
      document.offered = _completion;
      final bloc = await ready();
      bloc.add(const EditorEvent.caretMoved(_completing));
      final state = await bloc.stream.firstWhere((s) => s.completion != null);
      expect(state.completion, _completion);
    });

    test('takes the completions away where there are none', () async {
      document.offered = _completion;
      final bloc = await ready();
      bloc.add(const EditorEvent.caretMoved(_completing));
      await bloc.stream.firstWhere((s) => s.completion != null);
      bloc.add(const EditorEvent.caretMoved(_context));
      await bloc.stream.firstWhere((s) => s.completion == null);
    });

    test('an answer that a later move overtook is dropped', () async {
      final slow = Completer<Completion?>();
      var asked = 0;
      document.completionBuilder = () {
        asked++;
        return asked == 1 ? slow.future : Future<Completion?>.value();
      };
      final bloc = await ready();
      bloc
        ..add(const EditorEvent.caretMoved(_completing))
        ..add(const EditorEvent.caretMoved(_completing));
      await Future<void>.delayed(const Duration(milliseconds: 30));
      slow.complete(_completion);
      await Future<void>.delayed(const Duration(milliseconds: 30));
      expect(bloc.state.completion, isNull);
    });

    test('a failed completion shows none', () async {
      document.failures['completion'] = const ApplicationError.storage(
        message: 'x',
      );
      final bloc = await ready();
      bloc.add(const EditorEvent.caretMoved(_completing));
      await Future<void>.delayed(const Duration(milliseconds: 30));
      expect(bloc.state.completion, isNull);
      expect(bloc.state.context?.completionHere, isTrue);
    });
  });

  group('EditorBloc completionDismissed and propertiesToggled', () {
    test('dismissing takes the completions away', () async {
      document.offered = _completion;
      final bloc = await ready();
      bloc.add(const EditorEvent.caretMoved(_completing));
      await bloc.stream.firstWhere((s) => s.completion != null);
      bloc.add(const EditorEvent.completionDismissed());
      await bloc.stream.firstWhere((s) => s.completion == null);
    });

    test('the properties open and close', () async {
      final bloc = await ready();
      bloc.add(const EditorEvent.propertiesToggled());
      await bloc.stream.firstWhere((s) => s.propertiesOpen);
      bloc.add(const EditorEvent.propertiesToggled());
      await bloc.stream.firstWhere((s) => !s.propertiesOpen);
    });
  });

  group('EditorBloc propertySet and propertyRemoved', () {
    test(
      'a property set is the properties the core has, and unsaved',
      () async {
        final bloc = await ready();
        document.properties = const <Property>[
          _property,
          Property(
            key: 'rank',
            kind: PropertyKind.number,
            text: '2',
            items: [],
          ),
        ];
        bloc.add(
          const EditorEvent.propertySet(key: 'rank', value: '2'),
        );
        final state = await bloc.stream.firstWhere(
          (s) => s.properties.length == 2,
        );
        expect(state.save, isA<SaveState_Pending>());
        expect(document.calls, contains('setProperty null rank 2'));
      },
    );

    test('a property removed is the properties the core has', () async {
      final bloc = await ready();
      document.properties = const <Property>[];
      bloc.add(const EditorEvent.propertyRemoved(0));
      await bloc.stream.firstWhere((s) => s.properties.isEmpty);
      expect(document.calls, contains('removeProperty 0'));
    });

    test('a property that does not work says so and changes nothing', () async {
      final bloc = await ready();
      document.failures['setProperty'] = const ApplicationError.invalid(
        message: 'A property needs a name.',
      );
      bloc.add(const EditorEvent.propertySet(key: '', value: 'x'));
      final state = await bloc.stream.firstWhere(
        (s) => s.propertyError != null,
      );
      expect(state.properties, const <Property>[_property]);
      expect(state.save, isA<SaveState_Saved>());
    });

    test('does not hide the question about a change made elsewhere', () async {
      final bloc = await ready();
      document.events_.add(const core.EditorEvent.incoming(deleted: false));
      await bloc.stream.firstWhere((s) => s.save is SaveState_Conflict);
      bloc.add(const EditorEvent.propertyRemoved(0));
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(bloc.state.save, isA<SaveState_Conflict>());
    });
  });

  group('EditorBloc keepMineRequested and useTheirsRequested', () {
    test('keeping mine says what the save came to', () async {
      final bloc = await ready();
      document.save = const SaveState.saved(rev: 'r5', atMs: 9);
      bloc.add(const EditorEvent.keepMineRequested());
      await bloc.stream.firstWhere(
        (s) => _savedRev(s) == 'r5',
      );
      expect(document.calls, contains('keepMine'));
    });

    test('keeping mine can fail', () async {
      final bloc = await ready();
      document.failures['keepMine'] = const ApplicationError.storage(
        message: 'disk',
      );
      bloc.add(const EditorEvent.keepMineRequested());
      await bloc.stream.firstWhere((s) => s.save is SaveState_Failed);
    });

    test('using theirs shows their text', () async {
      final bloc = await ready();
      document
        ..text = 'theirs'
        ..version = 7;
      bloc.add(const EditorEvent.useTheirsRequested());
      final state = await bloc.stream.firstWhere((s) => s.reloadSeq == 1);
      expect(state.snapshot?.text, 'theirs');
      expect(state.snapshot?.version, 7);
    });

    test('using theirs for a deleted note lets it go', () async {
      final bloc = await ready();
      document.failures['useTheirs'] = const ApplicationError.notFound(
        message: 'gone',
      );
      bloc.add(const EditorEvent.useTheirsRequested());
      final state = await bloc.stream.firstWhere(
        (s) => s.status == EditorStatus.notFound,
      );
      expect(state.snapshot, isNull);
      expect(state.document, isNull);
      expect(document.listening, isFalse);
    });

    test('using theirs can fail otherwise', () async {
      final bloc = await ready();
      document.failures['useTheirs'] = const ApplicationError.storage(
        message: 'disk',
      );
      bloc.add(const EditorEvent.useTheirsRequested());
      await bloc.stream.firstWhere((s) => s.save is SaveState_Failed);
      expect(bloc.state.status, EditorStatus.ready);
    });
  });

  group('EditorBloc flushRequested and lifecycleChanged', () {
    test('saves now and says what came of it', () async {
      final bloc = await ready();
      bloc.add(const EditorEvent.edited());
      await bloc.stream.firstWhere((s) => s.save is SaveState_Pending);
      document.save = const SaveState.saved(rev: 'r6', atMs: 3);
      bloc.add(const EditorEvent.flushRequested(FlushReason.shortcut));
      await bloc.stream.firstWhere((s) => s.save is SaveState_Saved);
      expect(document.calls, contains('flush'));
    });

    test('leaves a question about a change elsewhere open when saving for '
        'the shortcut or the background', () async {
      final bloc = await ready();
      document.events_.add(const core.EditorEvent.incoming(deleted: false));
      await bloc.stream.firstWhere((s) => s.save is SaveState_Conflict);
      bloc
        ..add(const EditorEvent.flushRequested(FlushReason.shortcut))
        ..add(const EditorEvent.lifecycleChanged(AppLifecycleState.paused));
      await Future<void>.delayed(const Duration(milliseconds: 30));
      expect(document.calls, isNot(contains('flush')));
      expect(bloc.state.save, isA<SaveState_Conflict>());
    });

    test('answers it with Keep mine on the way out', () async {
      final bloc = await ready();
      document.events_.add(const core.EditorEvent.incoming(deleted: false));
      await bloc.stream.firstWhere((s) => s.save is SaveState_Conflict);
      document.save = const SaveState.saved(rev: 'r7', atMs: 3);
      bloc.add(const EditorEvent.flushRequested(FlushReason.preview));
      await bloc.stream.firstWhere((s) => s.save is SaveState_Saved);
      expect(document.calls, contains('flush'));
    });

    test('a failed save is shown, and Retry saves again', () async {
      final bloc = await ready();
      document.failures['flush'] = const ApplicationError.storage(
        message: 'disk',
      );
      bloc.add(const EditorEvent.flushRequested(FlushReason.navigation));
      await bloc.stream.firstWhere((s) => s.save is SaveState_Failed);
      document.failures.clear();
      document.save = const SaveState.saved(rev: 'r8', atMs: 4);
      bloc.add(const EditorEvent.flushRequested(FlushReason.retry));
      await bloc.stream.firstWhere((s) => s.save is SaveState_Saved);
    });

    test('a save for a lock that has closed is the lock', () async {
      final bloc = await ready();
      document.failures['flush'] = const ApplicationError.locked(message: 'x');
      bloc.add(const EditorEvent.flushRequested(FlushReason.lifecycle));
      final state = await bloc.stream.firstWhere(
        (s) => s.status == EditorStatus.locked,
      );
      expect(state.snapshot, isNull);
    });

    test('going to the background saves, coming back does not', () async {
      final bloc = await ready();
      for (final state in <AppLifecycleState>[
        AppLifecycleState.inactive,
        AppLifecycleState.hidden,
        AppLifecycleState.paused,
      ]) {
        document.calls.clear();
        bloc.add(EditorEvent.lifecycleChanged(state));
        await Future<void>.delayed(const Duration(milliseconds: 20));
        expect(document.calls, contains('flush'), reason: '$state');
      }
      document.calls.clear();
      bloc
        ..add(const EditorEvent.lifecycleChanged(AppLifecycleState.resumed))
        ..add(const EditorEvent.lifecycleChanged(AppLifecycleState.detached));
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(document.calls, isNot(contains('flush')));
    });

    test('nothing is saved before the note is open', () async {
      final bloc = build()
        ..add(const EditorEvent.flushRequested(FlushReason.shortcut));
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(document.calls, isNot(contains('flush')));
      expect(bloc.state.status, EditorStatus.opening);
    });
  });

  group('EditorBloc linkOpenRequested and destinationHandled', () {
    test('a link to a note goes there, once what was typed is saved', () async {
      final bloc = await ready();
      bloc.add(
        const EditorEvent.linkOpenRequested(LinkTarget.note(id: 'note:other')),
      );
      final state = await bloc.stream.firstWhere((s) => s.destination != null);
      expect(state.destination, const EditorDestination.note('note:other'));
      expect(document.calls, contains('flush'));
      bloc.add(const EditorEvent.destinationHandled());
      await bloc.stream.firstWhere((s) => s.destination == null);
    });

    test('a link that leads to no note goes to making it, once what was typed '
        'is saved', () async {
      final bloc = await ready();
      bloc.add(
        const EditorEvent.linkOpenRequested(
          LinkTarget.newNote(title: 'Not yet'),
        ),
      );
      final state = await bloc.stream.firstWhere((s) => s.destination != null);
      expect(state.destination, const EditorDestination.newNote('Not yet'));
      expect(document.calls, contains('flush'));
    });

    test(
      'an address is opened where addresses are, and the editor stays',
      () async {
        final bloc = await ready();
        bloc.add(
          const EditorEvent.linkOpenRequested(
            LinkTarget.web(url: 'https://kansoapp.app'),
          ),
        );
        await Future<void>.delayed(const Duration(milliseconds: 20));
        expect(links.opened, <String>['https://kansoapp.app']);
        expect(bloc.state.destination, isNull);
        expect(bloc.state.linkError, isNull);
      },
    );

    test('an address that did not open says so', () async {
      links.answer = const Left<ApplicationError, Unit>(
        ApplicationError.invalid(message: 'no'),
      );
      final bloc = await ready();
      bloc.add(
        const EditorEvent.linkOpenRequested(
          LinkTarget.web(url: 'javascript:alert(1)'),
        ),
      );
      await bloc.stream.firstWhere((s) => s.linkError != null);
    });
  });

  group('EditorBloc closed', () {
    test('saves and lets go of the note', () async {
      final bloc = await ready();
      bloc.add(const EditorEvent.closed());
      await Future<void>.delayed(const Duration(milliseconds: 30));
      expect(document.calls, contains('close'));
      expect(document.listening, isFalse);
      expect(bloc.state.document, isNull);
      expect(bloc.state.snapshot, isNull);
    });

    test('closing the BLoC saves what was typed', () async {
      final bloc = await ready();
      await bloc.close();
      expect(document.calls, contains('close'));
    });

    test('a note whose lock has closed is not closed twice with an error '
        'shown', () async {
      final bloc = await ready();
      document.failures['close'] = const ApplicationError.locked(message: 'x');
      await bloc.close();
      expect(document.calls.where((c) => c == 'close'), hasLength(1));
    });
  });
}
