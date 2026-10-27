// Kanso — invitations_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/shared_notes_repository.dart';
import '../../models/data/share_problem.dart';
import '../../utils/share_problem_of.dart';

part 'invitations_bloc.freezed.dart';
part 'invitations_event.dart';
part 'invitations_state.dart';

/// The notes other people shared with this account that wait to be
/// accepted: for the Shared with you screen and the count beside its entry
/// in the library.
///
/// The invitations are read when the app starts and again each time a sync
/// of shared notes reports news. A note's title is sealed until it is
/// accepted, so an invitation names only who it is from. One that was
/// accepted or declined here is not shown again, even by a read that began
/// before it was answered.
@injectable
class InvitationsBloc extends Bloc<InvitationsEvent, InvitationsState> {
  InvitationsBloc(this._shared) : super(const InvitationsState()) {
    on<InvitationsStartedEvent>(_onStarted, transformer: restartable());
    on<InvitationsRefreshedEvent>(_onRefreshed, transformer: restartable());
    on<InvitationsAcceptedEvent>(_onAccepted, transformer: concurrent());
    on<InvitationsDeclinedEvent>(_onDeclined, transformer: concurrent());
  }

  final SharedNotesRepository _shared;

  /// The invitations accepted or declined here.
  final Set<String> _answered = <String>{};

  Future<void> _onStarted(
    InvitationsStartedEvent event,
    Emitter<InvitationsState> emit,
  ) async {
    await _read(emit);
    await emit.onEach<SharedEvent>(
      _shared.events(),
      onData: (_) => add(const InvitationsEvent.refreshed()),
    );
  }

  Future<void> _onRefreshed(
    InvitationsRefreshedEvent event,
    Emitter<InvitationsState> emit,
  ) => _read(emit);

  /// Reads the invitations. A failure after a first read keeps what was
  /// shown (a sync that could not reach the server is not news).
  Future<void> _read(Emitter<InvitationsState> emit) async {
    final read = await _shared.invitations();
    emit(
      read.fold(
        (error) => switch (error) {
          SignedOutError() => state.copyWith(
            status: LoadStatus.ready,
            invitations: const <InvitationRow>[],
            signedOut: true,
            problem: null,
          ),
          _ when state.status == LoadStatus.ready => state.copyWith(
            problem: shareProblemOf(error),
          ),
          _ => state.copyWith(status: LoadStatus.failed),
        },
        (invitations) => state.copyWith(
          status: LoadStatus.ready,
          invitations: <InvitationRow>[
            for (final row in invitations)
              if (!_answered.contains(row.shareId)) row,
          ],
          signedOut: false,
          problem: null,
        ),
      ),
    );
  }

  Future<void> _onAccepted(
    InvitationsAcceptedEvent event,
    Emitter<InvitationsState> emit,
  ) async {
    if (state.working.contains(event.shareId)) {
      return;
    }
    emit(
      state.copyWith(
        working: <String>{...state.working, event.shareId},
        problem: null,
      ),
    );
    final accepted = await _shared.accept(event.shareId);
    final working = <String>{...state.working}..remove(event.shareId);
    emit(
      accepted.fold(
        (error) =>
            state.copyWith(working: working, problem: shareProblemOf(error)),
        (noteId) {
          _answered.add(event.shareId);
          return state.copyWith(
            working: working,
            opened: noteId,
            invitations: <InvitationRow>[
              for (final row in state.invitations)
                if (row.shareId != event.shareId) row,
            ],
          );
        },
      ),
    );
  }

  Future<void> _onDeclined(
    InvitationsDeclinedEvent event,
    Emitter<InvitationsState> emit,
  ) async {
    if (state.working.contains(event.shareId)) {
      return;
    }
    emit(
      state.copyWith(
        working: <String>{...state.working, event.shareId},
        problem: null,
      ),
    );
    final declined = await _shared.decline(event.shareId);
    final working = <String>{...state.working}..remove(event.shareId);
    emit(
      declined.fold(
        (error) =>
            state.copyWith(working: working, problem: shareProblemOf(error)),
        (_) {
          _answered.add(event.shareId);
          return state.copyWith(
            working: working,
            invitations: <InvitationRow>[
              for (final row in state.invitations)
                if (row.shareId != event.shareId) row,
            ],
          );
        },
      ),
    );
  }
}
