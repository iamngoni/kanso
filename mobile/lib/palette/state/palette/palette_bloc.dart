// Kanso — palette_bloc.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/linking_types.dart';
import '../../../core/utils/debounce.dart';
import '../../models/data/palette_status.dart';
import '../../models/repos/abs/palette_repository.dart';

part 'palette_bloc.freezed.dart';
part 'palette_event.dart';
part 'palette_state.dart';

/// How long typing rests before the palette looks again.
const Duration paletteDebounce = Duration(milliseconds: 150);

/// The command palette: the commands the app can run now, the places of the
/// library and, once something is typed, the notes that match.
///
/// What the app can do is the [PaletteContext] the palette was opened with;
/// the Rust core only lists a command the app has a handler for, so a row
/// the app cannot run is never shown.
@injectable
class PaletteBloc extends Bloc<PaletteEvent, PaletteState> {
  PaletteBloc(this._palette) : super(const PaletteState()) {
    on<PaletteStartedEvent>(_onStarted, transformer: sequential());
    on<PaletteQueryChangedEvent>(_onQueryChanged, transformer: sequential());
    on<PaletteSearchDueEvent>(
      _onSearchDue,
      transformer: debounceRestartable<PaletteSearchDueEvent>(paletteDebounce),
    );
  }

  final PaletteRepository _palette;

  /// What the app can do, kept from [PaletteStartedEvent].
  PaletteContext? _context;

  void _onStarted(PaletteStartedEvent event, Emitter<PaletteState> emit) {
    _context = event.context;
    emit(
      state.copyWith(status: PaletteStatus.loading, problem: null),
    );
    add(PaletteSearchDueEvent(state.query));
  }

  void _onQueryChanged(
    PaletteQueryChangedEvent event,
    Emitter<PaletteState> emit,
  ) {
    emit(
      state.copyWith(
        query: event.query,
        status: PaletteStatus.loading,
        problem: null,
      ),
    );
    add(PaletteSearchDueEvent(event.query));
  }

  Future<void> _onSearchDue(
    PaletteSearchDueEvent event,
    Emitter<PaletteState> emit,
  ) async {
    final context = _context;
    if (context == null) {
      return;
    }
    final result = await _palette.search(query: event.query, context: context);
    // A later query owns the results; this one is stale.
    if (isClosed || state.query != event.query) {
      return;
    }
    result.fold(
      (error) => emit(
        state.copyWith(
          status: PaletteStatus.failed,
          problem: error.message,
          sections: const <PaletteSection>[],
        ),
      ),
      (sections) => emit(
        state.copyWith(
          status: PaletteStatus.ready,
          problem: null,
          sections: sections,
        ),
      ),
    );
  }
}
