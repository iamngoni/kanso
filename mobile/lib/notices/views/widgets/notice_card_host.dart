// Kanso — notice_card_host.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../core/bridge/api/notices_types.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/account/account_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../state/notices_cubit.dart';
import 'notice_card.dart';
import 'notice_sheet.dart';

/// The update notice at the top of a list, or nothing: it asks the
/// [NoticesCubit] above it what applies, and asks again when the account
/// changes. Without a cubit above it, it is empty.
class NoticeCardHost extends StatefulWidget {
  const NoticeCardHost({super.key});

  /// The page of Settings a notice names. The phones have no page for some
  /// of the desktop's, and open Settings itself for those.
  static String settingsLocation(String page) => switch (page) {
    'general' => KansoRoutes.settingsGeneral,
    'account' => KansoRoutes.settingsAccount,
    'updates' => KansoRoutes.settingsAbout,
    _ => KansoRoutes.settings,
  };

  @override
  State<NoticeCardHost> createState() => _NoticeCardHostState();
}

class _NoticeCardHostState extends State<NoticeCardHost> {
  NoticesCubit? _cubit;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final cubit = _find();
    if (cubit != _cubit) {
      _cubit = cubit;
      cubit?.refresh(signedIn: _signedIn());
    }
  }

  NoticesCubit? _find() {
    try {
      return context.read<NoticesCubit>();
    } on ProviderNotFoundException {
      return null;
    }
  }

  AccountBloc? _account() {
    try {
      return context.read<AccountBloc>();
    } on ProviderNotFoundException {
      return null;
    }
  }

  bool _signedIn() {
    final state = _account()?.state;
    return state != null && (state.signedIn || state.locked != null);
  }

  @override
  Widget build(BuildContext context) {
    final cubit = _cubit;
    if (cubit == null) {
      return const SizedBox.shrink();
    }
    final card = _card(cubit);
    final account = _account();
    if (account == null) {
      return card;
    }
    // What applies can depend on the account, so it is asked again when the
    // account signs in or out.
    return BlocListener<AccountBloc, AccountBlocState>(
      bloc: account,
      listenWhen: (before, after) =>
          (before.signedIn || before.locked != null) !=
          (after.signedIn || after.locked != null),
      listener: (context, _) => cubit.refresh(signedIn: _signedIn()),
      child: card,
    );
  }

  Widget _card(NoticesCubit cubit) {
    return BlocBuilder<NoticesCubit, PhoneNotices?>(
      bloc: cubit,
      builder: (context, notices) {
        final card = notices?.card;
        if (card == null) {
          return const SizedBox.shrink();
        }
        return NoticeCard(
          notice: card,
          countLabel: (notices?.total ?? 0) > 1
              ? context.l10n.noticeCount(notices!.total)
              : '',
          onHowToFix: () => _open(context, cubit, card),
          onDismiss: () => cubit.dismiss(card.id, signedIn: _signedIn()),
        );
      },
    );
  }

  void _open(BuildContext context, NoticesCubit cubit, PhoneNotice notice) {
    showKansoBottomSheet<void>(
      context,
      builder: (sheet) {
        void close() => Navigator.of(sheet).maybePop();
        return NoticeSheet(
          notice: notice,
          onAction: notice.actionKind == PhoneNoticeActionKind.none
              ? null
              : () {
                  close();
                  _run(context, notice);
                },
          onMarkFixed: () {
            close();
            cubit.markFixed(notice.id, signedIn: _signedIn());
          },
          onDismiss: () {
            close();
            cubit.dismiss(notice.id, signedIn: _signedIn());
          },
        );
      },
    );
  }

  void _run(BuildContext context, PhoneNotice notice) {
    if (!context.mounted) {
      return;
    }
    switch (notice.actionKind) {
      case PhoneNoticeActionKind.none:
        break;
      case PhoneNoticeActionKind.openSettings:
        context.go(NoticeCardHost.settingsLocation(notice.actionTarget));
      case PhoneNoticeActionKind.openUrl:
        final uri = Uri.tryParse(notice.actionTarget);
        if (uri != null && uri.scheme == 'https') {
          launchUrl(uri, mode: LaunchMode.externalApplication);
        }
    }
  }
}
