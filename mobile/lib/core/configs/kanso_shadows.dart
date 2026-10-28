// Kanso — kanso_shadows.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter/painting.dart';

/// Shadows under floating surfaces.
///
/// They are warm, not grey: on a light theme the shadow is a dark umber, on
/// a dark theme a near-black that keeps a trace of the same warmth. Each is
/// two layers, a soft wide one and a tight one, so a surface looks lifted
/// rather than outlined.
abstract final class KansoShadows {
  static const Color _lightBase = Color(0xFF45361F);
  static const Color _darkBase = Color(0xFF070604);

  static Color _base(Brightness brightness) {
    return brightness == Brightness.light ? _lightBase : _darkBase;
  }

  static double _strength(Brightness brightness) {
    return brightness == Brightness.light ? 1 : 2.4;
  }

  /// A button afloat over a list.
  static List<BoxShadow> raised(Brightness brightness) {
    final base = _base(brightness);
    final k = _strength(brightness);
    return <BoxShadow>[
      BoxShadow(
        color: base.withValues(alpha: 0.16 * k),
        blurRadius: 20,
        offset: const Offset(0, 8),
      ),
      BoxShadow(
        color: base.withValues(alpha: 0.10 * k),
        blurRadius: 4,
        offset: const Offset(0, 1),
      ),
    ];
  }

  /// A menu or a popup over text.
  static List<BoxShadow> floating(Brightness brightness) {
    final base = _base(brightness);
    final k = _strength(brightness);
    return <BoxShadow>[
      BoxShadow(
        color: base.withValues(alpha: 0.14 * k),
        blurRadius: 24,
        offset: const Offset(0, 10),
      ),
      BoxShadow(
        color: base.withValues(alpha: 0.08 * k),
        blurRadius: 4,
        offset: const Offset(0, 1),
      ),
    ];
  }

  /// A sheet over the whole screen.
  static List<BoxShadow> sheet(Brightness brightness) {
    final base = _base(brightness);
    final k = _strength(brightness);
    return <BoxShadow>[
      BoxShadow(
        color: base.withValues(alpha: 0.20 * k),
        blurRadius: 48,
        offset: const Offset(0, 18),
      ),
      BoxShadow(
        color: base.withValues(alpha: 0.10 * k),
        blurRadius: 6,
        offset: const Offset(0, 2),
      ),
    ];
  }
}
