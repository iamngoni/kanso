// Kanso — triage_bar.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../injection.dart';
import '../../models/repos/abs/triage_repository.dart';

/// The line under the Inbox's bar: how many notes wait to be filed, and the
/// way into Triage. Hidden while none wait or Show Triage bar is off.
class TriageBar extends StatefulWidget {
  const TriageBar({super.key});

  @override
  State<TriageBar> createState() => _TriageBarState();
}

class _TriageBarState extends State<TriageBar> {
  final TriageRepository _triage = getIt<TriageRepository>();
  StreamSubscription<List<String>>? _changes;
  int _waiting = 0;
  bool _show = true;

  @override
  void initState() {
    super.initState();
    unawaited(_read());
    _changes = _triage.changes().listen((_) => unawaited(_read()));
  }

  Future<void> _read() async {
    final status = await _triage.status();
    final settings = await _triage.settings();
    if (!mounted) {
      return;
    }
    setState(() {
      _waiting = status.toOption().toNullable()?.waiting ?? 0;
      _show = settings.toOption().toNullable()?.showBar ?? true;
    });
  }

  @override
  void dispose() {
    unawaited(_changes?.cancel());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!_show || _waiting == 0) {
      return const SizedBox.shrink();
    }
    final l10n = context.l10n;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.only(
            start: KansoSpacing.md,
            end: KansoSpacing.xxs,
          ),
          child: Row(
            children: <Widget>[
              Expanded(
                child: Text(
                  l10n.triageBarCount(_waiting),
                  style: context.typography.body,
                ),
              ),
              KansoPressable(
                onTap: () => context.push(KansoRoutes.triage),
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.sm),
                  child: Text(
                    l10n.triageBarOpen,
                    style: context.typography.body.copyWith(
                      color: context.colors.accent,
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
        const KansoDivider(),
      ],
    );
  }
}
