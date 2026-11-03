// Kanso — notebook_home_bloc.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/notebook_home_types.dart';
import '../../../core/bridge/api/people_types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../models/repos/abs/notebook_home_repository.dart';

part 'notebook_home_bloc.freezed.dart';
part 'notebook_home_event.dart';
part 'notebook_home_state.dart';

/// One notebook's homepage: its words and sections, and what the person does
/// to them.
///
/// Everything that reads or changes the page's data (opening, refreshing,
/// keeping, hiding, ticking, laying out) goes through one sequential queue,
/// so two of them never overlap and the page is never built from an answer
/// older than the last one. The first open reads the words first, in the
/// same event, so nothing can open the page before they are known. Only the
/// heavy map is read apart, and it touches nothing but its own field.
@injectable
class NotebookHomeBloc extends Bloc<NotebookHomeEvent, NotebookHomeState> {
  NotebookHomeBloc(this._home) : super(const NotebookHomeState()) {
    // One queue for everything that reads or changes the page: handlers
    // registered one by one would each have a queue of their own, and
    // events of different kinds would run side by side.
    on<NotebookHomeEvent>(
      _onData,
      transformer: (events, mapper) => sequential<NotebookHomeEvent>().call(
        events.where((e) => e is! NotebookHomeMapRequestedEvent),
        mapper,
      ),
    );
    // The map only fills its own field, and takes as long as it takes.
    on<NotebookHomeMapRequestedEvent>(_onMap, transformer: restartable());
  }

  Future<void> _onData(
    NotebookHomeEvent event,
    Emitter<NotebookHomeState> emit,
  ) async {
    switch (event) {
      case NotebookHomeStartedEvent():
        await _onStarted(event, emit);
      case NotebookHomeReopenedEvent():
        await _open(emit, map: true);
      case NotebookHomeRefreshAllRequestedEvent():
        await _onRefreshAll(emit);
      case NotebookHomeSectionRefreshRequestedEvent(:final key):
        await _refresh(key, emit);
      case NotebookHomePinnedEvent(:final key, :final on):
        _take(emit, await _home.pin(state.bookId, key, on: on));
      case NotebookHomeHiddenEvent(:final key, :final hidden):
        _take(emit, await _home.hide(state.bookId, key, hidden: hidden));
      case NotebookHomeHiddenShownEvent():
        _take(emit, await _home.showHidden(state.bookId));
      case NotebookHomeNotRightEvent(:final key, :final index):
        _take(emit, await _home.notRight(state.bookId, key, index));
      case NotebookHomeCopiedIntoHomeEvent(:final key):
        await _onCopiedIntoHome(key, emit);
      case NotebookHomeTaskTickedEvent(:final task, :final checked):
        await _home.setTask(task, checked: checked);
        if (!emit.isDone) {
          await _open(emit);
        }
      case NotebookHomeAllShownEvent(:final key):
        _take(emit, await _home.showAll(state.bookId, key));
      case NotebookHomeCustomisedEvent(:final order, :final hidden):
        _take(emit, await _home.customise(state.bookId, order, hidden));
      case NotebookHomeLayoutResetEvent():
        _take(emit, await _home.resetLayout(state.bookId));
      case NotebookHomeHomeChosenEvent(:final noteId):
        await _home.chooseHome(state.bookId, noteId);
        if (!emit.isDone) {
          await _open(emit);
        }
      case NotebookHomeNewHomeRequestedEvent():
        await _onNewHome(emit);
      case NotebookHomeNoticeTakenEvent():
        emit(state.copyWith(copied: false, madeNoteId: null));
      case NotebookHomeMapRequestedEvent():
        break;
    }
  }

  final NotebookHomeRepository _home;

  Future<void> _onStarted(
    NotebookHomeStartedEvent event,
    Emitter<NotebookHomeState> emit,
  ) async {
    emit(NotebookHomeState(bookId: event.bookId));
    final words = (await _home.words()).toOption().toNullable();
    if (emit.isDone) {
      return;
    }
    emit(state.copyWith(words: words));
    await _open(emit, first: true, map: true);
  }

  Future<void> _open(
    Emitter<NotebookHomeState> emit, {
    bool first = false,
    bool map = false,
  }) async {
    final result = await _home.open(state.bookId);
    if (emit.isDone) {
      return;
    }
    result.fold((_) => emit(state.copyWith(status: LoadStatus.failed)), (
      panel,
    ) {
      emit(
        state.copyWith(
          status: panel == null ? LoadStatus.failed : LoadStatus.ready,
          panel: panel,
        ),
      );
      if (panel == null || panel.locked) {
        return;
      }
      if (first) {
        // The sections the person's refresh setting wants rewritten now, one
        // by one so a failure names its own section. Asked once.
        for (final key in panel.due) {
          add(NotebookHomeEvent.sectionRefreshRequested(key));
        }
      }
      // After the sections are drawn, not before.
      if (map) {
        add(const NotebookHomeEvent.mapRequested());
      }
    });
  }

  Future<void> _onMap(
    NotebookHomeMapRequestedEvent event,
    Emitter<NotebookHomeState> emit,
  ) async {
    final map = (await _home.map(state.bookId)).toOption().toNullable();
    if (!emit.isDone && map != null) {
      emit(state.copyWith(map: map));
    }
  }

  Future<void> _refresh(String key, Emitter<NotebookHomeState> emit) async {
    emit(
      state.copyWith(
        working: <String>{...state.working, key},
        trouble: <String, String>{...state.trouble}..remove(key),
      ),
    );
    final done = await _home.refreshSection(state.bookId, key);
    if (emit.isDone) {
      return;
    }
    final working = <String>{...state.working}..remove(key);
    done.fold(
      (error) => emit(
        state.copyWith(
          working: working,
          trouble: <String, String>{...state.trouble, key: error.message},
        ),
      ),
      (panel) =>
          emit(state.copyWith(working: working, panel: panel ?? state.panel)),
    );
  }

  Future<void> _onRefreshAll(Emitter<NotebookHomeState> emit) async {
    final sections = state.panel?.sections ?? const <PageSection>[];
    for (final s in sections.where((s) => s.ai)) {
      if (s.state != PageSectionState.pinned) {
        await _refresh(s.key, emit);
      }
    }
  }

  void _take(
    Emitter<NotebookHomeState> emit,
    Either<ApplicationError, HomePanel?> result,
  ) {
    final panel = result.toOption().toNullable();
    if (!emit.isDone && panel != null) {
      emit(state.copyWith(panel: panel));
    }
  }

  Future<void> _onCopiedIntoHome(
    String key,
    Emitter<NotebookHomeState> emit,
  ) async {
    final done = await _home.copyIntoHome(state.bookId, key);
    if (emit.isDone || done.isLeft()) {
      return;
    }
    emit(state.copyWith(copied: true));
    await _open(emit);
  }

  Future<void> _onNewHome(Emitter<NotebookHomeState> emit) async {
    final made = (await _home.newHome(state.bookId)).toOption().toNullable();
    if (emit.isDone || made == null) {
      return;
    }
    emit(state.copyWith(madeNoteId: made));
  }
}
