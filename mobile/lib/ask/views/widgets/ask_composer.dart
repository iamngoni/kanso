// Kanso — ask_composer.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_text_field.dart';

/// The question field at the foot of the conversation, with a button that
/// asks, and that stops the answer while one is being written.
class AskComposer extends StatefulWidget {
  const AskComposer({
    required this.busy,
    required this.onAsk,
    required this.onStop,
    super.key,
  });

  /// Whether a question is being answered.
  final bool busy;
  final ValueChanged<String> onAsk;
  final VoidCallback onStop;

  /// The most lines the field grows to before it scrolls.
  static const int maxLines = 5;

  @override
  State<AskComposer> createState() => _AskComposerState();
}

class _AskComposerState extends State<AskComposer> {
  final TextEditingController controller = TextEditingController();

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  void send() {
    final question = controller.text.trim();
    if (question.isEmpty || widget.busy) {
      return;
    }
    controller.clear();
    widget.onAsk(question);
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.app,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          const KansoDivider(),
          Padding(
            padding: EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.xs,
              KansoSpacing.xs,
              bottom + KansoSpacing.xs,
            ),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: <Widget>[
                Expanded(
                  child: Container(
                    constraints: const BoxConstraints(
                      minHeight: KansoSizes.touchTarget,
                    ),
                    alignment: AlignmentDirectional.centerStart,
                    padding: const EdgeInsets.symmetric(
                      horizontal: KansoSpacing.sm,
                      vertical: KansoSpacing.sm,
                    ),
                    decoration: BoxDecoration(
                      color: colors.fill,
                      borderRadius: KansoRadii.mediumBorder,
                    ),
                    child: KansoTextField(
                      controller: controller,
                      hint: l10n.askHint,
                      semanticLabel: l10n.askHint,
                      maxLines: AskComposer.maxLines,
                      autocorrect: true,
                      textInputAction: TextInputAction.newline,
                    ),
                  ),
                ),
                ListenableBuilder(
                  listenable: controller,
                  builder: (context, _) {
                    final canSend =
                        controller.text.trim().isNotEmpty && !widget.busy;
                    final stops = widget.busy;
                    return KansoPressable(
                      onTap: stops ? widget.onStop : (canSend ? send : null),
                      semanticLabel: stops ? l10n.askStop : l10n.askSend,
                      child: SizedBox.square(
                        dimension: KansoSizes.touchTarget + KansoSpacing.xs,
                        child: Center(
                          child: Container(
                            width: KansoSizes.touchTarget - KansoSpacing.xxs,
                            height: KansoSizes.touchTarget - KansoSpacing.xxs,
                            decoration: BoxDecoration(
                              color: stops || canSend
                                  ? colors.button
                                  : colors.fill,
                              shape: BoxShape.circle,
                            ),
                            child: Center(
                              child: KansoIcon(
                                stops ? KansoIcons.square : KansoIcons.arrowUp,
                                size: stops
                                    ? KansoSizes.iconSmall
                                    : KansoSizes.icon,
                                color: stops || canSend
                                    ? colors.buttonText
                                    : colors.textMuted,
                              ),
                            ),
                          ),
                        ),
                      ),
                    );
                  },
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
