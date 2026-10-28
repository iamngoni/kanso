// Kanso — note_mode_intent_bloc.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/data/note_mode.dart';

part 'note_mode_intent_event.dart';
part 'note_mode_intent_state.dart';
part 'note_mode_intent_bloc.freezed.dart';

/// Carries a requested mode from app-level surfaces to the mounted note.
@injectable
class NoteModeIntentBloc
    extends Bloc<NoteModeIntentEvent, NoteModeIntentState> {
  NoteModeIntentBloc() : super(const NoteModeIntentState()) {
    on<NoteModeIntentRequestedEvent>((event, emit) {
      emit(NoteModeIntentState(noteId: event.noteId, mode: event.mode));
    });
  }
}
