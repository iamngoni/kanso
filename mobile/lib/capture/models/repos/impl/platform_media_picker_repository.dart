// Kanso — platform_media_picker_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:file_picker/file_picker.dart';
import 'package:image_picker/image_picker.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../data/media_source.dart';
import '../abs/media_picker_repository.dart';

/// [MediaPickerRepository] over `image_picker` and `file_picker`.
@LazySingleton(as: MediaPickerRepository)
class PlatformMediaPickerRepository implements MediaPickerRepository {
  PlatformMediaPickerRepository(this._images);

  /// The longest side of a photo kept: a phone's full-size photo can pass
  /// the 10 MB an attachment may be.
  static const double maxSide = 2560;

  /// The JPEG quality a scaled photo is written at.
  static const int quality = 85;

  final ImagePicker _images;

  @override
  Future<Either<ApplicationError, List<ShareFile>>> pick(
    MediaSource source,
  ) async {
    try {
      final files = switch (source) {
        MediaSource.camera => await _camera(),
        MediaSource.photos => await _photos(),
        MediaSource.pdf => await _pdfs(),
      };
      return Right<ApplicationError, List<ShareFile>>(files);
    } on Object catch (error) {
      return Left<ApplicationError, List<ShareFile>>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }

  Future<List<ShareFile>> _camera() async {
    final photo = await _images.pickImage(
      source: ImageSource.camera,
      maxWidth: maxSide,
      maxHeight: maxSide,
      imageQuality: quality,
    );
    return photo == null ? <ShareFile>[] : <ShareFile>[await _read(photo)];
  }

  Future<List<ShareFile>> _photos() async {
    final photos = await _images.pickMultiImage(
      maxWidth: maxSide,
      maxHeight: maxSide,
      imageQuality: quality,
    );
    return <ShareFile>[for (final photo in photos) await _read(photo)];
  }

  Future<List<ShareFile>> _pdfs() async {
    final picked = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: const <String>['pdf'],
    );
    return <ShareFile>[
      for (final file in picked)
        ShareFile(
          name: file.name,
          contentType: 'application/pdf',
          bytes: await file.readAsBytes(),
        ),
    ];
  }

  Future<ShareFile> _read(XFile file) async => ShareFile(
    name: file.name,
    contentType: file.mimeType,
    bytes: await file.readAsBytes(),
  );
}
