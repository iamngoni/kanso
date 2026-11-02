// Kanso — triage_bloc.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/triage_types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/haptics_repository.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../core/utils/debounce.dart';
import '../../models/data/triage_card_state.dart';
import '../../models/repos/abs/triage_repository.dart';

part 'triage_bloc.freezed.dart';
part 'triage_event.dart';
part 'triage_state.dart';

/// How many cards are read at once.
const int triageCardLimit = 200;

/// How long the library stays still before the cards are read again.
const Duration triageReloadDelay = Duration(milliseconds: 250);

/// How long the Undo notice after filing stays.
const Duration triageUndoStays = Duration(seconds: 8);

/// Inbox triage: a card for each note in the Inbox, with a chip for each
/// suggested notebook, tag, title and link. The person accepts or rejects
/// each chip (a rejection is remembered), and File applies the accepted
/// chips of a card, or File all accepted those of every card, asking once.
/// Nothing is written until then, and the notice that follows offers Undo.
///
/// The suggestions are the core's (`kanso_engine::triage`): this holds only
/// what the person did to them. A note changed on another device while its
/// card shows says so, and the card is made again before it can be filed.
@injectable
class TriageBloc extends Bloc<TriageEvent, TriageState> {
  TriageBloc(this._triage, this._undoNotice, this._haptics)
    : super(const TriageState()) {
    on<TriageStartedEvent>(_onStarted, transformer: restartable());
    on<TriageChangesReceivedEvent>(_onChanges, transformer: sequential());
    on<TriageReloadedEvent>(
      (event, emit) => _load(emit),
      transformer: debounceRestartable<TriageReloadedEvent>(triageReloadDelay),
    );
    // One queue for all three: a `sequential()` of its own for each would
    // let an accept and a reject of the same chip run side by side.
    on<TriageChipAcceptedEvent>(
      (event, emit) => _inOrder(() => _onAccepted(event, emit)),
    );
    on<TriageChipRejectedEvent>(
      (event, emit) => _inOrder(() => _onRejected(event, emit)),
    );
    on<TriageChipEditedEvent>(
      (event, emit) => _inOrder(() => _onEdited(event, emit)),
    );
    on<TriageCardAcceptedAllEvent>(
      (event, emit) => _update(emit, event.noteId, (c) => c.acceptAll()),
    );
    on<TriageFieldAcceptedEverywhereEvent>(_onAcceptedEverywhere);
    on<TriageCardSkippedEvent>(
      (event, emit) =>
          _update(emit, event.noteId, (c) => c.copyWith(skipped: true)),
    );
    on<TriageCardFiledEvent>(_onCardFiled, transformer: sequential());
    on<TriageFileAllRequestedEvent>(_onFileAllRequested);
    on<TriageFileAllConfirmedEvent>(
      _onFileAllConfirmed,
      transformer: droppable(),
    );
    on<TriagePlanDismissedEvent>((event, emit) {
      _planned = null;
      emit(state.copyWith(plan: null));
    });
    on<TriageRefineRequestedEvent>(_onRefine, transformer: sequential());
    on<TriageRelatedTurnedOnEvent>(
      _onRelatedTurnedOn,
      transformer: droppable(),
    );
    on<TriageNoticeDismissedEvent>(
      (event, emit) => emit(state.copyWith(notice: null)),
    );
  }

  final TriageRepository _triage;
  final UndoNoticeBloc _undoNotice;
  final HapticsRepository _haptics;
  StreamSubscription<List<String>>? _changes;
  final Set<String> _asked = <String>{};

  /// What "File all accepted" asked about, kept from the plan: the answer
  /// files exactly this, whatever the cards became meanwhile.
  List<TriageFiling>? _planned;

  /// The end of the chip queue.
  Future<void> _tail = Future<void>.value();

  /// Runs [work] after everything queued before it, one at a time.
  Future<void> _inOrder(Future<void> Function() work) {
    final run = _tail.then((_) => work());
    _tail = run.then((_) {}, onError: (Object _) {});
    return run;
  }

  Future<void> _onStarted(
    TriageStartedEvent event,
    Emitter<TriageState> emit,
  ) async {
    emit(const TriageState());
    _asked.clear();
    _planned = null;
    await _changes?.cancel();
    _changes = _triage.changes().listen(
      (ids) => add(TriageEvent.changesReceived(ids)),
    );
    await _load(emit);
  }

  /// Reads the cards, keeping what was done to those still there: a card
  /// whose note has not changed is kept as it is, one that has keeps the
  /// marks of the chips still suggested.
  Future<void> _load(Emitter<TriageState> emit) async {
    final status = await _triage.status();
    final read = await _triage.cards(limit: triageCardLimit);
    if (emit.isDone) {
      return;
    }
    final info = status.toOption().toNullable();
    read.fold(
      (error) => emit(state.copyWith(status: LoadStatus.failed, error: error)),
      (cards) {
        final before = <String, TriageCardState>{
          for (final c in state.cards) c.noteId: c,
        };
        emit(
          state.copyWith(
            status: LoadStatus.ready,
            error: null,
            info: info ?? state.info,
            cards: <TriageCardState>[
              for (final card in cards) _merged(before[card.noteId], card),
            ],
          ),
        );
      },
    );
    if (info?.aiOn ?? false) {
      for (final card in state.cards) {
        if (_asked.add(card.noteId)) {
          add(TriageEvent.refineRequested(card.noteId));
        }
      }
    }
  }

  TriageCardState _merged(TriageCardState? before, TriageCard card) {
    if (before == null) {
      return TriageCardState.fromCard(card);
    }
    if (before.rev == card.rev) {
      return before.copyWith(changed: false);
    }
    // The note changed: the new chips, with the marks of those still there.
    return TriageCardState.fromCard(card).copyWith(
      chips: before.withChips(card.chips).chips,
      skipped: before.skipped,
      aiError: before.aiError,
    );
  }

  void _onChanges(
    TriageChangesReceivedEvent event,
    Emitter<TriageState> emit,
  ) {
    final ids = event.ids.toSet();
    final touched = state.cards.any((c) => ids.contains(c.noteId));
    if (touched) {
      emit(
        state.copyWith(
          cards: <TriageCardState>[
            for (final c in state.cards)
              ids.contains(c.noteId) ? c.copyWith(changed: true) : c,
          ],
        ),
      );
    }
    // A note coming into the Inbox, or leaving it, changes the list too.
    add(const TriageEvent.reloaded());
  }

  void _update(
    Emitter<TriageState> emit,
    String noteId,
    TriageCardState Function(TriageCardState) change,
  ) {
    emit(
      state.copyWith(
        cards: <TriageCardState>[
          for (final c in state.cards) c.noteId == noteId ? change(c) : c,
        ],
      ),
    );
  }

  TriageCardState? _card(String noteId) {
    for (final c in state.cards) {
      if (c.noteId == noteId) {
        return c;
      }
    }
    return null;
  }

  Future<void> _onAccepted(
    TriageChipAcceptedEvent event,
    Emitter<TriageState> emit,
  ) async {
    final card = _card(event.noteId);
    if (card == null || event.index >= card.chips.length) {
      return;
    }
    final chip = card.chips[event.index];
    if (chip.mark == ChipMark.rejected) {
      await _triage.unreject(event.noteId, chip.chip.field, chip.chip.value);
    }
    if (emit.isDone) {
      return;
    }
    _update(emit, event.noteId, (c) => c.toggleAccept(event.index));
    await _haptics.selection();
  }

  Future<void> _onRejected(
    TriageChipRejectedEvent event,
    Emitter<TriageState> emit,
  ) async {
    final card = _card(event.noteId);
    if (card == null || event.index >= card.chips.length) {
      return;
    }
    final chip = card.chips[event.index];
    _update(emit, event.noteId, (c) => c.toggleReject(event.index));
    // Remembered, so the same suggestion does not come back.
    if (chip.mark == ChipMark.rejected) {
      await _triage.unreject(event.noteId, chip.chip.field, chip.chip.value);
    } else {
      await _triage.reject(event.noteId, chip.chip.field, chip.chip.value);
    }
    await _haptics.selection();
  }

  Future<void> _onEdited(
    TriageChipEditedEvent event,
    Emitter<TriageState> emit,
  ) async {
    final card = _card(event.noteId);
    if (card == null || event.index >= card.chips.length) {
      return;
    }
    final field = card.chips[event.index].chip.field;
    final resolved = await _triage.resolve(field, event.text);
    if (emit.isDone) {
      return;
    }
    resolved.fold(
      (error) => emit(state.copyWith(notice: _words(error))),
      (chip) => _update(emit, event.noteId, (c) => c.edit(event.index, chip)),
    );
  }

  void _onAcceptedEverywhere(
    TriageFieldAcceptedEverywhereEvent event,
    Emitter<TriageState> emit,
  ) {
    emit(
      state.copyWith(
        cards: <TriageCardState>[
          for (final c in state.cards)
            c.skipped ? c : c.acceptField(event.field),
        ],
      ),
    );
  }

  Future<void> _onCardFiled(
    TriageCardFiledEvent event,
    Emitter<TriageState> emit,
  ) async {
    final filing = _card(event.noteId)?.filing;
    if (filing == null) {
      return;
    }
    await _file(<TriageFiling>[filing], emit);
  }

  Future<void> _onFileAllRequested(
    TriageFileAllRequestedEvent event,
    Emitter<TriageState> emit,
  ) async {
    final filings = <TriageFiling>[
      for (final c in state.fileable) c.filing!,
    ];
    if (filings.isEmpty) {
      return;
    }
    final planned = await _triage.plan(filings);
    if (emit.isDone) {
      return;
    }
    planned.fold(
      (error) => emit(state.copyWith(notice: _words(error))),
      (plan) {
        _planned = filings;
        emit(state.copyWith(plan: plan));
      },
    );
  }

  Future<void> _onFileAllConfirmed(
    TriageFileAllConfirmedEvent event,
    Emitter<TriageState> emit,
  ) async {
    // The filings the question was about, not the cards as they are now.
    final filings = _planned;
    _planned = null;
    emit(state.copyWith(plan: null));
    if (filings != null && filings.isNotEmpty) {
      await _file(filings, emit);
    }
  }

  /// Files [filings] in one write, takes the filed notes off the list,
  /// shows Undo, and makes the cards of notes that changed meanwhile again.
  Future<void> _file(
    List<TriageFiling> filings,
    Emitter<TriageState> emit,
  ) async {
    final result = await _triage.file(filings);
    if (emit.isDone) {
      return;
    }
    await result.fold(
      (error) async => emit(state.copyWith(notice: _words(error))),
      (filed) async {
        final stale = filed.stale.toSet();
        final done = <String>{
          for (final f in filings)
            if (!stale.contains(f.noteId)) f.noteId,
        };
        emit(
          state.copyWith(
            cards: <TriageCardState>[
              for (final c in state.cards)
                if (!done.contains(c.noteId))
                  stale.contains(c.noteId) ? c.copyWith(changed: true) : c,
            ],
          ),
        );
        if (filed.undo > 0) {
          _undoNotice.add(
            UndoNoticeEvent.shown(filed.undo, filed.message, triageUndoStays),
          );
        }
        await _haptics.impact();
        if (stale.isNotEmpty) {
          add(const TriageEvent.reloaded());
        }
      },
    );
  }

  Future<void> _onRefine(
    TriageRefineRequestedEvent event,
    Emitter<TriageState> emit,
  ) async {
    final result = await _triage.refine(event.noteId);
    if (emit.isDone || _card(event.noteId) == null) {
      return;
    }
    result.fold(
      (error) => _update(
        emit,
        event.noteId,
        (c) => c.copyWith(aiError: _words(error)),
      ),
      (chips) => _update(
        emit,
        event.noteId,
        (c) => c.withChips(chips).copyWith(aiError: null),
      ),
    );
  }

  Future<void> _onRelatedTurnedOn(
    TriageRelatedTurnedOnEvent event,
    Emitter<TriageState> emit,
  ) async {
    final result = await _triage.turnOnRelatedNotes();
    if (emit.isDone) {
      return;
    }
    await result.fold(
      (error) async => emit(state.copyWith(notice: _words(error))),
      (_) async => _load(emit),
    );
  }

  /// An error as one plain sentence.
  static String _words(ApplicationError error) => error.message;

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
