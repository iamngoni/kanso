// Kanso — send_feedback_pane.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../settings/views/widgets/discovery_setting_switch_row.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/data/feedback_draft.dart';
import '../../models/data/feedback_kind.dart';
import '../../models/data/feedback_result.dart';
import '../../state/send_feedback/send_feedback_bloc.dart';
import '../../utils/feedback_labels.dart';

/// Settings › Send feedback: the kind, the words, an optional screenshot, the
/// app info (on, and shown exactly as sent) and an optional credit name.
/// Send is pinned at the bottom and stays off until there are 10 characters.
/// It needs a [SendFeedbackBloc] above it. **No note is ever read.**
class SendFeedbackPane extends StatefulWidget {
  const SendFeedbackPane({super.key});

  @override
  State<SendFeedbackPane> createState() => _SendFeedbackPaneState();
}

class _SendFeedbackPaneState extends State<SendFeedbackPane> {
  final TextEditingController _text = TextEditingController();
  final TextEditingController _name = TextEditingController();

  @override
  void dispose() {
    _text.dispose();
    _name.dispose();
    super.dispose();
  }

  /// A report that was sent or saved closes the page, and a toast says so
  /// with a way to My feedback.
  void _done(BuildContext context, FeedbackSendResult result) {
    final l10n = context.l10n;
    final message = l10n.feedbackOutcomeMessage(result);
    final router = GoRouter.of(context);
    context.pop();
    KansoFeedback.show(
      context,
      message,
      tone: result.outcome == FeedbackOutcome.sent
          ? KansoFeedbackTone.success
          : KansoFeedbackTone.info,
      actionLabel: l10n.feedbackMine,
      onAction: () => router.push(KansoRoutes.settingsFeedback),
    );
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return BlocConsumer<SendFeedbackBloc, SendFeedbackState>(
      listenWhen: (before, after) =>
          before.result != after.result &&
          after.result != null &&
          feedbackClosesPage(after.result!.outcome),
      listener: (context, state) => _done(context, state.result!),
      builder: (context, state) {
        final bloc = context.read<SendFeedbackBloc>();
        final draft = state.draft;
        final result = state.result;
        final error = state.screenshotTooBig
            ? l10n.feedbackErrorScreenshot
            : result != null && !feedbackClosesPage(result.outcome)
            ? l10n.feedbackOutcomeMessage(result)
            : null;
        return ColoredBox(
          color: colors.app,
          child: Column(
            children: <Widget>[
              KansoTopBar(
                backgroundColor: colors.app,
                bordered: false,
                leading: const ShellBackButton(),
                title: l10n.feedbackSend,
              ),
              Expanded(
                child: ListView(
                  keyboardDismissBehavior:
                      ScrollViewKeyboardDismissBehavior.onDrag,
                  padding: const EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    KansoSpacing.xs,
                    KansoSpacing.md,
                    KansoSpacing.lg,
                  ),
                  children: <Widget>[
                    KansoSegmented<FeedbackKind>(
                      expanded: true,
                      semanticLabel: l10n.feedbackKindLabel,
                      selected: draft.kind,
                      segments: <KansoSegment<FeedbackKind>>[
                        for (final kind in FeedbackKind.values)
                          KansoSegment<FeedbackKind>(
                            value: kind,
                            label: l10n.feedbackKindName(kind),
                          ),
                      ],
                      onChanged: (kind) =>
                          bloc.add(SendFeedbackEvent.kindChanged(kind)),
                    ),
                    const SizedBox(height: KansoSpacing.md),
                    ClipRRect(
                      borderRadius: KansoRadii.largeBorder,
                      child: ColoredBox(
                        color: context.groupedSurface,
                        child: Padding(
                          padding: const EdgeInsets.all(KansoSpacing.md),
                          child: KansoTextField(
                            controller: _text,
                            hint: l10n.feedbackHint(draft.kind),
                            semanticLabel: l10n.feedbackSend,
                            minLines: 6,
                            maxLines: 12,
                            keyboardType: TextInputType.multiline,
                            autocorrect: true,
                            enabled: !state.sending,
                            onChanged: (text) =>
                                bloc.add(SendFeedbackEvent.textChanged(text)),
                          ),
                        ),
                      ),
                    ),
                    if (draft.showsCounter)
                      Padding(
                        padding: const EdgeInsets.only(top: KansoSpacing.xs),
                        child: Align(
                          alignment: AlignmentDirectional.centerEnd,
                          child: Text(
                            l10n.feedbackCounter(
                              draft.length,
                              feedbackMaxText,
                            ),
                            style: context.typography.secondary.copyWith(
                              color: draft.length > feedbackMaxText
                                  ? colors.danger
                                  : colors.textSecondary,
                            ),
                          ),
                        ),
                      ),
                    const SizedBox(height: KansoSpacing.md),
                    _ScreenshotRow(state: state),
                    const SizedBox(height: KansoSpacing.md),
                    KansoGroupedSection(
                      children: <Widget>[
                        DiscoverySettingSwitchRow(
                          title: l10n.feedbackIncludeAppInfo,
                          detail: l10n.feedbackIncludeAppInfoDetail,
                          value: draft.includeApp,
                          onChanged: state.sending
                              ? null
                              : (on) => bloc.add(
                                  SendFeedbackEvent.appInfoToggled(on: on),
                                ),
                        ),
                        _ShowSent(state: state),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.md),
                    KansoGroupedSection(
                      children: <Widget>[
                        DiscoverySettingSwitchRow(
                          title: l10n.feedbackCredit,
                          detail: l10n.feedbackCreditDetail,
                          value: draft.credit,
                          onChanged: state.sending
                              ? null
                              : (on) => bloc.add(
                                  SendFeedbackEvent.creditToggled(on: on),
                                ),
                        ),
                        if (draft.credit)
                          Padding(
                            padding: const EdgeInsets.fromLTRB(
                              KansoSpacing.md,
                              0,
                              KansoSpacing.md,
                              KansoSpacing.md,
                            ),
                            child: KansoTextField(
                              controller: _name,
                              hint: l10n.feedbackCreditName,
                              semanticLabel: l10n.feedbackCreditName,
                              invalid:
                                  draft.creditName.isNotEmpty &&
                                  !draft.creditNameValid,
                              enabled: !state.sending,
                              onChanged: (name) => bloc.add(
                                SendFeedbackEvent.creditNameChanged(name),
                              ),
                            ),
                          ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.md),
                    Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.xs,
                      ),
                      child: Text(
                        l10n.feedbackFooter,
                        style: context.typography.secondary,
                      ),
                    ),
                    if (error != null) ...<Widget>[
                      const SizedBox(height: KansoSpacing.md),
                      KansoInlineMessage(message: error),
                    ],
                  ],
                ),
              ),
              Padding(
                padding: EdgeInsets.fromLTRB(
                  KansoSpacing.md,
                  KansoSpacing.sm,
                  KansoSpacing.md,
                  bottom + KansoSpacing.md,
                ),
                child: KansoButton(
                  label: state.sending
                      ? l10n.feedbackSending
                      : l10n.feedbackSendButton,
                  onPressed: state.sending || !draft.canSend
                      ? null
                      : () => bloc.add(const SendFeedbackEvent.sendRequested()),
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}

/// Add screenshot, or the chosen one with Remove, and the warning.
class _ScreenshotRow extends StatelessWidget {
  const _ScreenshotRow({required this.state});

  final SendFeedbackState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<SendFeedbackBloc>();
    final shot = state.draft.screenshot;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        if (shot == null)
          KansoButton(
            label: l10n.feedbackAddScreenshot,
            icon: KansoIcons.image,
            variant: KansoButtonVariant.secondary,
            onPressed: state.sending
                ? null
                : () => bloc.add(const SendFeedbackEvent.screenshotRequested()),
          )
        else
          Row(
            children: <Widget>[
              ClipRRect(
                borderRadius: KansoRadii.smallBorder,
                child: Image.memory(
                  shot.bytes,
                  width: 56,
                  height: 56,
                  fit: BoxFit.cover,
                  gaplessPlayback: true,
                  errorBuilder: (context, error, stack) =>
                      const SizedBox(width: 56, height: 56),
                ),
              ),
              const SizedBox(width: KansoSpacing.sm),
              Expanded(
                child: Text(
                  l10n.feedbackScreenshotAttached(shot.name),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: context.typography.body,
                ),
              ),
              KansoButton(
                label: l10n.feedbackRemoveScreenshot,
                variant: KansoButtonVariant.quiet,
                expanded: false,
                compact: true,
                onPressed: state.sending
                    ? null
                    : () => bloc.add(
                        const SendFeedbackEvent.screenshotRemoved(),
                      ),
              ),
            ],
          ),
        const SizedBox(height: KansoSpacing.xs),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.xs),
          child: Text(
            l10n.feedbackScreenshotWarning,
            style: context.typography.secondary,
          ),
        ),
      ],
    );
  }
}

/// Show what's sent: the exact text of the report, as it will go.
class _ShowSent extends StatelessWidget {
  const _ShowSent({required this.state});

  final SendFeedbackState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<SendFeedbackBloc>();
    final shown = state.payload.json;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.sm),
            child: KansoButton(
              label: state.showSent
                  ? l10n.feedbackHideSent
                  : l10n.feedbackShowSent,
              variant: KansoButtonVariant.quiet,
              expanded: false,
              compact: true,
              onPressed: () => bloc.add(const SendFeedbackEvent.sentToggled()),
            ),
          ),
        ),
        if (state.showSent)
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              0,
              KansoSpacing.md,
              KansoSpacing.md,
            ),
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: colors.editor,
                borderRadius: KansoRadii.mediumBorder,
              ),
              child: Padding(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                child: SelectableText(
                  shown.isEmpty ? l10n.feedbackSentEmpty : shown,
                  key: const ValueKey<String>('feedback-sent-json'),
                  style: context.typography.secondary.copyWith(
                    fontFamily: 'monospace',
                    color: colors.text,
                  ),
                ),
              ),
            ),
          ),
      ],
    );
  }
}
