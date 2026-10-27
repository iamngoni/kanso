// Kanso — kanso_pressable.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_motion.dart';

/// Makes its [child] tappable, and dims it while a finger is on it.
///
/// There is no ink splash: the feedback is the same on every platform. With
/// no [onTap] the child is drawn as it is and does not react. Give
/// [semanticLabel] to a child that has no text of its own.
class KansoPressable extends StatefulWidget {
  const KansoPressable({
    required this.child,
    this.onTap,
    this.onLongPress,
    this.semanticLabel,
    this.scale = 1,
    super.key,
  });

  final Widget child;
  final VoidCallback? onTap;

  /// What a press that is held does.
  final VoidCallback? onLongPress;
  final String? semanticLabel;

  /// How far it shrinks while pressed: [KansoEffects.pressScale] for a
  /// button, 1 (none) for a row that spans the page.
  final double scale;

  @override
  State<KansoPressable> createState() => _KansoPressableState();
}

class _KansoPressableState extends State<KansoPressable> {
  final ValueNotifier<bool> pressed = ValueNotifier<bool>(false);

  @override
  void dispose() {
    pressed.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final onTap = widget.onTap;
    return Semantics(
      button: onTap != null,
      enabled: onTap != null,
      label: widget.semanticLabel,
      onLongPress: widget.onLongPress,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        onLongPress: widget.onLongPress,
        onTapDown: onTap == null ? null : (_) => pressed.value = true,
        onTapUp: onTap == null ? null : (_) => pressed.value = false,
        onTapCancel: onTap == null ? null : () => pressed.value = false,
        child: ValueListenableBuilder<bool>(
          valueListenable: pressed,
          builder: (context, isPressed, child) {
            final duration = KansoMotion.resolve(context, KansoMotion.fast);
            return AnimatedOpacity(
              opacity: isPressed ? KansoEffects.pressed : 1,
              duration: duration,
              curve: Curves.easeOut,
              child: AnimatedScale(
                scale: isPressed ? widget.scale : 1,
                duration: duration,
                curve: Curves.easeOut,
                child: child,
              ),
            );
          },
          child: widget.child,
        ),
      ),
    );
  }
}
