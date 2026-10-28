// Kanso — tabbed_note.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/models/data/note_drag_data.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../../window/models/repos/abs/note_window_repository.dart';
import '../../state/tabs/tabs_bloc.dart';
import 'note_tab_bar.dart';

/// The note pane of a tablet with the row of tabs above it.
///
/// The bar shows from two tabs, as the desktop's does, and takes the status
/// bar's room, so the note's own bar sits right below it. With fewer, a strip
/// the bar's height appears over the top of the pane while notes are dragged
/// over it, and a note dropped there opens in a new tab. Where the tabs are
/// off (the window has one pane) the pane is shown as it is.
///
/// "Open in New Window" is in the tabs' menu only where [windows] (the
/// registered `NoteWindowRepository` when null) is there and `supported`.
class TabbedNote extends StatelessWidget {
  const TabbedNote({required this.child, this.windows, super.key});

  final Widget child;
  final NoteWindowRepository? windows;

  NoteWindowRepository? _windows() {
    final given = windows;
    if (given != null) {
      return given;
    }
    return getIt.isRegistered<NoteWindowRepository>()
        ? getIt<NoteWindowRepository>()
        : null;
  }

  @override
  Widget build(BuildContext context) {
    final tabs = context.read<TabsBloc>();
    return BlocBuilder<TabsBloc, TabsState>(
      buildWhen: (before, after) =>
          before.enabled != after.enabled ||
          !listEquals(before.tabs.metas, after.tabs.metas) ||
          before.tabs.active != after.tabs.active,
      builder: (context, state) {
        void dropNotes(List<String> ids) {
          for (final id in ids) {
            tabs.add(TabsEvent.openedInNewTab(id));
          }
        }

        final windows = _windows();
        final window = windows != null && windows.supported ? windows : null;
        // The note is in the same place of the tree whether or not there is
        // a bar, so showing the second tab does not rebuild the note.
        // The pane's slot may give the height loosely (a Row centres a child
        // it does not stretch), so the stack takes all of it: the bar and the
        // note fill the pane they are drawn in.
        return Stack(
          fit: StackFit.expand,
          children: <Widget>[
            Positioned.fill(
              child: Column(
                children: <Widget>[
                  if (state.barVisible)
                    ColoredBox(
                      color: context.colors.editor,
                      child: SafeArea(
                        bottom: false,
                        child: NoteTabBar(
                          tabs: state.tabs.metas,
                          active: state.tabs.active,
                          onActivate: (index) =>
                              tabs.add(TabsEvent.activated(index)),
                          onClose: (index) => tabs.add(TabsEvent.closed(index)),
                          onCloseOthers: (index) =>
                              tabs.add(TabsEvent.othersClosed(index)),
                          onCloseRight: (index) =>
                              tabs.add(TabsEvent.rightClosed(index)),
                          onReorder: (from, to) =>
                              tabs.add(TabsEvent.reordered(from, to)),
                          onDropNotes: dropNotes,
                          onOpenInNewWindow: window == null
                              ? null
                              : (id) => unawaited(window.open(id)),
                        ),
                      ),
                    )
                  else
                    const SizedBox.shrink(),
                  Expanded(
                    child: MediaQuery.removePadding(
                      context: context,
                      removeTop: state.barVisible,
                      child: child,
                    ),
                  ),
                ],
              ),
            ),
            if (state.enabled && !state.barVisible)
              Positioned(
                top: 0,
                left: 0,
                right: 0,
                height:
                    MediaQuery.paddingOf(context).top + KansoSizes.tabBarHeight,
                child: _DropStrip(onDrop: dropNotes),
              )
            else
              const SizedBox.shrink(),
          ],
        );
      },
    );
  }
}

/// The room a first extra tab is dropped on: nothing until notes are dragged
/// over it.
class _DropStrip extends StatelessWidget {
  const _DropStrip({required this.onDrop});

  final ValueChanged<List<String>> onDrop;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return DragTarget<NoteDragData>(
      onWillAcceptWithDetails: (_) => true,
      onAcceptWithDetails: (details) => onDrop(details.data.noteIds),
      builder: (context, hovering, rejected) {
        if (hovering.isEmpty) {
          return const SizedBox.expand();
        }
        return ColoredBox(
          color: Color.alphaBlend(
            colors.accent.withValues(alpha: KansoEffects.wash),
            colors.editor,
          ),
          child: Align(
            alignment: Alignment.bottomCenter,
            child: SizedBox(
              height: KansoSizes.tabBarHeight,
              child: Center(
                child: Text(
                  context.l10n.tabOpenInNewTab,
                  style: context.typography.label,
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
