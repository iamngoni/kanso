// Kanso — tabs_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/state/navigation_history/navigation_history_bloc.dart';
import 'package:kanso/tabs/state/tabs/tabs_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/library_fixtures.dart';
import '../../helpers/mock_repositories.dart';
import '../../helpers/seeded_appearance_bloc.dart';

void main() {
  late MockLibraryRepository library;
  late SeededPreferencesRepository preferences;
  late StreamController<List<String>> changes;
  final notes = <String, NoteView>{};

  NoteView plainNote(String id, String title, {bool locked = false}) =>
      NoteView(
        id: id,
        rev: '1',
        title: title,
        lock: locked ? const LockState.closed() : const LockState.plain(),
        notebookId: 'b_1',
        tags: const <String>[],
        status: 'none',
        pinned: false,
        createdMs: 0,
        updatedMs: 0,
      );

  NoteView lockedNote(String id) => plainNote(id, 'Note $id', locked: true);

  setUp(() {
    notes
      ..clear()
      ..addEntries(
        <String>['a', 'b', 'c', 'd'].map((id) => MapEntry(id, noteView(id))),
      );
    changes = StreamController<List<String>>.broadcast();
    addTearDown(changes.close);
    preferences = SeededPreferencesRepository(const Preferences());
    library = MockLibraryRepository();
    when(() => library.changes()).thenAnswer((_) => changes.stream);
    when(() => library.note(any())).thenAnswer((call) async {
      final note = notes[call.positionalArguments.first as String];
      return note == null
          ? const Left<ApplicationError, NoteView>(
              ApplicationError.notFound(message: 'Not found.'),
            )
          : Right<ApplicationError, NoteView>(note);
    });
  });

  Future<TabsBloc> start({bool enabled = true, String? shown}) async {
    final bloc = TabsBloc(library, preferences);
    addTearDown(bloc.close);
    bloc
      ..add(const TabsEvent.started())
      ..add(TabsEvent.enabledChanged(enabled: enabled, shownNoteId: shown));
    await pumpEventQueue();
    return bloc;
  }

  Future<void> send(TabsBloc bloc, TabsEvent event) async {
    bloc.add(event);
    await pumpEventQueue();
  }

  List<String> ids(TabsBloc bloc) => bloc.state.tabs.ids;

  Future<TabsBloc> three() async {
    final bloc = await start(shown: 'a');
    await send(bloc, const TabsEvent.openedInNewTab('b'));
    await send(bloc, const TabsEvent.openedInNewTab('c'));
    await send(bloc, const TabsEvent.consumed());
    return bloc;
  }

  group('TabsBloc', () {
    test('starts with no tabs, off', () {
      final bloc = TabsBloc(library, preferences);
      addTearDown(bloc.close);
      expect(bloc.state.tabs.isEmpty, isTrue);
      expect(bloc.state.enabled, isFalse);
      expect(bloc.state.barVisible, isFalse);
    });

    test('while off, nothing opens and nothing is kept', () async {
      final bloc = await start(enabled: false);
      await send(bloc, const TabsEvent.noteOpened('a'));
      await send(bloc, const TabsEvent.openedInNewTab('b'));
      expect(bloc.state.tabs.isEmpty, isTrue);
      expect(preferences.current.openTabs, isEmpty);
    });

    test('the first note opens a tab, the next takes its place', () async {
      final bloc = await start(shown: 'a');
      expect(ids(bloc), <String>['a']);
      await send(bloc, const TabsEvent.noteOpened('b'));
      expect(ids(bloc), <String>['b']);
      expect(bloc.state.tabs.history?.visits.$1, <String>['a', 'b']);
      expect(bloc.state.pendingNoteId, isNull, reason: 'the router is there');
    });

    test('a note open in another tab shows that tab', () async {
      final bloc = await three();
      await send(bloc, const TabsEvent.noteOpened('a'));
      expect(ids(bloc), <String>['a', 'b', 'c']);
      expect(bloc.state.tabs.activeId, 'a');
      expect(bloc.state.pendingNoteId, isNull);
    });

    test('a note that does not exist opens nothing', () async {
      final bloc = await start(shown: 'a');
      await send(bloc, const TabsEvent.openedInNewTab('gone'));
      await send(bloc, const TabsEvent.noteOpened('gone'));
      expect(ids(bloc), <String>['a']);
    });

    test('a new tab goes after the showing one and is asked for', () async {
      final bloc = await start(shown: 'a');
      await send(bloc, const TabsEvent.openedInNewTab('b'));
      expect(ids(bloc), <String>['a', 'b']);
      expect(bloc.state.tabs.activeId, 'b');
      expect(bloc.state.pendingNoteId, 'b');
      expect(bloc.state.barVisible, isTrue);
      await send(bloc, const TabsEvent.consumed());
      expect(bloc.state.pendingNoteId, isNull);
    });

    test('a tab chosen is asked for, the showing one too', () async {
      final bloc = await three();
      await send(bloc, const TabsEvent.activated(0));
      expect(bloc.state.tabs.activeId, 'a');
      expect(bloc.state.pendingNoteId, 'a');
      await send(bloc, const TabsEvent.consumed());
      await send(bloc, const TabsEvent.activated(0));
      expect(bloc.state.pendingNoteId, 'a');
      await send(bloc, const TabsEvent.consumed());
      await send(bloc, const TabsEvent.activated(9));
      expect(bloc.state.pendingNoteId, isNull);
    });

    test('closing the showing tab shows its neighbour', () async {
      final bloc = await three();
      await send(bloc, const TabsEvent.closed(2));
      expect(ids(bloc), <String>['a', 'b']);
      expect(bloc.state.pendingNoteId, 'b');
      await send(bloc, const TabsEvent.consumed());
      await send(bloc, const TabsEvent.closed(0));
      expect(ids(bloc), <String>['b']);
      expect(bloc.state.pendingNoteId, isNull, reason: 'b stays showing');
      await send(bloc, const TabsEvent.closed(0));
      expect(ids(bloc), <String>['b'], reason: 'never the only one');
    });

    test('close others and close right', () async {
      final bloc = await three();
      await send(bloc, const TabsEvent.rightClosed(0));
      expect(ids(bloc), <String>['a']);
      expect(bloc.state.pendingNoteId, 'a');
      await send(bloc, const TabsEvent.consumed());
      await send(bloc, const TabsEvent.openedInNewTab('b'));
      await send(bloc, const TabsEvent.openedInNewTab('c'));
      await send(bloc, const TabsEvent.othersClosed(1));
      expect(ids(bloc), <String>['b']);
      expect(bloc.state.tabs.canReopen, isTrue);
    });

    test('reopening brings the last closed tab back', () async {
      final bloc = await three();
      await send(bloc, const TabsEvent.closed(1));
      expect(ids(bloc), <String>['a', 'c']);
      await send(bloc, const TabsEvent.consumed());
      await send(bloc, const TabsEvent.reopenRequested());
      expect(ids(bloc), <String>['a', 'b', 'c']);
      expect(bloc.state.tabs.activeId, 'b');
      expect(bloc.state.pendingNoteId, 'b');
    });

    test(
      'a closed tab whose note was trashed or deleted stays closed',
      () async {
        final bloc = await three();
        await send(bloc, const TabsEvent.closed(2));
        await send(bloc, const TabsEvent.closed(1));
        notes['a'] = noteView('a');
        notes['b'] = noteView('b', notebookId: 'trash');
        notes.remove('c');
        await send(bloc, const TabsEvent.reopenRequested());
        expect(ids(bloc), <String>['a']);
        expect(bloc.state.tabs.canReopen, isFalse);
      },
    );

    test('a reopened tab shows the note as it is now', () async {
      final bloc = await three();
      await send(bloc, const TabsEvent.closed(1));
      notes['b'] = lockedNote('b');
      await send(bloc, const TabsEvent.reopenRequested());
      final reopened = bloc.state.tabs.tabAt(1)!.meta;
      expect(reopened.locked, isTrue);
    });

    test('next, previous and a number wrap and ask for the note', () async {
      final bloc = await three();
      await send(bloc, const TabsEvent.nextRequested());
      expect(bloc.state.tabs.activeId, 'a');
      expect(bloc.state.pendingNoteId, 'a');
      await send(bloc, const TabsEvent.previousRequested());
      expect(bloc.state.tabs.activeId, 'c');
      await send(bloc, const TabsEvent.numbered(2));
      expect(bloc.state.tabs.activeId, 'b');
      await send(bloc, const TabsEvent.numbered(9));
      expect(bloc.state.tabs.activeId, 'c');
    });

    test('a reorder keeps the showing note', () async {
      final bloc = await three();
      await send(bloc, const TabsEvent.reordered(2, 0));
      expect(ids(bloc), <String>['c', 'a', 'b']);
      expect(bloc.state.tabs.activeId, 'c');
      expect(bloc.state.pendingNoteId, isNull);
    });

    test('back and forward step the showing tab alone', () async {
      final bloc = await start(shown: 'a');
      await send(bloc, const TabsEvent.noteOpened('b'));
      await send(bloc, const TabsEvent.openedInNewTab('c'));
      await send(bloc, const TabsEvent.noteOpened('d'));
      await send(bloc, const TabsEvent.consumed());
      expect(ids(bloc), <String>['b', 'd']);
      await send(bloc, const TabsEvent.stepRequested(back: true));
      expect(ids(bloc), <String>['b', 'c']);
      expect(bloc.state.tabs.active, 1);
      expect(bloc.state.pendingNoteId, 'c');
      await send(bloc, const TabsEvent.consumed());
      await send(bloc, const TabsEvent.stepRequested(back: true));
      expect(ids(bloc), <String>['b', 'c'], reason: 'c has nothing behind it');
      expect(bloc.state.pendingNoteId, isNull);
      await send(bloc, const TabsEvent.stepRequested(back: false));
      expect(ids(bloc), <String>['b', 'd']);
      await send(bloc, const TabsEvent.activated(0));
      await send(bloc, const TabsEvent.stepRequested(back: true));
      expect(ids(bloc), <String>['a', 'd']);
      expect(bloc.state.tabs.active, 0);
    });

    test('going back skips a note since trashed', () async {
      final bloc = await start(shown: 'a');
      await send(bloc, const TabsEvent.noteOpened('b'));
      await send(bloc, const TabsEvent.noteOpened('c'));
      notes['b'] = noteView('b', notebookId: 'trash');
      await send(bloc, const TabsEvent.stepRequested(back: true));
      expect(ids(bloc), <String>['a']);
      expect(bloc.state.pendingNoteId, 'a');
    });

    test('the note stepped to may be open in another tab', () async {
      final bloc = await start(shown: 'a');
      await send(bloc, const TabsEvent.noteOpened('b'));
      await send(bloc, const TabsEvent.openedInNewTab('a'));
      expect(ids(bloc), <String>['b', 'a']);
      await send(bloc, const TabsEvent.activated(0));
      await send(bloc, const TabsEvent.stepRequested(back: true));
      expect(ids(bloc), <String>['a', 'a']);
      expect(bloc.state.tabs.active, 0);
    });

    test('a rename and a new lock update the tab', () async {
      final bloc = await three();
      notes['a'] = lockedNote('a');
      changes.add(<String>['a', 'zzz']);
      await pumpEventQueue();
      final meta = bloc.state.tabs.tabAt(0)!.meta;
      expect(meta.locked, isTrue);
      expect(meta.title, 'Note a');
      notes['b'] = plainNote('b', 'Renamed');
      changes.add(<String>['b']);
      await pumpEventQueue();
      expect(bloc.state.tabs.tabAt(1)!.meta.title, 'Renamed');
      expect(bloc.state.pendingNoteId, isNull);
    });

    test(
      'a note trashed closes its tab, and the showing one is replaced',
      () async {
        final bloc = await three();
        notes['b'] = noteView('b', notebookId: 'trash');
        changes.add(<String>['b']);
        await pumpEventQueue();
        expect(ids(bloc), <String>['a', 'c']);
        expect(bloc.state.pendingNoteId, isNull, reason: 'c was showing');
        notes['c'] = noteView('c', notebookId: 'trash');
        changes.add(<String>['c']);
        await pumpEventQueue();
        expect(ids(bloc), <String>['a']);
        expect(bloc.state.pendingNoteId, 'a');
        expect(
          bloc.state.tabs.canReopen,
          isFalse,
          reason: 'not kept to reopen',
        );
      },
    );

    test('a note deleted closes its tab', () async {
      final bloc = await three();
      notes.remove('a');
      changes.add(<String>['a']);
      await pumpEventQueue();
      expect(ids(bloc), <String>['b', 'c']);
      expect(bloc.state.tabs.activeId, 'c');
    });

    test('a failed read keeps the tab', () async {
      final bloc = await three();
      when(() => library.note('a')).thenAnswer(
        (_) async => const Left<ApplicationError, NoteView>(
          ApplicationError.storage(message: 'Disk.'),
        ),
      );
      changes.add(<String>['a']);
      await pumpEventQueue();
      expect(ids(bloc), <String>['a', 'b', 'c']);
    });

    test('the tabs are kept in the preferences', () async {
      final bloc = await three();
      expect(preferences.current.openTabs, <String>['a', 'b', 'c']);
      expect(preferences.current.activeTab, 2);
      await send(bloc, const TabsEvent.activated(0));
      expect(preferences.current.activeTab, 0);
      await send(bloc, const TabsEvent.closed(1));
      expect(preferences.current.openTabs, <String>['a', 'c']);
    });

    test('they are restored at the next start, without lost notes', () async {
      preferences = SeededPreferencesRepository(
        const Preferences(
          openTabs: <String>['a', 'gone', 'b', 'c', 'd'],
          activeTab: 3,
        ),
      );
      notes['c'] = noteView('c', notebookId: 'trash');
      final bloc = await start();
      expect(ids(bloc), <String>['a', 'b', 'd']);
      expect(bloc.state.tabs.activeId, 'a', reason: 'c was showing; gone');
      expect(bloc.state.pendingNoteId, 'a', reason: 'no note was showing');
    });

    test('the showing tab is the one that was showing', () async {
      preferences = SeededPreferencesRepository(
        const Preferences(openTabs: <String>['a', 'gone', 'b'], activeTab: 2),
      );
      final bloc = await start();
      expect(bloc.state.tabs.activeId, 'b');
    });

    test('a note the address shows at launch opens or shows its tab', () async {
      preferences = SeededPreferencesRepository(
        const Preferences(openTabs: <String>['a', 'b']),
      );
      final bloc = await start(shown: 'b');
      expect(ids(bloc), <String>['a', 'b']);
      expect(bloc.state.tabs.activeId, 'b');
      expect(bloc.state.pendingNoteId, isNull);
    });

    test('turning the tabs off keeps them, and none is kept after', () async {
      final bloc = await three();
      await send(bloc, const TabsEvent.enabledChanged(enabled: false));
      await send(bloc, const TabsEvent.noteOpened('d'));
      expect(ids(bloc), <String>['a', 'b', 'c']);
      expect(bloc.state.barVisible, isFalse);
      await send(
        bloc,
        const TabsEvent.enabledChanged(enabled: true, shownNoteId: 'd'),
      );
      expect(ids(bloc), <String>['a', 'b', 'd'], reason: 'd took the tab');
    });
  });

  group('NavigationHistoryBloc with tabs', () {
    late TabsBloc tabs;
    late NavigationHistoryBloc history;

    Future<void> settle() => pumpEventQueue();

    setUp(() async {
      tabs = TabsBloc(library, preferences);
      history = NavigationHistoryBloc(library, tabs);
      addTearDown(tabs.close);
      addTearDown(history.close);
    });

    test("while the tabs are off the history is the app's own", () async {
      history
        ..add(const NavigationHistoryEvent.visited('a'))
        ..add(const NavigationHistoryEvent.visited('b'));
      await settle();
      expect(history.state.history.visits.$1, <String>['a', 'b']);
      expect(history.state.canGoBack, isTrue);
      expect(tabs.state.tabs.isEmpty, isTrue);
    });

    test('with tabs, a visit opens the note in the showing tab', () async {
      tabs.add(const TabsEvent.enabledChanged(enabled: true));
      await settle();
      history
        ..add(const NavigationHistoryEvent.visited('a'))
        ..add(const NavigationHistoryEvent.visited('b'));
      await settle();
      expect(tabs.state.tabs.ids, <String>['b']);
      expect(history.state.history.visits.$1, <String>['a', 'b']);
      expect(history.state.canGoBack, isTrue);
      expect(history.state.canGoForward, isFalse);
    });

    test('back steps the showing tab, each tab has its own history', () async {
      tabs.add(const TabsEvent.enabledChanged(enabled: true));
      await settle();
      history
        ..add(const NavigationHistoryEvent.visited('a'))
        ..add(const NavigationHistoryEvent.visited('b'));
      await settle();
      tabs.add(const TabsEvent.openedInNewTab('c'));
      await settle();
      expect(history.state.canGoBack, isFalse, reason: 'c is new');
      history.add(const NavigationHistoryEvent.backRequested());
      await settle();
      expect(tabs.state.tabs.ids, <String>['b', 'c']);
      tabs.add(const TabsEvent.activated(0));
      await settle();
      expect(history.state.canGoBack, isTrue);
      history.add(const NavigationHistoryEvent.backRequested());
      await settle();
      expect(tabs.state.tabs.ids, <String>['a', 'c']);
      expect(tabs.state.pendingNoteId, 'a');
      expect(history.state.canGoForward, isTrue);
      expect(history.state.pendingNoteId, isNull);
      history.add(const NavigationHistoryEvent.forwardRequested());
      await settle();
      expect(tabs.state.tabs.ids, <String>['b', 'c']);
    });

    test("turning the tabs off returns the app's own history", () async {
      history.add(const NavigationHistoryEvent.visited('a'));
      await settle();
      tabs.add(const TabsEvent.enabledChanged(enabled: true));
      await settle();
      history.add(const NavigationHistoryEvent.visited('b'));
      await settle();
      expect(history.state.history.visits.$1, <String>['b']);
      tabs.add(const TabsEvent.enabledChanged(enabled: false));
      await settle();
      expect(history.state.history.visits.$1, <String>['a']);
    });
  });
}
