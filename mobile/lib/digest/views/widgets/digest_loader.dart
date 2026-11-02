// Kanso — digest_loader.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart' show Either;
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/debounce.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';

/// How long the library must be quiet before a page of loops is read again.
const Duration digestRefreshDebounce = Duration(milliseconds: 400);

/// A page that reads something from the Rust core, shows it with [builder],
/// and reads it again when the library changes, when [reloadKey] changes or
/// when [builder] asks.
class DigestLoader<T> extends StatefulWidget {
  const DigestLoader({
    required this.title,
    required this.load,
    required this.builder,
    this.subtitle,
    this.reloadKey,
    this.trailing = const <Widget>[],
    super.key,
  });

  final String title;
  final String? Function(T value)? subtitle;
  final Future<Either<ApplicationError, T>> Function() load;
  final Widget Function(
    BuildContext context,
    T value,
    Future<void> Function() reload,
  )
  builder;

  /// Reading starts again when this changes.
  final Object? reloadKey;
  final List<Widget> trailing;

  @override
  State<DigestLoader<T>> createState() => _DigestLoaderState<T>();
}

class _DigestLoaderState<T> extends State<DigestLoader<T>> {
  T? value;
  bool failed = false;
  StreamSubscription<List<String>>? changes;
  int generation = 0;

  @override
  void initState() {
    super.initState();
    changes = debounceStream(
      getIt<LibraryRepository>().changes(),
      digestRefreshDebounce,
    ).listen((_) => unawaited(read()));
    unawaited(read());
  }

  @override
  void didUpdateWidget(DigestLoader<T> oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.reloadKey != widget.reloadKey) {
      unawaited(read());
    }
  }

  @override
  void dispose() {
    unawaited(changes?.cancel());
    super.dispose();
  }

  Future<void> read() async {
    final mine = ++generation;
    final result = await widget.load();
    if (!mounted || mine != generation) {
      return;
    }
    setState(() {
      result.fold((_) => failed = true, (it) {
        value = it;
        failed = false;
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final current = value;
    final String? line = current == null
        ? null
        : widget.subtitle?.call(current);
    return KansoPage(
      backgroundColor: context.colors.list,
      body: Column(
        children: <Widget>[
          KansoTopBar(
            title: widget.title,
            subtitle: line,
            leading: KansoIconButton(
              icon: KansoIcons.chevronLeft,
              semanticLabel: l10n.settingsBack,
              onPressed: () => context.pop(),
            ),
            trailing: widget.trailing,
          ),
          Expanded(
            child: current != null
                ? widget.builder(context, current, read)
                : failed
                ? KansoEmptyState(
                    title: l10n.digestError,
                    message: '',
                    action: KansoButton(
                      label: l10n.digestRetry,
                      expanded: false,
                      onPressed: () => unawaited(read()),
                    ),
                  )
                : const Center(child: KansoSpinner()),
          ),
        ],
      ),
    );
  }
}

/// The padding of a page's list.
const EdgeInsets digestListPadding = EdgeInsets.only(
  bottom: KansoSpacing.xxl,
);
