// Kanso — push_registration_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/application_error.dart';
import '../../models/data/push_registration_phase.dart';
import '../../models/data/push_token.dart';
import '../../models/repos/abs/push_repository.dart';

part 'push_registration_bloc.freezed.dart';
part 'push_registration_event.dart';
part 'push_registration_state.dart';

/// Keeps this phone's push token registered: asks the OS for it once, hands
/// each token the OS issues (a new one included) to the library, and has the
/// library retry when the app starts and when it comes back to the front.
///
/// Nothing here may crash the app: a failure is kept in the state, and the
/// next token or return to the front tries again.
@injectable
class PushRegistrationBloc
    extends Bloc<PushRegistrationEvent, PushRegistrationState> {
  PushRegistrationBloc(this._push) : super(const PushRegistrationState()) {
    on<PushRegistrationStartedEvent>(_onStarted, transformer: restartable());
    // In the order the OS issued them, so the last token is the one kept.
    on<PushRegistrationTokenReceivedEvent>(
      _onTokenReceived,
      transformer: sequential(),
    );
    on<PushRegistrationResumedEvent>(_onResumed, transformer: droppable());
  }

  final PushRepository _push;

  Future<void> _onStarted(
    PushRegistrationStartedEvent event,
    Emitter<PushRegistrationState> emit,
  ) async {
    final reason = await _push.unavailableReason();
    final unavailable = reason.fold((_) => true, (why) => why != null);
    if (unavailable) {
      emit(
        PushRegistrationState(
          phase: PushRegistrationPhase.unavailable,
          error: reason.fold((error) => error, (_) => null),
        ),
      );
      return;
    }
    // Listening comes before asking, so a token that arrives at once is not
    // missed.
    final listening = emit.onEach<PushToken>(
      _push.tokens(),
      onData: (token) => add(PushRegistrationEvent.tokenReceived(token)),
    );
    final asked = await _push.register();
    asked.fold(
      (error) => emit(state.copyWith(error: error)),
      (_) => emit(
        state.copyWith(phase: PushRegistrationPhase.waiting, error: null),
      ),
    );
    await _refresh(emit, clearsError: asked.isRight());
    await listening;
  }

  Future<void> _onTokenReceived(
    PushRegistrationTokenReceivedEvent event,
    Emitter<PushRegistrationState> emit,
  ) async {
    final result = await _push.setToken(event.token);
    emit(
      result.fold(
        (error) => state.copyWith(error: error),
        (_) => state.copyWith(
          phase: PushRegistrationPhase.registered,
          error: null,
        ),
      ),
    );
  }

  Future<void> _onResumed(
    PushRegistrationResumedEvent event,
    Emitter<PushRegistrationState> emit,
  ) {
    if (state.phase == PushRegistrationPhase.unavailable) {
      return Future<void>.value();
    }
    return _refresh(emit, clearsError: true);
  }

  /// Has the library bring the server in line. A retry that works ends the
  /// error of the last try ([clearsError]); the first try at start does not
  /// hide an error of asking the OS.
  Future<void> _refresh(
    Emitter<PushRegistrationState> emit, {
    required bool clearsError,
  }) async {
    final result = await _push.refresh();
    result.fold(
      (error) => emit(state.copyWith(error: error)),
      (_) {
        if (clearsError) {
          emit(state.copyWith(error: null));
        }
      },
    );
  }
}
