// Kanso — agent_session_actions.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_dialog.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../../core/views/widgets/kanso_text_field.dart';

/// What the session menu can do to a session.
enum AgentSessionAction { rename, delete }

/// Opens the menu of what can be done to a session, at [at], and completes
/// with the choice, or null.
Future<AgentSessionAction?> showAgentSessionMenu(
  BuildContext context,
  Offset at,
) {
  final l10n = context.l10n;
  return showKansoMenu<AgentSessionAction>(
    context,
    at,
    <KansoMenuEntry<AgentSessionAction>>[
      KansoMenuEntry<AgentSessionAction>(
        AgentSessionAction.rename,
        l10n.agentSessionRename,
      ),
      KansoMenuEntry<AgentSessionAction>(
        AgentSessionAction.delete,
        l10n.agentSessionDelete,
        divider: true,
      ),
    ],
  );
}

/// Asks for a session's new name, starting from [current]. Completes with the
/// trimmed name, or null when the person cancelled or left it blank.
Future<String?> askAgentSessionTitle(BuildContext context, String current) {
  return showKansoDialog<String>(
    context,
    builder: (_) => _RenameDialog(current: current),
  );
}

/// True only when the person confirms deleting the session.
Future<bool> confirmDeleteAgentSession(BuildContext context) {
  final l10n = context.l10n;
  return showKansoConfirm(
    context,
    title: l10n.agentSessionDeleteTitle,
    body: l10n.agentSessionDeleteBody,
    cancelLabel: l10n.agentTasksReadAllCancel,
    confirmLabel: l10n.agentSessionDelete,
    danger: true,
  );
}

class _RenameDialog extends StatefulWidget {
  const _RenameDialog({required this.current});

  final String current;

  @override
  State<_RenameDialog> createState() => _RenameDialogState();
}

class _RenameDialogState extends State<_RenameDialog> {
  late final TextEditingController _name = TextEditingController(
    text: widget.current,
  );

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  void _save() {
    final name = _name.text.trim();
    Navigator.of(context).pop(name.isEmpty ? null : name);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoDialog(
      title: l10n.agentSessionRenameTitle,
      actions: <Widget>[
        KansoButton(label: l10n.agentSessionRenameSave, onPressed: _save),
        const SizedBox(height: KansoSpacing.xs),
        KansoButton(
          label: l10n.agentTasksReadAllCancel,
          variant: KansoButtonVariant.secondary,
          onPressed: () => Navigator.of(context).pop(),
        ),
      ],
      child: KansoTextField(
        controller: _name,
        hint: l10n.agentSessionRenameHint,
        semanticLabel: l10n.agentSessionRenameHint,
        autofocus: true,
        onSubmitted: (_) => _save(),
      ),
    );
  }
}
