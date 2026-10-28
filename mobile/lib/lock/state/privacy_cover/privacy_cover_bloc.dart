// Kanso — privacy_cover_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/repos/abs/device_lock_repository.dart';

part 'privacy_cover_bloc.freezed.dart';
part 'privacy_cover_event.dart';
part 'privacy_cover_state.dart';

/// Whether the phone keeps the app out of screenshots and the app switcher
/// (a cover on iOS, a secure window on Android): on exactly while an open
/// locked note is on screen. Asked in order, so the last ask wins.
@injectable
class PrivacyCoverBloc extends Bloc<PrivacyCoverEvent, PrivacyCoverState> {
  PrivacyCoverBloc(this._device) : super(const PrivacyCoverState()) {
    on<PrivacyCoverChangedEvent>(_onChanged, transformer: sequential());
  }

  final DeviceLockRepository _device;

  Future<void> _onChanged(
    PrivacyCoverChangedEvent event,
    Emitter<PrivacyCoverState> emit,
  ) async {
    if (event.secure == state.secure) {
      return;
    }
    final result = await _device.setSecure(on: event.secure);
    // A phone that refuses stays as it was: the state says what is true.
    result.fold((_) {}, (_) => emit(PrivacyCoverState(secure: event.secure)));
  }
}
