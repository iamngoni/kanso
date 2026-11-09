// Kanso — kanso_page.dart.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../utils/context_extensions.dart';

/// A page body with explicit Kanso-owned background and keyboard behavior.
///
/// The body owns its own safe areas. When [resizeToAvoidBottomInset] is
/// true, the body gives up the space occupied by the software keyboard
/// while keeping the original [MediaQuery] available to descendants.
class KansoPage extends StatelessWidget {
  const KansoPage({
    required this.body,
    this.backgroundColor,
    this.resizeToAvoidBottomInset = true,
    super.key,
  });

  final Widget body;
  final Color? backgroundColor;
  final bool resizeToAvoidBottomInset;

  @override
  Widget build(BuildContext context) {
    final keyboard = resizeToAvoidBottomInset
        ? MediaQuery.viewInsetsOf(context).bottom
        : 0.0;
    // Always the same widgets above the body: wrapping it only while the
    // keyboard is up re-made every field in it as the keyboard began to
    // open, and the field lost its focus and the keyboard went again.
    final content = AnimatedPadding(
      duration: const Duration(milliseconds: 150),
      curve: Curves.easeOut,
      padding: EdgeInsets.only(bottom: keyboard),
      child: body,
    );
    return ColoredBox(
      color: backgroundColor ?? context.colors.editor,
      child: DefaultTextStyle(
        style: context.typography.body,
        child: MediaQuery.removeViewInsets(
          context: context,
          removeBottom: resizeToAvoidBottomInset,
          child: content,
        ),
      ),
    );
  }
}
