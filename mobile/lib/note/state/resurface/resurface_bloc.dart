// Kanso — resurface_bloc.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/repos/abs/preferences_repository.dart';
import '../../../discovery/models/repos/abs/discovery_repository.dart';

part 'resurface_bloc.freezed.dart';
part 'resurface_event.dart';
part 'resurface_state.dart';

/// How many past notes the pill offers.
const int resurfaceShown = 3;

/// How long the text stays still before the pill asks.
const Duration resurfaceSettle = Duration(milliseconds: 1500);

/// Past notes about the paragraph being written, and "On this day" for a
/// daily note.
///
/// It asks only after typing pauses ([settle]), never while it goes on; the
/// notes already offered stay where they are and new ones come in at the
/// end. It offers nothing for a note that is locked or tagged
/// `#ai-exclude` (the library makes no query for one), while related notes
/// are off, or when the Settings say not to.
@injectable
class ResurfaceBloc extends Bloc<ResurfaceEvent, ResurfaceState> {
  ResurfaceBloc(this._discovery, this._preferences)
    : super(const ResurfaceState()) {
    on<ResurfaceStartedEvent>(_onStarted);
    on<ResurfaceEditedEvent>(_onEdited, transformer: restartable());
    on<ResurfaceDismissedEvent>(_onDismissed);
  }

  final DiscoveryRepository _discovery;
  final PreferencesRepository _preferences;

  /// How long the text stays still before asking; tests make it shorter.
  @visibleForTesting
  Duration settle = resurfaceSettle;

  /// The zone now, in seconds from UTC; tests fix it.
  @visibleForTesting
  int Function() utcOffsetSecs = _systemOffset;

  static int _systemOffset() => DateTime.now().timeZoneOffset.inSeconds;

  /// An edit that came before the note was loaded; run once it is.
  ResurfaceEditedEvent? _pending;

  /// What the Settings say now: whether paragraphs are offered, whether the
  /// notes of this day are, and how close a note must be.
  Future<({bool enabled, bool day, String closeness})> _readSettings() async {
    final preferences = await _preferences.load();
    final related = await _discovery.relatedEnabled();
    final settings = preferences.fold((_) => null, (value) => value);
    final on = related.fold((_) => false, (value) => value);
    return (
      enabled: on && (settings?.resurface ?? true),
      day: settings?.onThisDay ?? true,
      closeness: settings?.resurfaceCloseness ?? 'default',
    );
  }

  Future<void> _onStarted(
    ResurfaceStartedEvent event,
    Emitter<ResurfaceState> emit,
  ) async {
    final settings = await _readSettings();
    emit(
      ResurfaceState(
        noteId: event.noteId,
        enabled: settings.enabled,
        closeness: settings.closeness,
      ),
    );
    // An edit that came while the settings were being read.
    final pending = _pending;
    if (pending != null) {
      _pending = null;
      add(pending);
    }
    if (!settings.day) {
      return;
    }
    final day = await _discovery.onThisDay(
      event.noteId,
      utcOffsetSecs: utcOffsetSecs(),
    );
    emit(
      state.copyWith(
        day: day.fold((_) => const <OnThisDayRow>[], (rows) => rows),
      ),
    );
  }

  Future<void> _onEdited(
    ResurfaceEditedEvent event,
    Emitter<ResurfaceState> emit,
  ) async {
    if (state.noteId.isEmpty) {
      // Not loaded yet: ask when it is.
      _pending = event;
      return;
    }
    // Off when the note was opened: look at the Settings again before
    // waiting on anything (they may have been turned on since).
    if (!state.enabled && !await _refresh(emit)) {
      return;
    }
    // Typing: nothing is asked until the text has been still for a moment.
    // A newer edit drops this one (the transformer restarts).
    await Future<void>.delayed(settle);
    if (emit.isDone) {
      return;
    }
    // The Settings may have changed while typing.
    if (!await _refresh(emit)) {
      return;
    }
    final result = await _discovery.resurface(
      id: state.noteId,
      title: event.title,
      body: event.body,
      caret: event.caret,
      closeness: state.closeness,
      utcOffsetSecs: utcOffsetSecs(),
    );
    final rows = result.fold(
      (_) => const <ResurfaceRow>[],
      (found) => switch (found) {
        ResurfaceResult_Found(:final rows) => rows,
        _ => const <ResurfaceRow>[],
      },
    );
    emit(state.copyWith(rows: mergeResurfaced(state.rows, rows)));
  }

  /// Takes the Settings as they are now; true when notes are offered.
  Future<bool> _refresh(Emitter<ResurfaceState> emit) async {
    final settings = await _readSettings();
    if (emit.isDone) {
      return false;
    }
    if (settings.enabled != state.enabled ||
        settings.closeness != state.closeness) {
      emit(
        state.copyWith(
          enabled: settings.enabled,
          closeness: settings.closeness,
          rows: settings.enabled ? state.rows : const <ResurfaceRow>[],
        ),
      );
    }
    return settings.enabled;
  }

  Future<void> _onDismissed(
    ResurfaceDismissedEvent event,
    Emitter<ResurfaceState> emit,
  ) async {
    emit(
      state.copyWith(
        rows: <ResurfaceRow>[
          for (final row in state.rows)
            if (row.id != event.otherId) row,
        ],
      ),
    );
    await _discovery.dismissRelated(state.noteId, event.otherId);
  }
}

/// What is offered after a new search: the notes still relevant stay where
/// they are, the gone go, new ones come in at the end, at most three.
List<ResurfaceRow> mergeResurfaced(
  List<ResurfaceRow> shown,
  List<ResurfaceRow> found,
) {
  final merged = <ResurfaceRow>[];
  for (final row in shown) {
    for (final fresh in found) {
      if (fresh.id == row.id) {
        merged.add(fresh);
        break;
      }
    }
  }
  for (final fresh in found) {
    if (!merged.any((row) => row.id == fresh.id)) {
      merged.add(fresh);
    }
  }
  return merged.length > resurfaceShown
      ? merged.sublist(0, resurfaceShown)
      : merged;
}
