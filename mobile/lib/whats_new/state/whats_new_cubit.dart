// Kanso — whats_new_cubit.dart
// Created 2026-11-12.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:injectable/injectable.dart';

import '../../core/bridge/api/whats_new_types.dart';
import '../../core/models/repos/abs/whats_new_repository.dart';

/// What's new, once after an update: the sheet to show, or none.
///
/// [load] asks once per run; [seen] puts the sheet away for this version.
/// A failure shows nothing and the next start asks again: What's new is
/// never worth an error of its own.
@injectable
class WhatsNewCubit extends Cubit<PhoneWhatsNew?> {
  WhatsNewCubit(this._repository) : super(null);

  final WhatsNewRepository _repository;
  bool _asked = false;

  /// Asks the library, the first time only.
  Future<void> load() async {
    if (_asked) {
      return;
    }
    _asked = true;
    final result = await _repository.whatsNew();
    if (isClosed) {
      return;
    }
    result.fold((_) => emit(null), emit);
  }

  /// The sheet was shown: it doesn't come back for this version.
  Future<void> seen() async {
    emit(null);
    await _repository.markSeen();
  }
}
