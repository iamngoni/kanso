// Kanso — send_feedback_bloc.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/data/feedback_draft.dart';
import '../../models/data/feedback_kind.dart';
import '../../models/data/feedback_result.dart';
import '../../models/repos/abs/feedback_repository.dart';
import '../../models/repos/abs/feedback_screenshot_repository.dart';

part 'send_feedback_bloc.freezed.dart';
part 'send_feedback_event.dart';
part 'send_feedback_state.dart';

/// The Send feedback page: the report as it is being written, the exact
/// payload on request, and the send. Every event is on one sequential queue,
/// so a payload preview never races the text that made it, and a send sees
/// the last thing typed.
@injectable
class SendFeedbackBloc extends Bloc<SendFeedbackEvent, SendFeedbackState> {
  SendFeedbackBloc(this._feedback, this._screenshots)
    : super(const SendFeedbackState()) {
    on<SendFeedbackStartedEvent>(
      (event, emit) => emit(state.copyWith(screen: event.screen)),
      transformer: sequential(),
    );
    on<SendFeedbackKindChangedEvent>(
      (event, emit) => _edit(emit, state.draft.copyWith(kind: event.kind)),
      transformer: sequential(),
    );
    on<SendFeedbackTextChangedEvent>(
      (event, emit) => _edit(emit, state.draft.copyWith(text: event.text)),
      transformer: sequential(),
    );
    on<SendFeedbackCreditToggledEvent>(
      (event, emit) => _edit(emit, state.draft.copyWith(credit: event.on)),
      transformer: sequential(),
    );
    on<SendFeedbackCreditNameChangedEvent>(
      (event, emit) =>
          _edit(emit, state.draft.copyWith(creditName: event.name)),
      transformer: sequential(),
    );
    on<SendFeedbackAppInfoToggledEvent>(
      (event, emit) => _edit(emit, state.draft.copyWith(includeApp: event.on)),
      transformer: sequential(),
    );
    on<SendFeedbackScreenshotRequestedEvent>(
      _onScreenshotRequested,
      transformer: sequential(),
    );
    on<SendFeedbackScreenshotRemovedEvent>(
      (event, emit) async {
        emit(state.copyWith(screenshotTooBig: false));
        await _edit(emit, state.draft.copyWith(screenshot: null));
      },
      transformer: sequential(),
    );
    on<SendFeedbackSentToggledEvent>(_onSentToggled, transformer: sequential());
    on<SendFeedbackSendRequestedEvent>(_onSend, transformer: sequential());
  }

  final FeedbackRepository _feedback;
  final FeedbackScreenshotRepository _screenshots;

  /// Takes a new draft, and refreshes the payload when it is on show.
  Future<void> _edit(
    Emitter<SendFeedbackState> emit,
    FeedbackDraft draft,
  ) async {
    emit(state.copyWith(draft: draft, result: null));
    if (state.showSent) {
      await _refreshPayload(emit);
    }
  }

  Future<void> _refreshPayload(Emitter<SendFeedbackState> emit) async {
    final payload = await _feedback.preview(state.draft, screen: state.screen);
    emit(state.copyWith(payload: payload));
  }

  Future<void> _onScreenshotRequested(
    SendFeedbackScreenshotRequestedEvent event,
    Emitter<SendFeedbackState> emit,
  ) async {
    final picked = await _screenshots.pick();
    if (picked == null) {
      return;
    }
    if (picked.bytes.length > feedbackMaxScreenshot) {
      emit(state.copyWith(screenshotTooBig: true));
      return;
    }
    emit(state.copyWith(screenshotTooBig: false));
    await _edit(emit, state.draft.copyWith(screenshot: picked));
  }

  Future<void> _onSentToggled(
    SendFeedbackSentToggledEvent event,
    Emitter<SendFeedbackState> emit,
  ) async {
    emit(state.copyWith(showSent: !state.showSent));
    if (state.showSent) {
      await _refreshPayload(emit);
    }
  }

  Future<void> _onSend(
    SendFeedbackSendRequestedEvent event,
    Emitter<SendFeedbackState> emit,
  ) async {
    if (state.sending || !state.draft.canSend) {
      return;
    }
    emit(state.copyWith(sending: true, result: null));
    final result = await _feedback.send(state.draft, screen: state.screen);
    emit(state.copyWith(sending: false, result: result));
  }
}
