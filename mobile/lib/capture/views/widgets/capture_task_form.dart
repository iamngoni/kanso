// Kanso — capture_task_form.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart' show TextInputAction, TextInputType;
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart' show CaptureTarget;
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../editor/utils/when_date_order.dart';
import '../../models/data/capture_mode.dart';
import '../../state/capture_task/capture_task_cubit.dart';

/// The capture sheet in Task mode: a field for one task, the date and repeat
/// at its end read as they will be written (`Fri 13 Nov, 15:00 · every
/// week`) under it, where the task will go, and Add task. A task that is
/// written empties the field for the next one and says where it went.
/// It needs the [CaptureTaskCubit] above it.
class CaptureTaskForm extends StatefulWidget {
  const CaptureTaskForm({super.key});

  @override
  State<CaptureTaskForm> createState() => _CaptureTaskFormState();
}

class _CaptureTaskFormState extends State<CaptureTaskForm> {
  final TextEditingController _text = TextEditingController();

  @override
  void dispose() {
    _text.dispose();
    super.dispose();
  }

  bool get _monthFirst => monthFirstFor(Localizations.localeOf(context));

  Future<void> _submit() async {
    final cubit = context.read<CaptureTaskCubit>();
    if (_text.text.trim().isEmpty) {
      return;
    }
    final written = await cubit.submitted(_text.text, monthFirst: _monthFirst);
    if (written && mounted) {
      _text.clear();
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        0,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          DecoratedBox(
            decoration: BoxDecoration(
              color: colors.elevated,
              borderRadius: KansoRadii.mediumBorder,
              border: Border.all(color: colors.divider),
            ),
            child: Padding(
              padding: const EdgeInsets.symmetric(
                horizontal: KansoSpacing.sm,
                vertical: KansoSpacing.sm,
              ),
              child: KansoTextField(
                controller: _text,
                hint: l10n.captureTaskHint,
                semanticLabel: l10n.captureTaskFieldSemantics,
                autofocus: true,
                autocorrect: true,
                minLines: 2,
                maxLines: 4,
                keyboardType: TextInputType.multiline,
                textInputAction: TextInputAction.newline,
                onChanged: (text) => context
                    .read<CaptureTaskCubit>()
                    .textChanged(text, monthFirst: _monthFirst),
              ),
            ),
          ),
          const SizedBox(height: KansoSpacing.sm),
          BlocBuilder<CaptureTaskCubit, CaptureTaskState>(
            builder: (context, state) {
              return AnimatedSize(
                duration: const Duration(milliseconds: 140),
                curve: Curves.easeOutCubic,
                alignment: Alignment.topCenter,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    if (state.hint case final hint?)
                      Padding(
                        padding: const EdgeInsets.only(bottom: KansoSpacing.xs),
                        child: _HintChip(label: hint),
                      ),
                    _StatusLine(state: state),
                  ],
                ),
              );
            },
          ),
          const SizedBox(height: KansoSpacing.sm),
          BlocBuilder<CaptureTaskCubit, CaptureTaskState>(
            buildWhen: (before, after) => before.status != after.status,
            builder: (context, state) {
              return ListenableBuilder(
                listenable: _text,
                builder: (context, _) {
                  final ready =
                      _text.text.trim().isNotEmpty &&
                      state.status != CaptureTaskStatus.saving;
                  return KansoButton(
                    label: l10n.captureTaskAdd,
                    icon: KansoIcons.plus,
                    onPressed: ready ? _submit : null,
                  );
                },
              );
            },
          ),
        ],
      ),
    );
  }
}

/// The date and repeat as they will be written, in an accent wash.
class _HintChip extends StatelessWidget {
  const _HintChip({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      container: true,
      label: context.l10n.captureTaskHintSemantics(label),
      child: ExcludeSemantics(
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: colors.accent.withValues(alpha: 0.12),
            borderRadius: KansoRadii.pillBorder,
            border: Border.all(color: colors.accent.withValues(alpha: 0.28)),
          ),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xs,
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                KansoIcon(
                  label.contains('·')
                      ? KansoIcons.refreshCw
                      : KansoIcons.calendar,
                  size: KansoSizes.iconSmall,
                  color: colors.accent,
                ),
                const SizedBox(width: KansoSpacing.xs),
                Flexible(
                  child: Text(
                    label,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: context.typography.label.copyWith(
                      color: colors.text,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// Where the task will go, or where it went, or why it did not.
class _StatusLine extends StatelessWidget {
  const _StatusLine({required this.state});

  final CaptureTaskState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final place = switch (state.target) {
      CaptureTarget.dailyNote => l10n.captureTaskPlaceDaily,
      CaptureTarget.inbox => l10n.captureTaskPlaceInbox,
    };
    final (text, color, icon) = switch (state.status) {
      CaptureTaskStatus.saved => (
        l10n.captureTaskAdded(place),
        colors.success,
        KansoIcons.check,
      ),
      CaptureTaskStatus.locked => (
        l10n.captureTaskLocked,
        colors.danger,
        KansoIcons.lock,
      ),
      CaptureTaskStatus.nothing => (
        l10n.captureTaskNothing,
        colors.danger,
        null,
      ),
      CaptureTaskStatus.failed => (l10n.captureTaskFailed, colors.danger, null),
      _ => (l10n.captureTaskGoesTo(place), colors.textMuted, null),
    };
    return Semantics(
      liveRegion: true,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          if (icon != null) ...<Widget>[
            KansoIcon(icon, size: KansoSizes.iconSmall, color: color),
            const SizedBox(width: KansoSpacing.xs),
          ],
          Expanded(
            child: Text(
              text,
              style: context.typography.meta.copyWith(color: color),
            ),
          ),
        ],
      ),
    );
  }
}
