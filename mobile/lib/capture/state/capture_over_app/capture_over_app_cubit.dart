// Kanso — capture_over_app_cubit.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';

import '../../../core/bridge/api/types.dart' show Share;
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/capture_repository.dart';
import '../../models/data/capture_over_app_status.dart';
import '../../models/data/media_source.dart';
import '../../models/repos/abs/media_picker_repository.dart';

/// The Note mode of the capture sheet over another app: a photo taken,
/// photos or a PDF, picked and saved as one note in the Inbox through the
/// library's `capture`, as the sheet inside the app saves them. Inside the
/// app the note is then opened; over another app it is not, and the sheet
/// says where it went and goes.
class CaptureOverAppCubit extends Cubit<CaptureOverAppStatus> {
  CaptureOverAppCubit(this._capture, this._picker)
    : super(CaptureOverAppStatus.idle);

  final CaptureRepository _capture;
  final MediaPickerRepository _picker;

  /// Picks from [source] and saves what was picked. A second choice while
  /// one is under way is ignored.
  Future<void> sourceChosen(MediaSource source) async {
    if (state == CaptureOverAppStatus.working) {
      return;
    }
    emit(CaptureOverAppStatus.working);
    final picked = await _picker.pick(source);
    final files = picked.fold((_) => null, (files) => files);
    if (isClosed) {
      return;
    }
    if (files == null) {
      emit(CaptureOverAppStatus.failed);
      return;
    }
    if (files.isEmpty) {
      emit(CaptureOverAppStatus.cancelled);
      return;
    }
    final saved = await _capture.capture(Share(files: files));
    if (isClosed) {
      return;
    }
    emit(
      saved.fold(
        (error) => error is LockedError
            ? CaptureOverAppStatus.locked
            : CaptureOverAppStatus.failed,
        (_) => CaptureOverAppStatus.saved,
      ),
    );
  }
}
