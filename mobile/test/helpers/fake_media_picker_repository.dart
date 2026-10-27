// Kanso — fake_media_picker_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/capture/models/data/media_source.dart';
import 'package:kanso/capture/models/repos/abs/media_picker_repository.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';

/// A [MediaPickerRepository] with an answer a test sets: nothing picked by
/// default, and the sources it was asked for.
class FakeMediaPickerRepository implements MediaPickerRepository {
  Either<ApplicationError, List<ShareFile>> answer =
      const Right<ApplicationError, List<ShareFile>>(<ShareFile>[]);

  /// The sources the picker was asked for, in turn.
  final List<MediaSource> sources = <MediaSource>[];

  @override
  Future<Either<ApplicationError, List<ShareFile>>> pick(
    MediaSource source,
  ) async {
    sources.add(source);
    return answer;
  }
}
