// Kanso — tabs_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart' show LockState_Plain;
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/models/repos/abs/preferences_repository.dart';
import '../../../core/utils/note_liveness.dart';
import '../../models/data/open_tabs.dart';
import '../../models/data/tab_meta.dart';

part 'tabs_bloc.freezed.dart';
part 'tabs_event.dart';
part 'tabs_state.dart';

/// The notes open as tabs on a tablet (the desktop's `Tabs`), each with its
/// own back and forward.
///
/// The router stays the source of truth for the note showing. A note the
/// router opens (a tap in the list, a link, an address; the navigation
/// history sends it as [TabsEvent.noteOpened]) opens in the showing tab. What
/// the tabs themselves change (choosing, closing or reopening a tab, going
/// back) shows as `pendingNoteId`, and the screen opens that note where the
/// person is and says `consumed`.
///
/// The tabs are kept in the preferences while the window has panes for them,
/// restored at the next start without the notes that have gone or are in the
/// trash, and follow the library: a rename or a new lock updates a tab, and
/// a note trashed or deleted closes its tab and is not kept to reopen.
@injectable
class TabsBloc extends Bloc<TabsEvent, TabsState> {
  TabsBloc(this._library, this._preferences) : super(const TabsState()) {
    // One handler for every event, so each sees the tabs the one before
    // left.
    on<TabsEvent>(
      (event, emit) => switch (event) {
        TabsStartedEvent() => _onStarted(emit),
        TabsEnabledChangedEvent(:final enabled, :final shownNoteId) =>
          _onEnabledChanged(enabled, shownNoteId, emit),
        TabsNoteOpenedEvent(:final noteId) => _onNoteOpened(noteId, emit),
        TabsOpenedInNewTabEvent(:final noteId) => _onOpenedInNewTab(
          noteId,
          emit,
        ),
        TabsActivatedEvent(:final index) => _onActivated(index, emit),
        TabsClosedEvent(:final index) => _change(
          emit,
          (tabs) => tabs.close(index),
        ),
        TabsOthersClosedEvent(:final index) => _change(
          emit,
          (tabs) => tabs.closeOthers(index),
        ),
        TabsRightClosedEvent(:final index) => _change(
          emit,
          (tabs) => tabs.closeRight(index),
        ),
        TabsReorderedEvent(:final from, :final to) => _change(
          emit,
          (tabs) => tabs.reorder(from, to),
        ),
        TabsReopenRequestedEvent() => _onReopen(emit),
        TabsNextRequestedEvent() => _change(emit, (tabs) => tabs.next()),
        TabsPreviousRequestedEvent() => _change(
          emit,
          (tabs) => tabs.previous(),
        ),
        TabsNumberedEvent(:final number) => _change(
          emit,
          (tabs) => tabs.goTo(number),
        ),
        TabsStepRequestedEvent(:final back) => _onStep(back, emit),
        TabsConsumedEvent() => Future<void>.sync(
          () => emit(state.copyWith(pendingNoteId: null)),
        ),
        TabsChangesReceivedEvent(:final ids) => _onChanges(ids, emit),
      },
      transformer: sequential(),
    );
  }

  final LibraryRepository _library;
  final PreferencesRepository _preferences;
  StreamSubscription<List<String>>? _changes;

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }

  Future<void> _onStarted(Emitter<TabsState> emit) async {
    await _changes?.cancel();
    _changes = _library.changes().listen(
      (ids) => add(TabsEvent.changesReceived(ids)),
    );
    final saved = (await _preferences.load()).fold(
      (_) => null,
      (preferences) => preferences,
    );
    if (saved == null) {
      return;
    }
    // The notes that have gone since, and the ones in the trash, are left
    // out.
    final restored = <TabMeta>[];
    for (final id in saved.openTabs) {
      final meta = await _metaOf(id, allowTrashed: false);
      if (meta != null) {
        restored.add(meta);
      }
    }
    final wanted = saved.openTabs.elementAtOrNull(saved.activeTab);
    final shown = wanted == null
        ? 0
        : restored.indexWhere((meta) => meta.id == wanted);
    emit(
      state.copyWith(
        tabs: OpenTabs.restore(restored, shown < 0 ? 0 : shown),
      ),
    );
  }

  Future<void> _onEnabledChanged(
    bool enabled,
    String? shownNoteId,
    Emitter<TabsState> emit,
  ) async {
    emit(state.copyWith(enabled: enabled));
    if (!enabled) {
      return;
    }
    if (shownNoteId != null) {
      await _onNoteOpened(shownNoteId, emit);
    } else if (state.tabs.activeId case final shown?) {
      // The address shows no note but tabs were kept: show the one that
      // was showing.
      emit(state.copyWith(pendingNoteId: shown));
    }
  }

  Future<void> _onNoteOpened(String noteId, Emitter<TabsState> emit) async {
    if (!state.enabled || state.tabs.activeId == noteId) {
      return;
    }
    final meta = await _metaOf(noteId);
    if (meta == null) {
      return;
    }
    _commit(emit, state.tabs.openNote(meta));
  }

  /// Shows the tab at [index]; the note is asked for even when the tab was
  /// showing already, in case the address has gone elsewhere since.
  Future<void> _onActivated(int index, Emitter<TabsState> emit) async {
    if (!state.enabled || state.tabs.tabAt(index) == null) {
      return;
    }
    _commit(emit, state.tabs.activate(index), show: true);
  }

  Future<void> _onOpenedInNewTab(String noteId, Emitter<TabsState> emit) async {
    if (!state.enabled) {
      return;
    }
    final meta = await _metaOf(noteId);
    if (meta == null) {
      return;
    }
    _commit(emit, state.tabs.openInNew(meta), show: true);
  }

  /// Applies [change] to the tabs. The note showing is asked for when the
  /// tab showing now shows another note, or [show] says it must.
  Future<void> _change(
    Emitter<TabsState> emit,
    OpenTabs Function(OpenTabs tabs) change, {
    bool show = false,
  }) async {
    if (!state.enabled) {
      return;
    }
    final before = state.tabs;
    final after = change(before);
    if (identical(before, after)) {
      return;
    }
    _commit(emit, after, show: show || after.activeId != before.activeId);
  }

  Future<void> _onReopen(Emitter<TabsState> emit) async {
    if (!state.enabled || !state.tabs.canReopen) {
      return;
    }
    final live = <String, bool>{
      for (final closed in state.tabs.closed)
        closed.meta.id: await isNoteLive(_library, closed.meta.id),
    };
    final reopened = state.tabs.reopenClosed((id) => live[id] ?? false);
    var tabs = reopened.tabs;
    final meta = reopened.reopened;
    if (meta != null) {
      // The note may have been renamed since the tab was closed.
      final fresh = await _metaOf(meta.id);
      if (fresh != null) {
        tabs = tabs.updateNote(
          fresh.id,
          title: fresh.title,
          locked: fresh.locked,
        );
      }
    }
    _commit(emit, tabs, show: meta != null);
  }

  Future<void> _onStep(bool back, Emitter<TabsState> emit) async {
    final history = state.tabs.history;
    if (!state.enabled || history == null) {
      return;
    }
    final live = await noteLiveness(
      _library,
      back ? history.candidatesBack : history.candidatesForward,
      history.current,
    );
    final stepped = state.tabs.stepActive(
      back: back,
      live: (id) => live[id] ?? false,
    );
    final landed = stepped.landed;
    final meta = landed == null ? null : await _metaOf(landed);
    if (meta == null) {
      // Nowhere to go: the visits stepped over are dropped all the same.
      _commit(emit, stepped.tabs);
      return;
    }
    _commit(emit, stepped.tabs.setActiveMeta(meta), show: true);
  }

  Future<void> _onChanges(List<String> ids, Emitter<TabsState> emit) async {
    final open = state.tabs.ids.toSet();
    var tabs = state.tabs;
    for (final id in ids.toSet()) {
      if (!open.contains(id)) {
        continue;
      }
      final note = await _library.note(id);
      note.fold(
        (error) {
          if (error is NotFoundError) {
            tabs = tabs.removeNote(id);
          }
        },
        (found) {
          tabs = found.notebookId == trashNotebookId
              ? tabs.removeNote(id)
              : tabs.updateNote(
                  id,
                  title: found.title,
                  locked: found.lock is! LockState_Plain,
                );
        },
      );
    }
    if (tabs == state.tabs) {
      return;
    }
    // The tab showing went: the one that takes its place is shown.
    _commit(emit, tabs, show: tabs.activeId != state.tabs.activeId);
  }

  /// Puts [tabs] in the state, asking for the note showing when [show], and
  /// keeps them in the preferences when the row or the tab showing changed.
  void _commit(Emitter<TabsState> emit, OpenTabs tabs, {bool show = false}) {
    final before = state.tabs;
    emit(
      state.copyWith(
        tabs: tabs,
        pendingNoteId: show && tabs.activeId != null
            ? tabs.activeId
            : state.pendingNoteId,
      ),
    );
    final ids = tabs.ids;
    if (state.enabled &&
        (tabs.active != before.active || !_sameIds(ids, before.ids))) {
      unawaited(
        _preferences.update(
          (current) => current.copyWith(openTabs: ids, activeTab: tabs.active),
        ),
      );
    }
  }

  bool _sameIds(List<String> a, List<String> b) {
    if (a.length != b.length) {
      return false;
    }
    for (var at = 0; at < a.length; at++) {
      if (a[at] != b[at]) {
        return false;
      }
    }
    return true;
  }

  /// The tab a note makes, or null when there is no such note (or, unless
  /// [allowTrashed], when it is in the trash).
  Future<TabMeta?> _metaOf(String id, {bool allowTrashed = true}) async {
    final note = await _library.note(id);
    return note.fold((_) => null, (found) {
      if (!allowTrashed && found.notebookId == trashNotebookId) {
        return null;
      }
      return TabMeta(
        id: found.id,
        title: found.title,
        locked: found.lock is! LockState_Plain,
      );
    });
  }
}
