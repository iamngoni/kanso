// Kanso — kanso_icon_set_scope.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../models/data/kanso_icon_set.dart';

/// Makes [iconSet] the icon theme every `KansoIcon` below it draws in.
///
/// The app puts one above its routes, fed by the `AppearanceBloc`. Without
/// one above, icons are Kanso's own Lucide icons.
class KansoIconSetScope extends InheritedWidget {
  const KansoIconSetScope({
    required this.iconSet,
    required super.child,
    super.key,
  });

  final KansoIconSet iconSet;

  /// The icon theme in use below [context]: Lucide where no scope is above.
  static KansoIconSet of(BuildContext context) {
    return context
            .dependOnInheritedWidgetOfExactType<KansoIconSetScope>()
            ?.iconSet ??
        KansoIconSet.lucide;
  }

  @override
  bool updateShouldNotify(KansoIconSetScope oldWidget) =>
      oldWidget.iconSet != iconSet;
}
