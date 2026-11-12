// Kanso — whats_new_host.dart
// Created 2026-11-12.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/whats_new_types.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../state/whats_new_cubit.dart';
import 'whats_new_sheet.dart';

/// Opens What's new once after an update: asks the [WhatsNewCubit] above
/// it, shows the sheet when there is one, and puts it away for this
/// version when the sheet closes. Draws nothing itself; without a cubit
/// above it, it does nothing.
class WhatsNewHost extends StatefulWidget {
  const WhatsNewHost({super.key});

  @override
  State<WhatsNewHost> createState() => _WhatsNewHostState();
}

class _WhatsNewHostState extends State<WhatsNewHost> {
  WhatsNewCubit? _cubit;
  bool _showing = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    WhatsNewCubit? cubit;
    try {
      cubit = context.read<WhatsNewCubit>();
    } on ProviderNotFoundException {
      cubit = null;
    }
    if (cubit != _cubit) {
      _cubit = cubit;
      cubit?.load();
    }
  }

  @override
  Widget build(BuildContext context) {
    final cubit = _cubit;
    if (cubit == null) {
      return const SizedBox.shrink();
    }
    return BlocListener<WhatsNewCubit, PhoneWhatsNew?>(
      bloc: cubit,
      listenWhen: (_, after) => after != null,
      listener: (context, whatsNew) => _show(context, cubit, whatsNew!),
      child: const SizedBox.shrink(),
    );
  }

  Future<void> _show(
    BuildContext context,
    WhatsNewCubit cubit,
    PhoneWhatsNew whatsNew,
  ) async {
    if (_showing) {
      return;
    }
    _showing = true;
    await showKansoBottomSheet<void>(
      context,
      builder: (_) => WhatsNewSheet(whatsNew: whatsNew),
    );
    _showing = false;
    await cubit.seen();
  }
}
