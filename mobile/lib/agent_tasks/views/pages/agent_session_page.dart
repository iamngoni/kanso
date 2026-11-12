// Kanso — agent_session_page.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/agent_session.dart';
import '../../../core/models/data/agent_task.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_dialog.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_page_route.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../state/agent_session/agent_session_bloc.dart';
import '../../utils/session_text.dart';
import '../widgets/agent_composer.dart';
import '../widgets/agent_session_actions.dart';
import '../widgets/agent_transcript_view.dart';
import 'agent_review_page.dart';

/// One agent session: what the AI is doing as it happens, and a message
/// field to ask for more. [sessionKey] names the session.
class AgentSessionPage extends StatelessWidget {
  const AgentSessionPage({required this.sessionKey, super.key});

  final String sessionKey;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AgentSessionBloc>(
      create: (_) =>
          getIt<AgentSessionBloc>()..add(AgentSessionEvent.started(sessionKey)),
      child: KansoPage(
        backgroundColor: context.colors.app,
        body: const _Body(),
      ),
    );
  }
}

class _Body extends StatefulWidget {
  const _Body();

  @override
  State<_Body> createState() => _BodyState();
}

class _BodyState extends State<_Body> {
  final ScrollController _scroll = ScrollController();

  /// How close to the foot counts as "at the bottom", in points.
  static const double _stickBand = 96;

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  bool get _atBottom =>
      !_scroll.hasClients ||
      _scroll.position.maxScrollExtent - _scroll.offset <= _stickBand;

  /// Keeps the foot of the transcript in view while it grows, unless the
  /// person scrolled up to read.
  void _stick() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scroll.hasClients) {
        _scroll.jumpTo(_scroll.position.maxScrollExtent);
      }
    });
  }

  String? _reason(BuildContext context, AgentSession session) {
    final l10n = context.l10n;
    if (session.canFollowUp) {
      return null;
    }
    if (session.working) {
      return l10n.agentSessionComposerWorking;
    }
    return l10n.agentSessionComposerEnded;
  }

  Future<void> _end(BuildContext context) async {
    final l10n = context.l10n;
    final bloc = context.read<AgentSessionBloc>();
    final confirmed = await showKansoConfirm(
      context,
      title: l10n.agentSessionEndTitle,
      body: l10n.agentSessionEndBody,
      cancelLabel: l10n.agentTasksReadAllCancel,
      confirmLabel: l10n.agentSessionEnd,
      danger: true,
    );
    if (confirmed) {
      bloc.add(const AgentSessionEvent.ended());
    }
  }

  Future<void> _menu(
    BuildContext context,
    AgentSession session,
    Offset at,
  ) async {
    final bloc = context.read<AgentSessionBloc>();
    final action = await showAgentSessionMenu(context, at);
    if (action == null || !context.mounted) {
      return;
    }
    switch (action) {
      case AgentSessionAction.rename:
        final title = await askAgentSessionTitle(context, session.title);
        if (title != null) {
          bloc.add(AgentSessionEvent.renamed(title));
        }
      case AgentSessionAction.delete:
        if (await confirmDeleteAgentSession(context)) {
          bloc.add(const AgentSessionEvent.deleted());
        }
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<AgentSessionBloc>();
    return BlocConsumer<AgentSessionBloc, AgentSessionState>(
      listenWhen: (before, now) =>
          before.transcript?.version != now.transcript?.version,
      listener: (context, state) {
        if (_atBottom) {
          _stick();
        }
      },
      builder: (context, state) {
        final transcript = state.transcript;
        final session = transcript?.session;
        return Column(
          children: <Widget>[
            KansoTopBar(
              backgroundColor: colors.app,
              leading: KansoIconButton(
                icon: KansoIcons.chevronLeft,
                semanticLabel: l10n.agentTasksTitle,
                onPressed: () => Navigator.of(context).maybePop(),
              ),
              title: session?.title ?? l10n.agentTasksTitle,
              subtitle: session == null
                  ? null
                  : sessionLine(
                      l10n,
                      session,
                      DateTime.now().millisecondsSinceEpoch,
                    ),
              trailing: <Widget>[
                if (session != null)
                  Builder(
                    builder: (buttonContext) => KansoIconButton(
                      icon: KansoIcons.ellipsis,
                      semanticLabel: l10n.agentSessionMore,
                      onPressed: () {
                        final box =
                            buttonContext.findRenderObject()! as RenderBox;
                        final at = box.localToGlobal(
                          Offset(0, box.size.height),
                        );
                        unawaited(_menu(context, session, at));
                      },
                    ),
                  ),
                if (session != null && session.canStop)
                  KansoIconButton(
                    icon: KansoIcons.close,
                    semanticLabel: l10n.agentSessionEnd,
                    onPressed: () => unawaited(_end(context)),
                  ),
              ],
            ),
            Expanded(
              child: switch ((state.gone, transcript)) {
                (true, _) => KansoEmptyState(
                  title: l10n.agentSessionGone,
                  message: '',
                ),
                (_, null) when state.status == LoadStatus.failed =>
                  KansoEmptyState(title: l10n.agentSessionGone, message: ''),
                (_, null) => const SizedBox.shrink(),
                (_, final found?) => AgentTranscriptView(
                  controller: _scroll,
                  items: found.items,
                  agent: found.session.agent,
                  canReview: found.session.canReview,
                  canUndo: found.session.phase == AgentTaskPhase.applied,
                  onReview: () => Navigator.of(context).push<void>(
                    KansoPageRoute<void>(
                      builder: (_) => AgentReviewPage(task: found.session.id),
                    ),
                  ),
                  onUndo: () => bloc.add(const AgentSessionEvent.undone()),
                ),
              },
            ),
            if (state.trouble != null)
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                  vertical: KansoSpacing.xxs,
                ),
                child: Text(
                  state.trouble!,
                  style: context.typography.meta.copyWith(
                    color: colors.danger,
                  ),
                ),
              ),
            if (session != null) ...<Widget>[
              if (session.finished && session.phase != AgentTaskPhase.discarded)
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: KansoSpacing.md,
                    vertical: KansoSpacing.xxs,
                  ),
                  child: Align(
                    alignment: AlignmentDirectional.centerEnd,
                    child: KansoButton(
                      label: l10n.agentTaskDismiss,
                      variant: KansoButtonVariant.quiet,
                      compact: true,
                      expanded: false,
                      onPressed: () =>
                          bloc.add(const AgentSessionEvent.dismissed()),
                    ),
                  ),
                ),
              AgentComposer(
                canSend: session.canFollowUp,
                working: session.canStop,
                sending: state.sending,
                disabledReason: _reason(context, session),
                onSend: (text) =>
                    bloc.add(AgentSessionEvent.followUpSent(text)),
                onStop: () => bloc.add(const AgentSessionEvent.stopped()),
              ),
            ],
          ],
        );
      },
    );
  }
}
