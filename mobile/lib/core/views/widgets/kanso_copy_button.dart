// Kanso — kanso_copy_button.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../configs/kanso_icons.dart';
import '../../utils/context_extensions.dart';
import 'kanso_button.dart';

/// A button that puts [text] on the clipboard and says "Copied" after.
///
/// [label] is what it says before; it keeps its size when it changes, as
/// both are short.
class KansoCopyButton extends StatefulWidget {
  const KansoCopyButton({
    required this.text,
    this.label,
    this.variant = KansoButtonVariant.secondary,
    this.expanded = true,
    super.key,
  });

  final String text;

  /// The button's words; "Copy" when null.
  final String? label;
  final KansoButtonVariant variant;
  final bool expanded;

  @override
  State<KansoCopyButton> createState() => _KansoCopyButtonState();
}

class _KansoCopyButtonState extends State<KansoCopyButton> {
  final ValueNotifier<bool> copied = ValueNotifier<bool>(false);

  @override
  void dispose() {
    copied.dispose();
    super.dispose();
  }

  Future<void> copy() async {
    await Clipboard.setData(ClipboardData(text: widget.text));
    copied.value = true;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return ValueListenableBuilder<bool>(
      valueListenable: copied,
      builder: (context, done, _) {
        return KansoButton(
          label: done ? l10n.accountCopied : widget.label ?? l10n.accountCopy,
          icon: done ? KansoIcons.check : KansoIcons.copy,
          variant: widget.variant,
          expanded: widget.expanded,
          onPressed: copy,
        );
      },
    );
  }
}
