// Kanso — platform_capture_window_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';

import '../../data/capture_mode.dart';
import '../abs/capture_window_repository.dart';

/// [CaptureWindowRepository] over the `app.kanso/capture` channel that
/// Android's `CaptureActivity` answers.
class PlatformCaptureWindowRepository implements CaptureWindowRepository {
  PlatformCaptureWindowRepository([
    this._channel = const MethodChannel(channelName),
  ]);

  /// The channel's name, as `CaptureActivity.CHANNEL` has it.
  static const String channelName = 'app.kanso/capture';

  final MethodChannel _channel;

  @override
  Future<CaptureMode> mode() async {
    try {
      final mode = await _channel.invokeMethod<String>('mode');
      return mode == 'task' ? CaptureMode.task : CaptureMode.note;
    } on PlatformException {
      return CaptureMode.note;
    } on MissingPluginException {
      return CaptureMode.note;
    }
  }

  @override
  Future<void> finish() => _call('finish');

  @override
  Future<void> openInKanso(String link) => _call('open', link);

  Future<void> _call(String method, [Object? arguments]) async {
    try {
      await _channel.invokeMethod<void>(method, arguments);
    } on PlatformException {
      // The window is going in any case.
    } on MissingPluginException {
      // No window to close: nothing to do.
    }
  }
}
