// Kanso — photo_feedback_screenshot_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:image_picker/image_picker.dart';
import 'package:injectable/injectable.dart';

import '../../data/feedback_draft.dart';
import '../abs/feedback_screenshot_repository.dart';

/// [FeedbackScreenshotRepository] over the system photo picker.
@LazySingleton(as: FeedbackScreenshotRepository)
class PhotoFeedbackScreenshotRepository
    implements FeedbackScreenshotRepository {
  PhotoFeedbackScreenshotRepository(this._images);

  final ImagePicker _images;

  @override
  Future<FeedbackScreenshot?> pick() async {
    try {
      final photo = await _images.pickImage(source: ImageSource.gallery);
      if (photo == null) {
        return null;
      }
      return FeedbackScreenshot(
        bytes: await photo.readAsBytes(),
        name: photo.name,
      );
    } on Object {
      return null;
    }
  }
}
