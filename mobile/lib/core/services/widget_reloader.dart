// Kanso — widget_reloader.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';

/// The name of the channel the platforms answer on to redraw the widgets.
const String widgetsChannelName = 'app.kanso.kanso/widgets';

/// Asks the phone to draw the home-screen widgets again, after their colours
/// changed: WidgetKit on iOS, Glance on Android. The native halves are in
/// `AppDelegate.swift` and `MainActivity.kt`.
@lazySingleton
class WidgetReloader {
  static const MethodChannel _channel = MethodChannel(widgetsChannelName);

  /// Asks for the widgets to be redrawn. It does nothing off a phone, or
  /// where the platform has no such channel.
  Future<void> reload() async {
    if (!Platform.isIOS && !Platform.isAndroid) {
      return;
    }
    try {
      await _channel.invokeMethod<void>('reload');
    } on MissingPluginException {
      // A host without the channel has no widgets to redraw.
    }
  }
}
