// Kanso — journal_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/journal_repository.dart';
import '../../../core/models/repos/abs/templates_repository.dart';
import '../../../core/utils/journal_time.dart';

part 'journal_bloc.freezed.dart';
part 'journal_event.dart';
part 'journal_state.dart';

/// Finds or makes a note for the app to open: today's, the day before or
/// after, a day of the calendar, a note from a template, or a new template.
/// The Rust core decides what the note is (the same rules as the desktop's
/// journal); this asks, and says which note to open.
@injectable
class JournalBloc extends Bloc<JournalEvent, JournalState> {
  JournalBloc(this._journal, this._templates) : super(const JournalState()) {
    on<JournalTodayRequestedEvent>(
      (event, emit) => _make(emit, () => _journal.today(_now())),
      transformer: droppable(),
    );
    on<JournalStepRequestedEvent>(_onStep, transformer: droppable());
    on<JournalDayRequestedEvent>(
      (event, emit) => _make(emit, () => _journal.openDay(event.day, _now())),
      transformer: droppable(),
    );
    on<JournalFromTemplateRequestedEvent>(
      (event, emit) => _make(
        emit,
        () => _templates.newFromTemplate(
          templateId: event.templateId,
          bookId: event.notebookId,
          now: _now(),
        ),
      ),
      transformer: droppable(),
    );
    on<JournalNewTemplateRequestedEvent>(
      (event, emit) => _make(emit, _templates.newTemplate),
      transformer: droppable(),
    );
    on<JournalErrorDismissedEvent>(
      (event, emit) => emit(state.copyWith(error: null)),
    );
  }

  final JournalRepository _journal;
  final TemplatesRepository _templates;

  LocalTime _now() => localTimeOf(clock.now());

  Future<void> _onStep(
    JournalStepRequestedEvent event,
    Emitter<JournalState> emit,
  ) async {
    emit(state.copyWith(working: true, opened: null, error: null));
    final result = await _journal.step(
      openNote: event.openNote,
      now: _now(),
      forward: event.forward,
    );
    if (emit.isDone) {
      return;
    }
    result.fold(
      (error) => emit(JournalState(error: error)),
      // No day to go to: nothing opens.
      (made) => emit(JournalState(opened: made)),
    );
  }

  Future<void> _make(
    Emitter<JournalState> emit,
    Future<Either<ApplicationError, MadeNote>> Function() make,
  ) async {
    emit(state.copyWith(working: true, opened: null, error: null));
    final result = await make();
    if (emit.isDone) {
      return;
    }
    result.fold(
      (error) => emit(JournalState(error: error)),
      (made) => emit(JournalState(opened: made)),
    );
  }
}
