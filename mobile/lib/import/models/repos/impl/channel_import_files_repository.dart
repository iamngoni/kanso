// Kanso: channel_import_files_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../data/import_pick.dart';
import '../../data/opened_batch.dart';
import '../abs/import_files_repository.dart';

/// [ImportFilesRepository] over the native `app.kanso/files` channel
/// (`ios/Runner/KansoFiles.swift`, `KansoFiles.kt`).
///
/// Dart asks `pickFile`, `pickFolder`, `clearPicked`, `takeOpened` and
/// `discardOpened`; the native side calls `opened` when it has queued a
/// document the system opened in the app. Documents that arrive before
/// anything listens stay queued natively until `takeOpened`.
@LazySingleton(as: ImportFilesRepository)
class ChannelImportFilesRepository implements ImportFilesRepository {
  ChannelImportFilesRepository() {
    _channel.setMethodCallHandler(_onCall);
  }

  /// The channel's name, the same on both platforms.
  static const String channelName = 'app.kanso/files';

  static const MethodChannel _channel = MethodChannel(channelName);

  final StreamController<Unit> _arrivals = StreamController<Unit>.broadcast();

  @override
  Stream<Unit> get arrivals => _arrivals.stream;

  Future<Object?> _onCall(MethodCall call) async {
    if (call.method == 'opened' && _arrivals.hasListener) {
      _arrivals.add(unit);
    }
    return null;
  }

  @override
  Future<Either<ApplicationError, ImportPick?>> pickFile(
    List<String> extensions,
  ) => _pick('pickFile', <String, Object?>{'extensions': extensions});

  @override
  Future<Either<ApplicationError, ImportPick?>> pickFolder() =>
      _pick('pickFolder', null);

  Future<Either<ApplicationError, ImportPick?>> _pick(
    String method,
    Map<String, Object?>? arguments,
  ) async {
    try {
      final answer = await _channel.invokeMapMethod<String, Object?>(
        method,
        arguments,
      );
      final path = answer?['path'];
      if (answer == null || path is! String || path.isEmpty) {
        return const Right<ApplicationError, ImportPick?>(null);
      }
      final name = answer['name'];
      return Right<ApplicationError, ImportPick?>(
        ImportPick(path: path, name: name is String ? name : ''),
      );
    } on Object catch (error) {
      return Left<ApplicationError, ImportPick?>(_storage(error));
    }
  }

  @override
  Future<Either<ApplicationError, Unit>> clearPicked() => _call('clearPicked');

  @override
  Future<Either<ApplicationError, Unit>> discardOpened() =>
      _call('discardOpened');

  Future<Either<ApplicationError, Unit>> _call(String method) async {
    try {
      await _channel.invokeMethod<void>(method);
      return const Right<ApplicationError, Unit>(unit);
    } on Object catch (error) {
      return Left<ApplicationError, Unit>(_storage(error));
    }
  }

  @override
  Future<Either<ApplicationError, OpenedBatch>> takeOpened() async {
    try {
      final answer = await _channel.invokeMapMethod<String, Object?>(
        'takeOpened',
      );
      final staging = answer?['staging'];
      final files = answer?['files'];
      if (staging is! String || files is! List<Object?>) {
        return const Right<ApplicationError, OpenedBatch>(
          OpenedBatch(files: <OpenedFile>[], staging: ''),
        );
      }
      return Right<ApplicationError, OpenedBatch>(
        OpenedBatch(files: _filesOf(files), staging: staging),
      );
    } on Object catch (error) {
      return Left<ApplicationError, OpenedBatch>(_storage(error));
    }
  }

  List<OpenedFile> _filesOf(List<Object?> raw) {
    final files = <OpenedFile>[];
    for (final entry in raw) {
      if (entry is! Map<Object?, Object?>) {
        continue;
      }
      final path = entry['path'];
      final name = entry['name'];
      final identity = entry['identity'];
      if (path is String && path.isNotEmpty) {
        files.add(
          OpenedFile(
            path: path,
            name: name is String ? name : '',
            identity: identity is String && identity.isNotEmpty
                ? identity
                : null,
          ),
        );
      }
    }
    return files;
  }

  ApplicationError _storage(Object error) => ApplicationError.storage(
    message: error is PlatformException
        ? (error.message ?? error.code)
        : '$error',
  );
}
