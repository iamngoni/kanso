// Kanso — library_pane_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

part 'library_pane_bloc.freezed.dart';
part 'library_pane_event.dart';
part 'library_pane_state.dart';

/// Whether the library pane is shown beside the list, in a window wide
/// enough for all three panes. Hidden, the library is a drawer, as in
/// narrower windows.
///
/// [LibraryPaneState.panesHidden] puts both the library and the list away
/// while a note is open, on a tablet, without the rest of focus mode.
@injectable
class LibraryPaneBloc extends Bloc<LibraryPaneEvent, LibraryPaneState> {
  LibraryPaneBloc() : super(const LibraryPaneState()) {
    on<LibraryPaneToggledEvent>(
      (event, emit) => emit(state.copyWith(visible: !state.visible)),
    );
    on<LibraryPanesToggledEvent>(
      (event, emit) => emit(state.copyWith(panesHidden: !state.panesHidden)),
    );
  }
}
