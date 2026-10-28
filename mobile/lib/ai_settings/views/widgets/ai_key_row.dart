// Kanso — ai_key_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../models/data/ai_key.dart';

/// The API key: a hidden field and Save key while none is saved, and
/// Saved with Remove once one is. The key is never shown again.
class AiKeyRow extends StatefulWidget {
  const AiKeyRow({
    required this.label,
    required this.saved,
    required this.busy,
    required this.onSaved,
    required this.onRemoved,
    super.key,
  });

  final String label;

  /// Whether a key is saved for the provider.
  final bool saved;

  /// Whether a key is being saved or removed now.
  final bool busy;
  final ValueChanged<AiKey> onSaved;
  final VoidCallback onRemoved;

  @override
  State<AiKeyRow> createState() => _AiKeyRowState();
}

class _AiKeyRowState extends State<AiKeyRow> {
  final TextEditingController controller = TextEditingController();

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  void save() {
    final typed = controller.text.trim();
    if (typed.isEmpty) {
      return;
    }
    // The key leaves the field as soon as it is sent.
    controller.clear();
    widget.onSaved(AiKey(typed));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final label = Text(
      widget.label,
      style: context.typography.label.copyWith(color: colors.textSecondary),
    );
    return Padding(
      padding: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.md,
        vertical: KansoSpacing.sm,
      ),
      child: widget.saved
          ? Row(
              children: <Widget>[
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: <Widget>[
                      label,
                      const SizedBox(height: KansoSpacing.xxs),
                      Text(l10n.aiKeySaved, style: context.typography.body),
                    ],
                  ),
                ),
                KansoButton(
                  label: l10n.aiKeyRemove,
                  variant: KansoButtonVariant.danger,
                  expanded: false,
                  compact: true,
                  onPressed: widget.busy ? null : widget.onRemoved,
                ),
              ],
            )
          : Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                label,
                const SizedBox(height: KansoSpacing.xxs),
                Row(
                  children: <Widget>[
                    Expanded(
                      child: KansoTextField(
                        controller: controller,
                        hint: l10n.aiKeyHint,
                        semanticLabel: widget.label,
                        obscure: true,
                        onSubmitted: (_) => save(),
                      ),
                    ),
                    const SizedBox(width: KansoSpacing.xs),
                    ListenableBuilder(
                      listenable: controller,
                      builder: (context, _) => KansoButton(
                        label: l10n.aiKeySave,
                        variant: KansoButtonVariant.secondary,
                        expanded: false,
                        compact: true,
                        onPressed: widget.busy || controller.text.trim().isEmpty
                            ? null
                            : save,
                      ),
                    ),
                  ],
                ),
              ],
            ),
    );
  }
}
