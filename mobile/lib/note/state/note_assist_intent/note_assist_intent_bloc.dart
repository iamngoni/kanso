// Kanso — note_assist_intent_bloc.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/assist_action.dart';

part 'note_assist_intent_event.dart';
part 'note_assist_intent_state.dart';
part 'note_assist_intent_bloc.freezed.dart';

/// Carries a requested assistant action from app-level surfaces, such as
/// the command palette, to the mounted note editor.
@injectable
class NoteAssistIntentBloc
    extends Bloc<NoteAssistIntentEvent, NoteAssistIntentState> {
  NoteAssistIntentBloc() : super(const NoteAssistIntentState()) {
    on<NoteAssistIntentRequestedEvent>((event, emit) {
      if (event.action == null && !event.summarizeNote) {
        return;
      }
      emit(
        NoteAssistIntentState(
          request: state.request + 1,
          noteId: event.noteId,
          action: event.action,
          summarizeNote: event.summarizeNote,
        ),
      );
    });
    on<NoteAssistIntentConsumedEvent>((event, emit) {
      if (event.request == state.request) {
        emit(NoteAssistIntentState(request: state.request));
      }
    });
  }
}
