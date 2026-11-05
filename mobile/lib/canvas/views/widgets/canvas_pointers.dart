// Kanso — canvas_pointers.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../../core/utils/context_extensions.dart';
import '../../utils/canvas_viewport.dart';

/// The dot's size, and the ring that lifts it off the page.
const double _dot = 14;

/// The longest a name chip grows before it is cut.
const double _chipMaxWidth = 132;

/// The other people's pointers on a shared canvas: a dot where each one is
/// and their name beside it, in the colour the person is given everywhere
/// in Kanso. The marks sit above the canvas at a fixed size, whatever the
/// zoom, and glide to where the pointer moves. A mark near the right edge
/// puts its name on the left, so it never leaves the screen.
class CanvasPointers extends StatelessWidget {
  const CanvasPointers({
    required this.pointers,
    required this.viewport,
    required this.size,
    required this.dark,
    required this.ring,
    super.key,
  });

  final List<CanvasPointerRow> pointers;
  final CanvasViewport viewport;
  final Size size;
  final bool dark;

  /// The page's colour, for the ring round the dot.
  final Color ring;

  @override
  Widget build(BuildContext context) {
    final marks = <Widget>[];
    for (final pointer in pointers) {
      final at = viewport.toScreen(Offset(pointer.x, pointer.y), size);
      if (at.dx < -40 ||
          at.dy < -40 ||
          at.dx > size.width + 40 ||
          at.dy > size.height + 40) {
        continue;
      }
      marks.add(
        AnimatedPositioned(
          key: ValueKey<String>('pointer-${pointer.accountId}'),
          duration: const Duration(milliseconds: 140),
          curve: Curves.easeOut,
          left: at.dx - _dot / 2,
          top: at.dy - _dot / 2,
          child: _PointerMark(
            pointer: pointer,
            dark: dark,
            ring: ring,
            nameOnLeft: at.dx > size.width - 150,
          ),
        ),
      );
    }
    return IgnorePointer(
      child: Stack(clipBehavior: Clip.none, children: marks),
    );
  }
}

class _PointerMark extends StatelessWidget {
  const _PointerMark({
    required this.pointer,
    required this.dark,
    required this.ring,
    required this.nameOnLeft,
  });

  final CanvasPointerRow pointer;
  final bool dark;
  final Color ring;
  final bool nameOnLeft;

  @override
  Widget build(BuildContext context) {
    final color = pointerColor(dark ? pointer.colourDark : pointer.colourLight);
    final onColor =
        ThemeData.estimateBrightnessForColor(color) == Brightness.dark
        ? Colors.white
        : const Color(0xFF14161A);
    final label = pointer.name.isEmpty ? '·' : pointer.name;
    final dot = Container(
      width: _dot,
      height: _dot,
      decoration: BoxDecoration(
        color: color,
        shape: BoxShape.circle,
        border: Border.all(color: ring, width: 2),
        boxShadow: <BoxShadow>[
          BoxShadow(
            color: Colors.black.withValues(alpha: dark ? 0.5 : 0.22),
            blurRadius: 4,
            offset: const Offset(0, 1),
          ),
        ],
      ),
    );
    final chip = Container(
      constraints: const BoxConstraints(maxWidth: _chipMaxWidth),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(10),
        boxShadow: <BoxShadow>[
          BoxShadow(
            color: Colors.black.withValues(alpha: dark ? 0.45 : 0.18),
            blurRadius: 4,
            offset: const Offset(0, 1),
          ),
        ],
      ),
      child: Text(
        label,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: context.typography.meta.copyWith(
          color: onColor,
          fontWeight: FontWeight.w600,
          height: 1.3,
        ),
      ),
    );
    return Semantics(
      label: context.l10n.canvasPointerLabel(pointer.name),
      child: SizedBox(
        width: _dot,
        height: _dot,
        child: Stack(
          clipBehavior: Clip.none,
          children: <Widget>[
            Positioned(left: 0, top: 0, child: dot),
            Positioned(
              top: _dot - 2,
              left: nameOnLeft ? null : _dot - 2,
              right: nameOnLeft ? _dot - 2 : null,
              child: chip,
            ),
          ],
        ),
      ),
    );
  }
}

/// A pointer's colour from the `#rrggbb` the library gives; the page's text
/// grey when it is not one.
Color pointerColor(String hex) {
  final value = int.tryParse(hex.replaceFirst('#', ''), radix: 16);
  if (value == null || hex.length != 7) {
    return const Color(0xFF7A808A);
  }
  return Color(0xFF000000 | value);
}
