// Kanso — join_link_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_text_input.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/join_link/join_link_bloc.dart';
import 'share_message_text.dart';

/// Join with a link: a field for the invite link someone sent, and Join. The
/// note's owner lets the person in, so the answer is who was asked; the note
/// comes to the library once they approve. It needs a [JoinLinkBloc] above
/// it.
class JoinLinkPane extends StatefulWidget {
  const JoinLinkPane({super.key});

  @override
  State<JoinLinkPane> createState() => _JoinLinkPaneState();
}

class _JoinLinkPaneState extends State<JoinLinkPane> {
  late final TextEditingController link;

  @override
  void initState() {
    super.initState();
    link = TextEditingController();
  }

  @override
  void dispose() {
    link.dispose();
    super.dispose();
  }

  void join(BuildContext context) {
    context.read<JoinLinkBloc>().add(JoinLinkEvent.submitted(link.text));
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    final keyboard = MediaQuery.viewInsetsOf(context).bottom;
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: const ShellBackButton(),
            title: l10n.joinTitle,
          ),
          Expanded(
            child: BlocBuilder<JoinLinkBloc, JoinLinkState>(
              builder: (context, state) {
                final asked = state.asked;
                if (asked != null) {
                  return Center(
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      child: KansoMessageCard(
                        icon: KansoIcons.check,
                        title: l10n.joinSentTitle,
                        message: l10n.joinAsked(asked.owner),
                        actions: <Widget>[
                          KansoButton(
                            label: l10n.joinAnother,
                            variant: KansoButtonVariant.secondary,
                            onPressed: () {
                              link.clear();
                              context.read<JoinLinkBloc>().add(
                                const JoinLinkEvent.restarted(),
                              );
                            },
                          ),
                        ],
                      ),
                    ),
                  );
                }
                return ListView(
                  padding: EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    KansoSpacing.xs,
                    KansoSpacing.md,
                    bottom + keyboard + KansoSpacing.lg,
                  ),
                  children: <Widget>[
                    Text(l10n.joinMessage, style: context.typography.body),
                    const SizedBox(height: KansoSpacing.md),
                    KansoTextInput(
                      controller: link,
                      hint: l10n.joinHint,
                      autocorrect: false,
                      enabled: !state.joining,
                      keyboardType: TextInputType.url,
                      textInputAction: TextInputAction.go,
                      onSubmitted: (_) => join(context),
                    ),
                    if (state.problem != null) ...<Widget>[
                      const SizedBox(height: KansoSpacing.sm),
                      ShareMessageText(problem: state.problem),
                    ],
                    const SizedBox(height: KansoSpacing.md),
                    KansoButton(
                      label: l10n.joinButton,
                      onPressed: state.joining ? null : () => join(context),
                    ),
                  ],
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
