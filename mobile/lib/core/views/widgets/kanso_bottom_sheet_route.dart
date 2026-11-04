// Kanso — kanso_bottom_sheet_route.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_motion.dart';
import 'kanso_bottom_sheet.dart';
import 'kanso_scrim.dart';

/// Shows what [builder] makes in a [KansoBottomSheet] over a veil, above the
/// whole app, and completes with what the sheet pops with. A tap on the
/// veil, or the system's back, closes it.
///
/// The sheet is above the pages' own providers: give it what it reads.
///
/// With [restingFraction] the sheet opens that tall and can be pulled up by
/// its handle to its full height (`KansoBottomSheet.restingFraction`).
Future<T?> showKansoBottomSheet<T>(
  BuildContext context, {
  required WidgetBuilder builder,
  double? restingFraction,
}) {
  return Navigator.of(context, rootNavigator: true).push<T>(
    KansoBottomSheetRoute<T>(
      builder: builder,
      restingFraction: restingFraction,
    ),
  );
}

/// The route of [showKansoBottomSheet].
class KansoBottomSheetRoute<T> extends PopupRoute<T> {
  KansoBottomSheetRoute({required this.builder, this.restingFraction});

  final WidgetBuilder builder;
  final double? restingFraction;

  @override
  Duration get transitionDuration => KansoMotion.content;

  @override
  Duration get reverseTransitionDuration => KansoMotion.short;

  @override
  bool get barrierDismissible => true;

  // The veil is drawn with the sheet, in the app's own colour.
  @override
  Color? get barrierColor => null;

  @override
  String? get barrierLabel => null;

  @override
  Widget buildPage(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
  ) {
    return KansoBottomSheet(
      restingFraction: restingFraction,
      child: builder(context),
    );
  }

  @override
  Widget buildTransitions(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    final curved = animation.drive(CurveTween(curve: KansoMotion.enter));
    return Stack(
      children: <Widget>[
        Positioned.fill(
          child: FadeTransition(
            opacity: curved,
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: () => Navigator.of(context).maybePop(),
              child: const KansoScrim(),
            ),
          ),
        ),
        SlideTransition(
          position: Tween<Offset>(
            begin: const Offset(0, 1),
            end: Offset.zero,
          ).animate(curved),
          child: child,
        ),
      ],
    );
  }
}
