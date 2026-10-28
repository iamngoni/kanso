// Kanso — platform_note_window_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/models/data/application_error.dart';
import '../../../../core/models/data/library_folder.dart';
import '../../../../injection.dart';
import '../../../configs/window_configs.dart';
import '../../data/note_window_info.dart';
import '../../data/note_window_request.dart';
import '../abs/note_window_repository.dart';

/// [NoteWindowRepository] over the native side's `app.kanso/windows`
/// channel: an iPadOS scene, or an Android task, per note, each with an
/// engine of its own in this process.
///
/// Every engine has one of these. The main window's hears the requests the
/// note windows hand over; a note window's sends them.
class PlatformNoteWindowRepository implements NoteWindowRepository {
  /// [supported] is what the device answered when the app started (see
  /// [create]); the environment and the folder are handed to the windows it
  /// opens, so that they run the same app on the same library.
  PlatformNoteWindowRepository({
    required this.supported,
    required this._environment,
    required this._folder,
    MethodChannel? channel,
  }) : _channel = channel ?? const MethodChannel(WindowConfigs.channel) {
    _channel.setMethodCallHandler(_onCall);
  }

  /// Asks the device whether it shows more than one window, and makes the
  /// repository. A device that doesn't answer (a test, a platform without
  /// windows) shows one.
  static Future<PlatformNoteWindowRepository> create(
    LibraryFolder folder, {
    String? environment,
    MethodChannel? channel,
  }) async {
    final used = channel ?? const MethodChannel(WindowConfigs.channel);
    var supported = false;
    try {
      supported = await used.invokeMethod<bool>('supported') ?? false;
    } on Object {
      supported = false;
    }
    return PlatformNoteWindowRepository(
      supported: supported,
      environment: environment ?? activeEnvironmentName,
      folder: folder,
      channel: used,
    );
  }

  @override
  final bool supported;

  final String _environment;
  final LibraryFolder _folder;
  final MethodChannel _channel;
  final StreamController<NoteWindowRequest> _requests =
      StreamController<NoteWindowRequest>.broadcast();

  @override
  Future<Either<ApplicationError, Unit>> open(String noteId) {
    if (!supported) {
      return Future.value(left(_unsupported));
    }
    return _call('open', <String, Object?>{
      'noteId': noteId,
      'environment': _environment,
      'libraryFolder': _folder.path,
    });
  }

  @override
  Future<Either<ApplicationError, Unit>> closeThisWindow() =>
      _call('closeThisWindow');

  @override
  Future<Either<ApplicationError, Unit>> setTitle(String title) =>
      _call('setTitle', <String, Object?>{'title': title});

  @override
  Future<Either<ApplicationError, Unit>> hand(NoteWindowRequest request) =>
      _call('hand', encodeRequest(request));

  @override
  Future<Either<ApplicationError, Unit>> closeWindowOf(String noteId) =>
      _call('closeWindowOf', <String, Object?>{'noteId': noteId});

  @override
  Future<Either<ApplicationError, List<NoteWindowInfo>>> openWindows() async {
    try {
      final answer = await _channel.invokeListMethod<Map<Object?, Object?>>(
        'openWindows',
      );
      return right(<NoteWindowInfo>[
        for (final window in answer ?? const <Map<Object?, Object?>>[])
          if (window['noteId'] case final String id)
            NoteWindowInfo(
              noteId: id,
              title: window['title'] is String
                  ? window['title']! as String
                  : '',
            ),
      ]);
    } on MissingPluginException {
      return right(const <NoteWindowInfo>[]);
    } on PlatformException catch (error) {
      return left(
        ApplicationError.unexpected(message: error.message ?? error.code),
      );
    } on Object catch (error) {
      return left(ApplicationError.unexpected(message: '$error'));
    }
  }

  @override
  Stream<NoteWindowRequest> requests() => _requests.stream;

  /// The request as the channel carries it.
  static Map<String, Object?> encodeRequest(NoteWindowRequest request) {
    return switch (request) {
      NewNoteRequest() => <String, Object?>{'kind': 'newNote'},
      NewNoteTitledRequest(:final title) => <String, Object?>{
        'kind': 'newNoteTitled',
        'title': title,
      },
      SearchRequest() => <String, Object?>{'kind': 'search'},
      PaletteRequest() => <String, Object?>{'kind': 'palette'},
      OpenNoteRequest(:final noteId) => <String, Object?>{
        'kind': 'openNote',
        'noteId': noteId,
      },
    };
  }

  /// The request a channel call carried; null when it is not one this
  /// version knows.
  static NoteWindowRequest? decodeRequest(Object? arguments) {
    if (arguments is! Map) {
      return null;
    }
    return switch (arguments['kind']) {
      'newNote' => const NoteWindowRequest.newNote(),
      'newNoteTitled' when arguments['title'] is String =>
        NoteWindowRequest.newNoteTitled(arguments['title']! as String),
      'search' => const NoteWindowRequest.search(),
      'palette' => const NoteWindowRequest.palette(),
      'openNote' when arguments['noteId'] is String =>
        NoteWindowRequest.openNote(arguments['noteId']! as String),
      _ => null,
    };
  }

  Future<dynamic> _onCall(MethodCall call) async {
    if (call.method != 'request') {
      throw MissingPluginException(call.method);
    }
    final request = decodeRequest(call.arguments);
    if (request != null && !_requests.isClosed) {
      _requests.add(request);
    }
    return null;
  }

  Future<Either<ApplicationError, Unit>> _call(
    String method, [
    Map<String, Object?>? arguments,
  ]) async {
    try {
      await _channel.invokeMethod<void>(method, arguments);
      return right(unit);
    } on MissingPluginException {
      return left(_unsupported);
    } on PlatformException catch (error) {
      return left(
        ApplicationError.unexpected(message: error.message ?? error.code),
      );
    } on Object catch (error) {
      return left(ApplicationError.unexpected(message: '$error'));
    }
  }

  static const ApplicationError _unsupported = ApplicationError.invalid(
    message: 'This device shows one window.',
  );
}

/// Makes the repository of this engine.
@module
abstract class WindowModule {
  @Singleton(as: NoteWindowRepository)
  @preResolve
  Future<PlatformNoteWindowRepository> noteWindows(LibraryFolder folder) =>
      PlatformNoteWindowRepository.create(folder);
}
