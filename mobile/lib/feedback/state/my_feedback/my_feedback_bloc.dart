// Kanso — my_feedback_bloc.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/load_status.dart';
import '../../models/data/feedback_entry.dart';
import '../../models/repos/abs/feedback_repository.dart';

part 'my_feedback_bloc.freezed.dart';
part 'my_feedback_event.dart';
part 'my_feedback_state.dart';

/// The reports this person sent and where each stands. Nothing polls: the
/// server is asked when My feedback opens and at launch (at most every 6
/// hours); Settings only reads what the phone kept. Every event is on one
/// sequential queue, so a delete never races the refresh that follows it,
/// and the first read sees the launch before anything else.
@injectable
class MyFeedbackBloc extends Bloc<MyFeedbackEvent, MyFeedbackState> {
  MyFeedbackBloc(this._feedback) : super(const MyFeedbackState()) {
    on<MyFeedbackStartedEvent>(_onStarted, transformer: sequential());
    on<MyFeedbackRefreshRequestedEvent>(_onRefresh, transformer: sequential());
    on<MyFeedbackDeleteRequestedEvent>(_onDelete, transformer: sequential());
    on<MyFeedbackSeenEvent>(_onSeen, transformer: sequential());
    on<MyFeedbackResumedEvent>(_onResumed, transformer: sequential());
  }

  final FeedbackRepository _feedback;

  Future<void> _onStarted(
    MyFeedbackStartedEvent event,
    Emitter<MyFeedbackState> emit,
  ) async {
    final asks = event.launch && _feedback.refreshDue;
    await _load(emit, refresh: asks);
  }

  Future<void> _onRefresh(
    MyFeedbackRefreshRequestedEvent event,
    Emitter<MyFeedbackState> emit,
  ) => _load(emit, refresh: true);

  Future<void> _onDelete(
    MyFeedbackDeleteRequestedEvent event,
    Emitter<MyFeedbackState> emit,
  ) async {
    // Gone at once, or "Deleting…" until the server confirms.
    await _feedback.delete(event.id);
    await _load(emit, refresh: false);
  }

  Future<void> _onSeen(
    MyFeedbackSeenEvent event,
    Emitter<MyFeedbackState> emit,
  ) async {
    await _feedback.markSeen();
    await _load(emit, refresh: false);
  }

  Future<void> _onResumed(
    MyFeedbackResumedEvent event,
    Emitter<MyFeedbackState> emit,
  ) async {
    await _feedback.flush();
    await _load(emit, refresh: false);
  }

  Future<void> _load(
    Emitter<MyFeedbackState> emit, {
    required bool refresh,
  }) async {
    if (refresh) {
      emit(state.copyWith(refreshing: true));
    }
    final entries = await _feedback.entries(refresh: refresh);
    emit(
      entries == null
          ? state.copyWith(status: LoadStatus.failed, refreshing: false)
          : state.copyWith(
              status: LoadStatus.ready,
              entries: entries,
              refreshing: false,
            ),
    );
  }
}
