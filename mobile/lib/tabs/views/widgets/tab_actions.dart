// Kanso — tab_actions.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/data/tab_intent.dart';
import '../../state/tabs/tabs_bloc.dart';

/// Answers the [TabIntent]s from anywhere below it with the tabs' BLoC.
///
/// Closing the only tab does nothing, whatever the key: a tablet has no
/// window to close with it. While there are no tabs (a phone, a window with
/// one pane) every intent is left unhandled so a key still reaches whatever
/// else is bound to it.
class TabActions extends StatelessWidget {
  const TabActions({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final TabsBloc tabs;
    try {
      tabs = context.read<TabsBloc>();
    } on ProviderNotFoundException {
      return child;
    }
    CallbackAction<T> on<T extends Intent>(Object? Function(T intent) run) =>
        _TabAction<T>(() => tabs.state.enabled, run);
    return Actions(
      actions: <Type, Action<Intent>>{
        CloseTabIntent: on<CloseTabIntent>((_) {
          final state = tabs.state;
          if (state.tabs.length > 1) {
            tabs.add(TabsEvent.closed(state.tabs.active));
          }
          return null;
        }),
        NextTabIntent: on<NextTabIntent>(
          (_) => tabs.add(const TabsEvent.nextRequested()),
        ),
        PreviousTabIntent: on<PreviousTabIntent>(
          (_) => tabs.add(const TabsEvent.previousRequested()),
        ),
        ReopenClosedTabIntent: on<ReopenClosedTabIntent>(
          (_) => tabs.add(const TabsEvent.reopenRequested()),
        ),
        GoToTabIntent: on<GoToTabIntent>(
          (intent) => tabs.add(TabsEvent.numbered(intent.number)),
        ),
        OpenInNewTabIntent: on<OpenInNewTabIntent>(
          (intent) => tabs.add(TabsEvent.openedInNewTab(intent.noteId)),
        ),
      },
      child: child,
    );
  }
}

/// A callback for [T] that is enabled only while the tabs are on.
class _TabAction<T extends Intent> extends CallbackAction<T> {
  _TabAction(this._enabled, Object? Function(T intent) run)
    : super(onInvoke: run);

  final bool Function() _enabled;

  @override
  bool isEnabled(T intent) => _enabled();
}
