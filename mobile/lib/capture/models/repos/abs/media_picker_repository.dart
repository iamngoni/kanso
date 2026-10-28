// Kanso — media_picker_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../data/media_source.dart';

/// The phone's own pickers: the camera, the photo library and the files.
// ignore: one_member_abstracts
abstract interface class MediaPickerRepository {
  /// What the person picked from [source], read into memory; empty when
  /// they picked nothing. Photos are scaled to fit an attachment.
  Future<Either<ApplicationError, List<ShareFile>>> pick(MediaSource source);
}
