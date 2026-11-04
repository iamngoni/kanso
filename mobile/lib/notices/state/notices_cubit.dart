// Kanso — notices_cubit.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:injectable/injectable.dart';

import '../../core/bridge/api/notices_types.dart';
import '../../core/models/repos/abs/notices_repository.dart';

/// The update notice at the top of the notes list: the newest one that
/// applies to this phone and has not been put away, or none.
///
/// The notices are compiled into the library; what a person put away is
/// kept on this phone. Dismissing and marking fixed hide the card at once
/// and ask the library to remember.
@injectable
class NoticesCubit extends Cubit<PhoneNotices?> {
  NoticesCubit(this._repository) : super(null);

  final NoticesRepository _repository;

  /// Asks the library what applies now. [signedIn] is whether this phone
  /// has an account. A failure shows no card: a notice is never worth an
  /// error of its own.
  Future<void> refresh({required bool signedIn}) async {
    final result = await _repository.notices(signedIn: signedIn);
    if (isClosed) {
      return;
    }
    result.fold((_) => emit(null), (notices) {
      final card = notices.card;
      emit(card == null ? null : notices);
    });
  }

  /// Hides [id] on this phone for good.
  Future<void> dismiss(String id, {required bool signedIn}) async {
    await _repository.dismiss(id);
    await refresh(signedIn: signedIn);
  }

  /// Hides [id] because the person fixed what it said.
  Future<void> markFixed(String id, {required bool signedIn}) async {
    await _repository.markFixed(id);
    await refresh(signedIn: signedIn);
  }
}
