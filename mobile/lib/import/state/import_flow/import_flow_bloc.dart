// Kanso: import_flow_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../models/data/import_pick.dart';
import '../../models/data/import_problem.dart';
import '../../models/data/import_stage.dart';
import '../../models/repos/abs/import_files_repository.dart';
import '../../models/repos/abs/import_repository.dart';

part 'import_flow_bloc.freezed.dart';
part 'import_flow_event.dart';
part 'import_flow_state.dart';

/// Importing notes: choose a source, choose its file or folder, read it to
/// say what it holds, and, once the person agrees, write the notes.
///
/// Reading writes nothing; the import is the same work with the writing
/// switched on, so the summary is what the import will do. Both run in the
/// library on a thread of their own and report as a stream, so leaving the
/// flow stops listening but does not undo what was already written
/// (importing again updates the notes it made and never undoes a change
/// made in Kanso). The copies the pickers made in the cache are removed when
/// the flow starts over or closes.
@injectable
class ImportFlowBloc extends Bloc<ImportFlowEvent, ImportFlowState> {
  ImportFlowBloc(this._import, this._files) : super(const ImportFlowState()) {
    on<ImportFlowStartedEvent>(_onStarted, transformer: droppable());
    on<ImportFlowSourceChosenEvent>(_onSourceChosen, transformer: droppable());
    on<ImportFlowConfirmedEvent>(_onConfirmed, transformer: droppable());
    on<ImportFlowRetriedEvent>(_onRetried, transformer: droppable());
    on<ImportFlowRestartedEvent>(_onRestarted, transformer: restartable());
  }

  final ImportRepository _import;
  final ImportFilesRepository _files;

  Future<void> _onStarted(
    ImportFlowStartedEvent event,
    Emitter<ImportFlowState> emit,
  ) async {
    final sources = _import.sources();
    final start = state.copyWith(sources: sources);
    emit(start);
    final path = event.path;
    if (path == null || path.isEmpty) {
      return;
    }
    final kinds = _import.kindsFor(path);
    final known = <String>{for (final source in sources) source.kind};
    final failure = kinds.fold<ApplicationError?>(
      (error) => error,
      (_) => null,
    );
    final found = kinds.fold<List<String>>(
      (_) => <String>[],
      (list) => <String>[
        for (final kind in list)
          if (known.contains(kind)) kind,
      ],
    );
    final pick = ImportPick(path: path, name: _nameOf(path));
    final named = event.kind;
    if (failure != null) {
      emit(start.copyWith(problem: _problemOf(failure)));
    } else if (named != null && known.contains(named)) {
      await _read(
        emit,
        start.copyWith(pick: pick, only: <String>[named]),
        named,
      );
    } else if (found.length == 1) {
      await _read(emit, start.copyWith(pick: pick, only: found), found.single);
    } else if (found.isEmpty) {
      emit(start.copyWith(problem: const ImportProblem.notAnExport()));
    } else {
      emit(start.copyWith(pick: pick, only: found));
    }
  }

  Future<void> _onSourceChosen(
    ImportFlowSourceChosenEvent event,
    Emitter<ImportFlowState> emit,
  ) async {
    final source = _sourceOf(event.kind);
    if (source == null) {
      return;
    }
    final given = state.pick;
    if (given != null && !state.picked) {
      await _read(emit, state, source.kind);
      return;
    }
    emit(state.copyWith(picking: true, problem: null));
    final chosen = event.folder
        ? await _files.pickFolder()
        : await _files.pickFile(source.extensions);
    await chosen.fold(
      (error) async =>
          emit(state.copyWith(picking: false, problem: _problemOf(error))),
      (pick) async {
        if (pick == null) {
          emit(state.copyWith(picking: false));
          return;
        }
        await _read(
          emit,
          state.copyWith(picking: false, picked: true, pick: pick),
          source.kind,
        );
      },
    );
  }

  Future<void> _onConfirmed(
    ImportFlowConfirmedEvent event,
    Emitter<ImportFlowState> emit,
  ) async {
    final source = state.source;
    final pick = state.pick;
    if (state.stage != ImportStage.summary || source == null || pick == null) {
      return;
    }
    emit(
      state.copyWith(
        stage: ImportStage.importing,
        progress: null,
        problem: null,
      ),
    );
    await _follow(emit, _import.run(source.kind, pick.path), ImportStage.done);
  }

  Future<void> _onRetried(
    ImportFlowRetriedEvent event,
    Emitter<ImportFlowState> emit,
  ) async {
    final source = state.source;
    if (state.stage != ImportStage.failed ||
        source == null ||
        state.pick == null) {
      emit(state.copyWith(stage: ImportStage.sources, problem: null));
      return;
    }
    await _read(emit, state, source.kind);
  }

  Future<void> _onRestarted(
    ImportFlowRestartedEvent event,
    Emitter<ImportFlowState> emit,
  ) async {
    await _forgetPicked();
    final fresh = ImportFlowState(sources: state.sources);
    // A file somebody handed over stays the subject: the person picks which
    // kind of export it is, not the file again.
    final given = state.pick;
    emit(
      given != null && !state.picked
          ? fresh.copyWith(pick: given, only: state.only)
          : fresh,
    );
  }

  /// Reads the pick as a `kind` and shows what it holds.
  Future<void> _read(
    Emitter<ImportFlowState> emit,
    ImportFlowState from,
    String kind,
  ) async {
    final pick = from.pick;
    final source = _sourceOf(kind);
    if (pick == null || source == null) {
      emit(from);
      return;
    }
    emit(
      from.copyWith(
        stage: ImportStage.reading,
        source: source,
        progress: null,
        summary: null,
        problem: null,
      ),
    );
    await _follow(
      emit,
      _import.plan(source.kind, pick.path),
      ImportStage.summary,
    );
  }

  /// Shows the progress of [events] and ends in [done], or in a failure.
  Future<void> _follow(
    Emitter<ImportFlowState> emit,
    Stream<ImportEvent> events,
    ImportStage done,
  ) {
    return emit.forEach<ImportEvent>(
      events,
      onData: (event) => switch (event) {
        ImportEvent_Progress(:final field0) => state.copyWith(
          progress: field0,
        ),
        ImportEvent_Done(:final field0) => state.copyWith(
          stage: done,
          progress: null,
          summary: field0,
        ),
        ImportEvent_Failed(:final field0) => state.copyWith(
          stage: ImportStage.failed,
          progress: null,
          problem: field0.trim().isEmpty
              ? const ImportProblem.unexpected()
              : ImportProblem.words(field0.trim()),
        ),
      },
    );
  }

  ImportSource? _sourceOf(String kind) {
    for (final source in state.sources) {
      if (source.kind == kind) {
        return source;
      }
    }
    return null;
  }

  Future<void> _forgetPicked() async {
    if (state.picked) {
      await _files.clearPicked();
    }
  }

  ImportProblem _problemOf(ApplicationError error) {
    return switch (error) {
      InvalidError(:final message) when message.trim().isNotEmpty =>
        ImportProblem.words(message.trim()),
      StorageError() => const ImportProblem.pickFailed(),
      _ => const ImportProblem.unexpected(),
    };
  }

  static String _nameOf(String path) {
    final parts = path.split('/').where((part) => part.isNotEmpty);
    return parts.isEmpty ? path : parts.last;
  }

  @override
  Future<void> close() async {
    await super.close();
    await _forgetPicked();
  }
}
