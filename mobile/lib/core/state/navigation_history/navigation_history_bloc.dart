// Kanso — navigation_history_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../tabs/state/tabs/tabs_bloc.dart';
import '../../models/data/navigation_history.dart';
import '../../models/repos/abs/library_repository.dart';
import '../../utils/note_liveness.dart';

part 'navigation_history_bloc.freezed.dart';
part 'navigation_history_event.dart';
part 'navigation_history_state.dart';

/// Back and forward between the notes opened, as in a browser (the desktop's
/// `NavHistory`).
///
/// A screen records each note it opens as a visit and, when `pendingNoteId`
/// appears in the state, opens that note and says `consumed`. A note that is
/// gone (deleted, or in the trash) is skipped, and asked of the library.
///
/// With [_tabs] (the tablets' layouts, where the notes are open in tabs) the
/// history is the showing tab's own: a visit opens the note in that tab, going
/// back and forward step that tab alone (the tabs' BLoC does the stepping and
/// opens the note), and this BLoC mirrors the tab's history and whether it
/// can go either way, so the screens read them from here as they do on a
/// phone. While the tabs are off (a phone, or a window with one pane) the
/// history is this BLoC's own, one for the app, as it always was.
@injectable
class NavigationHistoryBloc
    extends Bloc<NavigationHistoryEvent, NavigationHistoryState> {
  NavigationHistoryBloc(this._library, @factoryParam this._tabs)
    : super(const NavigationHistoryState()) {
    // One handler for every event, so each sees the history the one before
    // left.
    on<NavigationHistoryEvent>(
      (event, emit) => switch (event) {
        NavigationHistoryVisitedEvent(:final noteId) => _onVisited(
          noteId,
          emit,
        ),
        NavigationHistoryBackRequestedEvent() => _onStep(back: true, emit),
        NavigationHistoryForwardRequestedEvent() => _onStep(back: false, emit),
        NavigationHistoryConsumedEvent() => Future<void>.sync(
          () => emit(state.copyWith(pendingNoteId: null)),
        ),
        NavigationHistoryTabsChangedEvent() => _onTabsChanged(emit),
      },
      transformer: sequential(),
    );
    _tabsChanges = _tabs?.stream
        .map((tabs) => (tabs.enabled, tabs.tabs.history))
        .distinct()
        .listen((_) => add(const NavigationHistoryEvent.tabsChanged()));
  }

  final LibraryRepository _library;
  final TabsBloc? _tabs;
  StreamSubscription<(bool, NavigationHistory?)>? _tabsChanges;

  /// The history of the app while the tabs are off.
  NavigationHistory _own = const NavigationHistory();

  bool get _inTabs => _tabs?.state.enabled ?? false;

  @override
  Future<void> close() async {
    await _tabsChanges?.cancel();
    return super.close();
  }

  /// The history is the showing tab's, or the app's own again.
  Future<void> _onTabsChanged(Emitter<NavigationHistoryState> emit) async {
    final history = _inTabs
        ? _tabs!.state.tabs.history ?? const NavigationHistory()
        : _own;
    emit(await _withReach(state.copyWith(history: history)));
  }

  Future<void> _onVisited(
    String noteId,
    Emitter<NavigationHistoryState> emit,
  ) async {
    if (_inTabs) {
      _tabs!.add(TabsEvent.noteOpened(noteId));
      return;
    }
    final history = _own.visit(noteId);
    if (history == _own) {
      return;
    }
    _own = history;
    emit(await _withReach(state.copyWith(history: history)));
  }

  Future<void> _onStep(
    Emitter<NavigationHistoryState> emit, {
    required bool back,
  }) async {
    if (_inTabs) {
      _tabs!.add(TabsEvent.stepRequested(back: back));
      return;
    }
    final history = _own;
    final live = await noteLiveness(
      _library,
      back ? history.candidatesBack : history.candidatesForward,
      history.current,
    );
    bool isLive(String id) => live[id] ?? false;
    final stepped = back ? history.goBack(isLive) : history.goForward(isLive);
    _own = stepped.history;
    emit(
      await _withReach(
        state.copyWith(
          history: stepped.history,
          pendingNoteId: stepped.landed ?? state.pendingNoteId,
        ),
      ),
    );
  }

  /// [next] with whether going back and forward would land anywhere.
  Future<NavigationHistoryState> _withReach(NavigationHistoryState next) async {
    final history = next.history;
    final backward = await noteLiveness(
      _library,
      history.candidatesBack,
      history.current,
    );
    final forward = await noteLiveness(
      _library,
      history.candidatesForward,
      history.current,
    );
    return next.copyWith(
      canGoBack: history.canGoBack((id) => backward[id] ?? false),
      canGoForward: history.canGoForward((id) => forward[id] ?? false),
    );
  }
}
