// Kanso — replace_notes_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/utils/debounce.dart';
import '../../models/data/replace_option.dart';
import '../../models/data/replace_preview_status.dart';
import '../../models/data/replace_problem.dart';
import '../../models/repos/abs/replace_repository.dart';

part 'replace_notes_bloc.freezed.dart';
part 'replace_notes_event.dart';
part 'replace_notes_state.dart';

/// How long typing rests before the notes are searched again.
const Duration replacePreviewDebounce = Duration(milliseconds: 150);

/// Replace in all notes: what to find and put instead, a preview of every
/// match note by note (each note can be left out), one change to the notes
/// picked and one Undo for it.
///
/// A locked note is never in the preview, and the library leaves one alone
/// even if a plan names it. A note that changed since the preview is left
/// alone and counted in the result.
@injectable
class ReplaceNotesBloc extends Bloc<ReplaceNotesEvent, ReplaceNotesState> {
  ReplaceNotesBloc(this._replace) : super(const ReplaceNotesState()) {
    on<ReplaceNotesFindChangedEvent>(_onFindChanged, transformer: sequential());
    on<ReplaceNotesReplacementChangedEvent>(_onReplacementChanged);
    on<ReplaceNotesOptionToggledEvent>(
      _onOptionToggled,
      transformer: sequential(),
    );
    on<ReplaceNotesPreviewDueEvent>(
      _onPreviewDue,
      transformer: debounceRestartable<ReplaceNotesPreviewDueEvent>(
        replacePreviewDebounce,
      ),
    );
    on<ReplaceNotesNoteToggledEvent>(_onNoteToggled);
    on<ReplaceNotesApplyRequestedEvent>(
      _onApplyRequested,
      transformer: droppable(),
    );
    on<ReplaceNotesUndoRequestedEvent>(
      _onUndoRequested,
      transformer: droppable(),
    );
    on<ReplaceNotesResultDismissedEvent>(_onResultDismissed);
  }

  final ReplaceRepository _replace;

  void _onFindChanged(
    ReplaceNotesFindChangedEvent event,
    Emitter<ReplaceNotesState> emit,
  ) {
    emit(
      state.copyWith(
        options: _with(state.options, find: event.text),
        problem: null,
        outcome: null,
        undone: null,
      ),
    );
    add(const ReplaceNotesEvent.previewDue());
  }

  void _onReplacementChanged(
    ReplaceNotesReplacementChangedEvent event,
    Emitter<ReplaceNotesState> emit,
  ) {
    emit(state.copyWith(replacement: event.text));
  }

  void _onOptionToggled(
    ReplaceNotesOptionToggledEvent event,
    Emitter<ReplaceNotesState> emit,
  ) {
    final now = state.options;
    emit(
      state.copyWith(
        options: switch (event.option) {
          ReplaceOption.matchCase => _with(now, matchCase: !now.matchCase),
          ReplaceOption.wholeWord => _with(now, wholeWord: !now.wholeWord),
          ReplaceOption.regex => _with(now, regex: !now.regex),
          ReplaceOption.skipCode => _with(now, skipCode: !now.skipCode),
          ReplaceOption.includeTrash => _with(
            now,
            includeTrash: !now.includeTrash,
          ),
        },
        problem: null,
      ),
    );
    add(const ReplaceNotesEvent.previewDue());
  }

  Future<void> _onPreviewDue(
    ReplaceNotesPreviewDueEvent event,
    Emitter<ReplaceNotesState> emit,
  ) async {
    final options = state.options;
    if (options.find.isEmpty) {
      emit(
        state.copyWith(
          previewStatus: ReplacePreviewStatus.idle,
          preview: null,
          excluded: const <String>{},
        ),
      );
      return;
    }
    emit(state.copyWith(previewStatus: ReplacePreviewStatus.loading));
    final found = await _replace.preview(options);
    emit(
      found.fold(
        (_) => state.copyWith(
          previewStatus: ReplacePreviewStatus.failed,
          preview: null,
        ),
        (preview) => state.copyWith(
          previewStatus: ReplacePreviewStatus.ready,
          preview: preview,
          // A note left out stays out while it still has matches.
          excluded: <String>{
            for (final note in preview.notes)
              if (state.excluded.contains(note.id)) note.id,
          },
        ),
      ),
    );
  }

  void _onNoteToggled(
    ReplaceNotesNoteToggledEvent event,
    Emitter<ReplaceNotesState> emit,
  ) {
    final excluded = <String>{...state.excluded};
    if (!excluded.remove(event.noteId)) {
      excluded.add(event.noteId);
    }
    emit(state.copyWith(excluded: excluded));
  }

  Future<void> _onApplyRequested(
    ReplaceNotesApplyRequestedEvent event,
    Emitter<ReplaceNotesState> emit,
  ) async {
    final planned = state.planned;
    if (state.applying || planned.isEmpty) {
      return;
    }
    emit(state.copyWith(applying: true, problem: null));
    final applied = await _replace.apply(
      options: state.options,
      replacement: state.replacement,
      planned: planned,
    );
    emit(
      applied.fold(
        (error) => state.copyWith(
          applying: false,
          problem: error is InvalidError
              ? ReplaceProblem.badPattern
              : ReplaceProblem.applyFailed,
        ),
        // What was previewed is changed: it is looked for again when the
        // person types, and not shown as if it were still there.
        (outcome) => state.copyWith(
          applying: false,
          outcome: outcome,
          undone: null,
          preview: null,
          previewStatus: ReplacePreviewStatus.idle,
          excluded: const <String>{},
        ),
      ),
    );
  }

  Future<void> _onUndoRequested(
    ReplaceNotesUndoRequestedEvent event,
    Emitter<ReplaceNotesState> emit,
  ) async {
    final outcome = state.outcome;
    if (outcome == null || state.undoing || state.undone != null) {
      return;
    }
    emit(state.copyWith(undoing: true, problem: null));
    final undone = await _replace.undo(outcome.batch);
    emit(
      undone.fold(
        (_) => state.copyWith(
          undoing: false,
          problem: ReplaceProblem.undoFailed,
        ),
        (report) => state.copyWith(undoing: false, undone: report),
      ),
    );
  }

  void _onResultDismissed(
    ReplaceNotesResultDismissedEvent event,
    Emitter<ReplaceNotesState> emit,
  ) {
    emit(state.copyWith(outcome: null, undone: null));
  }

  ReplaceOptions _with(
    ReplaceOptions now, {
    String? find,
    bool? matchCase,
    bool? wholeWord,
    bool? regex,
    bool? skipCode,
    bool? includeTrash,
  }) {
    return ReplaceOptions(
      find: find ?? now.find,
      matchCase: matchCase ?? now.matchCase,
      wholeWord: wholeWord ?? now.wholeWord,
      regex: regex ?? now.regex,
      skipCode: skipCode ?? now.skipCode,
      includeTrash: includeTrash ?? now.includeTrash,
    );
  }
}
