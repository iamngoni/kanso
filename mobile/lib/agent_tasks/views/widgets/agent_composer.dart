// Kanso — agent_composer.dart
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

/// The message field at the foot of a session: the person's follow-up goes
/// to the same agent conversation. While the AI works it shows Stop in place
/// of Send; when the session can't take a message it says why instead of
/// showing a field.
class AgentComposer extends StatefulWidget {
  const AgentComposer({
    required this.canSend,
    required this.working,
    required this.sending,
    required this.disabledReason,
    required this.onSend,
    required this.onStop,
    super.key,
  });

  /// Whether a message can be sent now.
  final bool canSend;

  /// Whether the AI is working: the button stops the turn.
  final bool working;

  /// Whether a message is on its way.
  final bool sending;

  /// Why nothing can be sent, said where the field would be; null when the
  /// field is shown.
  final String? disabledReason;
  final ValueChanged<String> onSend;
  final VoidCallback onStop;

  /// The most lines the field grows to before it scrolls.
  static const int maxLines = 5;

  @override
  State<AgentComposer> createState() => _AgentComposerState();
}

class _AgentComposerState extends State<AgentComposer> {
  final TextEditingController controller = TextEditingController();

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  void send() {
    final text = controller.text.trim();
    if (text.isEmpty || !widget.canSend || widget.sending) {
      return;
    }
    controller.clear();
    widget.onSend(text);
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    final reason = widget.disabledReason;
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
                    child: reason != null
                        ? Text(
                            reason,
                            style: context.typography.secondary.copyWith(
                              color: colors.textMuted,
                            ),
                          )
                        : KansoTextField(
                            controller: controller,
                            hint: l10n.agentSessionComposerHint,
                            semanticLabel: l10n.agentSessionComposerHint,
                            maxLines: AgentComposer.maxLines,
                            autocorrect: true,
                            textInputAction: TextInputAction.newline,
                          ),
                  ),
                ),
                ListenableBuilder(
                  listenable: controller,
                  builder: (context, _) {
                    final ready =
                        widget.canSend &&
                        controller.text.trim().isNotEmpty &&
                        !widget.sending;
                    final stops = widget.working && !ready;
                    final active = stops || ready;
                    return KansoPressable(
                      onTap: stops ? widget.onStop : (ready ? send : null),
                      semanticLabel: stops
                          ? l10n.agentTaskStop
                          : l10n.agentSessionComposerSend,
                      child: SizedBox.square(
                        dimension: KansoSizes.touchTarget + KansoSpacing.xs,
                        child: Center(
                          child: Container(
                            width: KansoSizes.touchTarget - KansoSpacing.xxs,
                            height: KansoSizes.touchTarget - KansoSpacing.xxs,
                            decoration: BoxDecoration(
                              color: active ? colors.button : colors.fill,
                              shape: BoxShape.circle,
                            ),
                            child: Center(
                              child: KansoIcon(
                                stops ? KansoIcons.square : KansoIcons.arrowUp,
                                size: stops
                                    ? KansoSizes.iconSmall
                                    : KansoSizes.icon,
                                color: active
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
