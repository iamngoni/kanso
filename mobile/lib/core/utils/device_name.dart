// Kanso — device_name.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../configs/kanso_layout.dart';

/// What this phone is called to the account's other devices until the person
/// names it: "iPhone", "iPad", "Android phone" or "Android tablet".
///
/// A tablet is a display whose shortest side is [shortestSide] or more (see
/// `KansoLayout.tabletShortestSide`).
String deviceNameFor(TargetPlatform platform, double shortestSide) {
  final tablet = shortestSide >= KansoLayout.tabletShortestSide;
  return switch (platform) {
    TargetPlatform.iOS => tablet ? 'iPad' : 'iPhone',
    _ => tablet ? 'Android tablet' : 'Android phone',
  };
}

/// [deviceNameFor] for the device [context] is drawn on.
String deviceNameOf(BuildContext context) {
  return deviceNameFor(
    defaultTargetPlatform,
    View.of(context).display.size.shortestSide /
        View.of(context).display.devicePixelRatio,
  );
}
