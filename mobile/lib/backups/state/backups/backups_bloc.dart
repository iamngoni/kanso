// Kanso — backups_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/load_status.dart';
import '../../models/data/backups_notice.dart';
import '../../models/data/backups_problem.dart';
import '../../models/repos/abs/backups_repository.dart';

part 'backups_bloc.freezed.dart';
part 'backups_event.dart';
part 'backups_state.dart';

/// The library's database backups: the daily one made when the app opens,
/// the list, Back up now and deleting one made on request.
///
/// The daily backup is made once the app is open and never blocks anything:
/// a failure is kept and the screen says so when it is opened. A backup is
/// listed only as the library's own folder has it, so what is shown is what
/// is on the device.
@injectable
class BackupsBloc extends Bloc<BackupsEvent, BackupsState> {
  BackupsBloc(this._backups) : super(const BackupsState()) {
    on<BackupsStartedEvent>(_onStarted, transformer: restartable());
    on<BackupsRefreshedEvent>(_onRefreshed, transformer: restartable());
    on<BackupsBackUpNowEvent>(_onBackUpNow, transformer: droppable());
    on<BackupsDeleteEvent>(_onDelete, transformer: sequential());
    on<BackupsNoticeClearedEvent>(_onNoticeCleared);
  }

  final BackupsRepository _backups;

  Future<void> _onStarted(
    BackupsStartedEvent event,
    Emitter<BackupsState> emit,
  ) async {
    final made = await _backups.backUpIfDue();
    final daily = made.isLeft() ? BackupsProblem.dailyFailed : null;
    await _read(emit, problem: daily);
  }

  Future<void> _onRefreshed(
    BackupsRefreshedEvent event,
    Emitter<BackupsState> emit,
  ) => _read(emit, problem: state.problem);

  /// Reads the list. A failure after a first read keeps the list shown.
  Future<void> _read(
    Emitter<BackupsState> emit, {
    required BackupsProblem? problem,
  }) async {
    final read = await _backups.backups();
    emit(
      read.fold(
        (_) => state.status == LoadStatus.ready
            ? state.copyWith(problem: BackupsProblem.listFailed)
            : state.copyWith(status: LoadStatus.failed, problem: null),
        (found) => state.copyWith(
          status: LoadStatus.ready,
          backups: found,
          problem: problem == BackupsProblem.listFailed ? null : problem,
        ),
      ),
    );
  }

  Future<void> _onBackUpNow(
    BackupsBackUpNowEvent event,
    Emitter<BackupsState> emit,
  ) async {
    emit(state.copyWith(working: true, problem: null, notice: null));
    final made = await _backups.backUpNow();
    if (made.isLeft()) {
      emit(
        state.copyWith(working: false, problem: BackupsProblem.backUpFailed),
      );
      return;
    }
    emit(state.copyWith(notice: BackupsNotice.madeNow));
    await _read(emit, problem: null);
    emit(state.copyWith(working: false));
  }

  Future<void> _onDelete(
    BackupsDeleteEvent event,
    Emitter<BackupsState> emit,
  ) async {
    emit(
      state.copyWith(
        deleting: <String>{...state.deleting, event.path},
        problem: null,
        notice: null,
      ),
    );
    final gone = await _backups.delete(event.path);
    final deleting = <String>{...state.deleting}..remove(event.path);
    if (gone.isLeft()) {
      emit(
        state.copyWith(
          deleting: deleting,
          problem: BackupsProblem.deleteFailed,
        ),
      );
      return;
    }
    emit(state.copyWith(deleting: deleting, notice: BackupsNotice.deleted));
    await _read(emit, problem: null);
  }

  void _onNoticeCleared(
    BackupsNoticeClearedEvent event,
    Emitter<BackupsState> emit,
  ) {
    emit(state.copyWith(notice: null));
  }
}
