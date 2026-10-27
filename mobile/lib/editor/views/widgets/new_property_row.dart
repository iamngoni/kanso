// Kanso — new_property_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_text_input.dart';

/// The row where a property is named and given a value before it is added:
/// a name, a value, and buttons to add it or give up.
class NewPropertyRow extends StatefulWidget {
  const NewPropertyRow({
    required this.onAdd,
    required this.onCancel,
    super.key,
  });

  /// The property was named (it is not empty) and given a value.
  final void Function(String key, String value) onAdd;

  final VoidCallback onCancel;

  @override
  State<NewPropertyRow> createState() => _NewPropertyRowState();
}

class _NewPropertyRowState extends State<NewPropertyRow> {
  final TextEditingController key = TextEditingController();
  final TextEditingController value = TextEditingController();
  final FocusNode keyFocus = FocusNode();
  final FocusNode valueFocus = FocusNode();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        keyFocus.requestFocus();
      }
    });
  }

  /// Adds the property, or goes back to its name when it has none.
  void add() {
    final name = key.text.trim();
    if (name.isEmpty) {
      keyFocus.requestFocus();
      return;
    }
    widget.onAdd(name, value.text);
  }

  @override
  void dispose() {
    valueFocus.dispose();
    keyFocus.dispose();
    value.dispose();
    key.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
      child: Row(
        children: <Widget>[
          Expanded(
            flex: 2,
            child: KansoTextInput(
              controller: key,
              focusNode: keyFocus,
              hint: l10n.propertiesName,
              semanticLabel: l10n.propertiesName,
              textInputAction: TextInputAction.next,
              autocorrect: false,
              onSubmitted: (_) => valueFocus.requestFocus(),
            ),
          ),
          const SizedBox(width: KansoSpacing.xs),
          Expanded(
            flex: 3,
            child: KansoTextInput(
              controller: value,
              focusNode: valueFocus,
              hint: l10n.propertiesValue,
              semanticLabel: l10n.propertiesValue,
              autocorrect: false,
              onSubmitted: (_) => add(),
            ),
          ),
          KansoIconButton(
            icon: KansoIcons.check,
            semanticLabel: l10n.propertiesConfirmAdd,
            onPressed: add,
          ),
          KansoIconButton(
            icon: KansoIcons.close,
            semanticLabel: l10n.propertiesCancelAdd,
            onPressed: widget.onCancel,
          ),
        ],
      ),
    );
  }
}
