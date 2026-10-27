// Kanso — kanso_magnifier.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';

/// The magnifier over the text while a handle or the caret is dragged: a
/// round glass above the line, looking at the line, drawn the same on iOS
/// and Android.
class KansoMagnifier extends StatelessWidget {
  const KansoMagnifier({required this.info, super.key});

  /// Where the finger is and what line it is on, as the field reports.
  final ValueNotifier<MagnifierInfo> info;

  /// How big the glass is across, in points.
  static const double diameter = 88;

  /// How far above the line the glass floats.
  static const double lift = 16;

  /// How much it magnifies.
  static const double scale = 1.25;

  /// The magnifier a field with the Kanso editor asks for.
  static TextMagnifierConfiguration configuration() {
    return TextMagnifierConfiguration(
      magnifierBuilder: (context, controller, info) {
        return KansoMagnifier(info: info);
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final screen = MediaQuery.sizeOf(context);
    return ValueListenableBuilder<MagnifierInfo>(
      valueListenable: info,
      builder: (context, shown, _) {
        final line = shown.currentLineBoundaries;
        final finger = shown.globalGesturePosition;
        final left = (finger.dx - diameter / 2).clamp(
          KansoSpacing.xs,
          screen.width - diameter - KansoSpacing.xs,
        );
        final top = (line.top - diameter - lift).clamp(
          MediaQuery.paddingOf(context).top,
          screen.height,
        );
        // The glass looks at the middle of the line, wherever it floats.
        final looksAt = Offset(
          finger.dx - (left + diameter / 2),
          line.center.dy - (top + diameter / 2),
        );
        return Positioned(
          left: left,
          top: top,
          child: RawMagnifier(
            size: const Size.square(diameter),
            magnificationScale: scale,
            focalPointOffset: looksAt,
            decoration: MagnifierDecoration(
              shape: CircleBorder(side: BorderSide(color: colors.divider)),
              shadows: KansoShadows.floating(Theme.brightnessOf(context)),
            ),
          ),
        );
      },
    );
  }
}
