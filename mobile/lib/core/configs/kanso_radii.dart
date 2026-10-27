// Kanso — kanso_radii.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';

/// Corner radii: the desktop's 6 and 10, 14 for sheets and cards, and pills.
abstract final class KansoRadii {
  /// 8: rows and small controls.
  static const double small = 8;

  /// 12: fields and menus.
  static const double medium = 12;

  /// 18: buttons, sheets and cards.
  static const double large = 18;

  /// 999: chips and the knob of a switch.
  static const double pill = 999;

  static const BorderRadius smallBorder = BorderRadius.all(
    Radius.circular(small),
  );
  static const BorderRadius mediumBorder = BorderRadius.all(
    Radius.circular(medium),
  );
  static const BorderRadius largeBorder = BorderRadius.all(
    Radius.circular(large),
  );
  static const BorderRadius pillBorder = BorderRadius.all(
    Radius.circular(pill),
  );
}
