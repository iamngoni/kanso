// Kanso — kanso_bottom_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../configs/kanso_colors.dart';
import '../../configs/kanso_motion.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_shadows.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';

/// The panel of a sheet that rises from the foot of the screen: a rounded
/// surface that is as tall as its [child] and at most nine tenths of the
/// screen, no wider than a tablet's sheet and centred when the screen is
/// wider. It takes the home indicator's height as well, moves above the
/// keyboard, dismisses the keyboard when its body or a scrolling pane is
/// touched, and goes down when its grab bar is dragged down.
///
/// With [restingFraction] the panel opens that tall, whatever its content,
/// and a drag up on the handle (or a tap on it) opens it to its full height;
/// a drag down brings it back before the next one closes it.
///
/// The veil behind it and the way it moves in are the route's
/// (`showKansoBottomSheet`).
class KansoBottomSheet extends StatefulWidget {
  const KansoBottomSheet({
    required this.child,
    this.restingFraction,
    super.key,
  });

  final Widget child;

  /// How much of the available height a sheet that can be expanded opens at;
  /// none for a sheet as tall as its content.
  final double? restingFraction;

  /// How much of the available screen height the panel may take.
  static const double heightFraction = 0.92;

  /// How far the handle is dragged before letting go closes the sheet.
  static const double dismissDistance = 120;

  /// How fast a flick closes the sheet, in points per second.
  static const double dismissVelocity = 700;

  /// How far the handle is dragged up before letting go expands the sheet.
  static const double expandDistance = 40;

  /// How fast an upward flick expands the sheet, in points per second.
  static const double expandVelocity = 500;

  /// The grab handle, for tests and for a steady place to drag from.
  static const Key handleKey = ValueKey<String>('kanso-sheet-handle');

  @override
  State<KansoBottomSheet> createState() => _KansoBottomSheetState();
}

class _KansoBottomSheetState extends State<KansoBottomSheet> {
  final ValueNotifier<double> _drag = ValueNotifier<double>(0);
  bool _expanded = false;
  double _lift = 0;

  bool get _expandable => widget.restingFraction != null;

  @override
  void dispose() {
    // A field in a sheet must not keep the keyboard alive after the route
    // goes. This also covers a system back or a veil tap.
    FocusManager.instance.primaryFocus?.unfocus();
    _drag.dispose();
    super.dispose();
  }

  void _dismissKeyboard() {
    FocusScope.of(context).unfocus();
  }

  void _dragStarted(DragStartDetails details) {
    _dismissKeyboard();
  }

  void _dragged(DragUpdateDetails details) {
    final delta = details.primaryDelta ?? 0;
    if (delta < 0 && _expandable && !_expanded) {
      _lift -= delta;
    }
    if (delta <= 0) return;
    _drag.value += delta;
  }

  void _dragEnded(DragEndDetails details) {
    final velocity = details.primaryVelocity ?? 0;
    final lifted = _lift;
    _lift = 0;
    if (_expandable &&
        !_expanded &&
        (lifted > KansoBottomSheet.expandDistance ||
            velocity < -KansoBottomSheet.expandVelocity)) {
      _drag.value = 0;
      setState(() => _expanded = true);
      return;
    }
    final far = _drag.value > KansoBottomSheet.dismissDistance;
    final fast = velocity > KansoBottomSheet.dismissVelocity;
    if (_expanded && (far || fast)) {
      // Down from the full height settles at the resting one.
      _drag.value = 0;
      setState(() => _expanded = false);
      return;
    }
    if (far || fast) {
      Navigator.of(context).maybePop();
      return;
    }
    _drag.value = 0;
  }

  void _toggleExpanded() {
    if (_expandable) {
      setState(() => _expanded = !_expanded);
    }
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return LayoutBuilder(
      builder: (context, constraints) {
        final media = MediaQuery.of(context);
        final keyboard = media.viewInsets.bottom;
        final available = (constraints.maxHeight - keyboard).clamp(
          0.0,
          constraints.maxHeight,
        );
        // Never into the status bar or the notch, whatever the fraction.
        final top = media.padding.top + KansoSpacing.xs;
        final maxHeight = math.min(
          available * KansoBottomSheet.heightFraction,
          math.max<double>(0, available - top),
        );
        final resting = widget.restingFraction;
        final height = resting == null
            ? null
            : _expanded
            ? maxHeight
            : math.min(maxHeight, available * resting);
        return ValueListenableBuilder<double>(
          valueListenable: _drag,
          builder: (context, drag, _) {
            final settling = drag == 0;
            final duration = KansoMotion.resolve(
              context,
              settling ? KansoMotion.short : const Duration(milliseconds: 1),
            );
            final body = NotificationListener<ScrollNotification>(
              onNotification: (notification) {
                // Only a finger dragging the content puts the keyboard
                // away. The scroll that brings the focused field into view
                // as the keyboard opens starts without one, and dismissed
                // the keyboard it was made for.
                if (notification is ScrollStartNotification &&
                    notification.dragDetails != null) {
                  _dismissKeyboard();
                }
                return false;
              },
              child: widget.child,
            );
            return Align(
              alignment: Alignment.bottomCenter,
              child: AnimatedPadding(
                duration: KansoMotion.resolve(context, KansoMotion.short),
                curve: KansoMotion.standard,
                padding: EdgeInsets.only(bottom: keyboard),
                child: AnimatedContainer(
                  duration: duration,
                  curve: KansoMotion.standard,
                  height: height,
                  transform: Matrix4.translationValues(0, drag, 0),
                  child: ConstrainedBox(
                    constraints: BoxConstraints(
                      maxWidth: KansoSizes.sheetWidth,
                      maxHeight: maxHeight,
                    ),
                    child: DecoratedBox(
                      decoration: BoxDecoration(
                        color: colors.app,
                        borderRadius: const BorderRadius.vertical(
                          top: Radius.circular(KansoRadii.large),
                        ),
                        border: Border.all(color: colors.divider),
                        boxShadow: KansoShadows.sheet(context.brightness),
                      ),
                      child: ClipRRect(
                        borderRadius: const BorderRadius.vertical(
                          top: Radius.circular(KansoRadii.large),
                        ),
                        child: SafeArea(
                          top: false,
                          child: Column(
                            mainAxisSize: MainAxisSize.min,
                            children: <Widget>[
                              _SheetGrabBar(
                                colors: colors,
                                onDragStart: _dragStarted,
                                onDragUpdate: _dragged,
                                onDragEnd: _dragEnded,
                                onTap: _expandable ? _toggleExpanded : null,
                              ),
                              if (resting == null)
                                Flexible(child: body)
                              else
                                Expanded(child: body),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            );
          },
        );
      },
    );
  }
}

/// The full-width grab area at the top of a sheet. A wide target is easier
/// to catch than the five-point bar alone, and it does not steal vertical
/// drags from the content below it.
class _SheetGrabBar extends StatelessWidget {
  const _SheetGrabBar({
    required this.colors,
    required this.onDragStart,
    required this.onDragUpdate,
    required this.onDragEnd,
    this.onTap,
  });

  final KansoColors colors;
  final GestureDragStartCallback onDragStart;
  final GestureDragUpdateCallback onDragUpdate;
  final GestureDragEndCallback onDragEnd;

  /// Taps on the handle, when it expands the sheet.
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      key: KansoBottomSheet.handleKey,
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      onVerticalDragStart: onDragStart,
      onVerticalDragUpdate: onDragUpdate,
      onVerticalDragEnd: onDragEnd,
      child: Semantics(
        button: true,
        label: context.l10n.sheetDismiss,
        child: SizedBox(
          height: 32,
          width: double.infinity,
          child: Center(
            child: Container(
              width: 40,
              height: 5,
              decoration: BoxDecoration(
                color: colors.textMuted.withValues(alpha: 0.55),
                borderRadius: KansoRadii.pillBorder,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
