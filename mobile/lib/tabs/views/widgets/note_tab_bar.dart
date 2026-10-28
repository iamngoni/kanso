// Kanso — note_tab_bar.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart' show CustomSemanticsAction;

import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_motion.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/note_drag_data.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../models/data/tab_drag_data.dart';
import '../../models/data/tab_meta.dart';

/// What a tab's menu offers.
enum _TabMenuAction { close, closeOthers, closeRight, openInNewWindow }

/// The row of tabs above a note on a tablet: the notes open, the one showing,
/// and what can be done to each (the desktop's `tab_bar.rs`).
///
/// A tap shows a tab. A press that is held lifts it: dragged along the row it
/// moves to the place it is dropped on, and let go in place it opens the
/// menu, as a right click (a secondary tap) does at once. The close button
/// is on the tab showing and, under a pointer, on the tab the pointer is
/// over. The row scrolls when the tabs run past the pane and keeps the tab
/// showing in view; the button at its end lists every tab.
///
/// Notes dragged from the list ([NoteDragData]) and dropped on the bar open
/// in new tabs. [onOpenInNewWindow] is null where a note cannot have a
/// window of its own, and then the menu has no such item.
class NoteTabBar extends StatefulWidget {
  const NoteTabBar({
    required this.tabs,
    required this.active,
    required this.onActivate,
    required this.onClose,
    required this.onCloseOthers,
    required this.onCloseRight,
    required this.onReorder,
    required this.onDropNotes,
    this.onOpenInNewWindow,
    super.key,
  });

  final List<TabMeta> tabs;

  /// The index of the tab showing.
  final int active;
  final ValueChanged<int> onActivate;
  final ValueChanged<int> onClose;
  final ValueChanged<int> onCloseOthers;
  final ValueChanged<int> onCloseRight;

  /// A tab was dropped from the place `from` on the place `to`.
  final void Function(int from, int to) onReorder;

  /// Notes were dropped on the bar.
  final ValueChanged<List<String>> onDropNotes;

  /// Opens the note with this id in a window of its own.
  final ValueChanged<String>? onOpenInNewWindow;

  @override
  State<NoteTabBar> createState() => _NoteTabBarState();
}

class _NoteTabBarState extends State<NoteTabBar> {
  final ScrollController _scroll = ScrollController();
  final Map<String, GlobalKey> _keys = <String, GlobalKey>{};

  @override
  void initState() {
    super.initState();
    _keepActiveInView();
  }

  @override
  void didUpdateWidget(NoteTabBar oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (_idAt(oldWidget, oldWidget.active) != _idAt(widget, widget.active) ||
        oldWidget.tabs.length != widget.tabs.length) {
      _keepActiveInView();
    }
  }

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  String? _idAt(NoteTabBar bar, int index) =>
      index >= 0 && index < bar.tabs.length ? bar.tabs[index].id : null;

  /// The row can run past the pane, and a tab out of sight with it. Asking
  /// after the frame lets the row lay out first.
  void _keepActiveInView() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) {
        return;
      }
      final id = _idAt(widget, widget.active);
      final context = id == null ? null : _keys[id]?.currentContext;
      if (context != null && context.mounted) {
        unawaited(
          Scrollable.ensureVisible(
            context,
            duration: KansoMotion.resolve(context, KansoMotion.short),
            curve: KansoMotion.standard,
          ),
        );
      }
    });
  }

  Future<void> _openMenu(int index, Offset at) async {
    final l10n = context.l10n;
    final count = widget.tabs.length;
    final window = widget.onOpenInNewWindow;
    final chosen = await showKansoMenu<_TabMenuAction>(
      context,
      at,
      <PopupMenuEntry<_TabMenuAction>>[
        kansoMenuItem(
          context,
          _TabMenuAction.close,
          l10n.tabClose,
          enabled: count > 1,
        ),
        kansoMenuItem(
          context,
          _TabMenuAction.closeOthers,
          l10n.tabCloseOthers,
          enabled: count > 1,
        ),
        kansoMenuItem(
          context,
          _TabMenuAction.closeRight,
          l10n.tabCloseRight,
          enabled: index + 1 < count,
        ),
        if (window != null) ...<PopupMenuEntry<_TabMenuAction>>[
          const PopupMenuDivider(height: 1),
          kansoMenuItem(
            context,
            _TabMenuAction.openInNewWindow,
            l10n.tabOpenInNewWindow,
          ),
        ],
      ],
    );
    switch (chosen) {
      case _TabMenuAction.close:
        widget.onClose(index);
      case _TabMenuAction.closeOthers:
        widget.onCloseOthers(index);
      case _TabMenuAction.closeRight:
        widget.onCloseRight(index);
      case _TabMenuAction.openInNewWindow:
        window?.call(widget.tabs[index].id);
      case null:
        break;
    }
  }

  Future<void> _openAllTabs(BuildContext button) async {
    final l10n = context.l10n;
    final box = button.findRenderObject()! as RenderBox;
    final at = box.localToGlobal(Offset(box.size.width, box.size.height));
    final chosen = await showKansoMenu<int>(
      context,
      at,
      <PopupMenuEntry<int>>[
        for (var index = 0; index < widget.tabs.length; index++)
          CheckedPopupMenuItem<int>(
            value: index,
            checked: index == widget.active,
            child: Text(
              widget.tabs[index].label(l10n.noteUntitled),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: context.typography.body,
            ),
          ),
      ],
    );
    if (chosen != null) {
      widget.onActivate(chosen);
    }
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return DragTarget<NoteDragData>(
      onWillAcceptWithDetails: (_) => true,
      onAcceptWithDetails: (details) =>
          widget.onDropNotes(details.data.noteIds),
      builder: (context, hovering, rejected) {
        return Semantics(
          container: true,
          explicitChildNodes: true,
          label: l10n.tabsBarLabel,
          child: Container(
            height: KansoSizes.tabBarHeight,
            padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.xs),
            decoration: BoxDecoration(
              color: hovering.isEmpty
                  ? colors.editor
                  : Color.alphaBlend(
                      colors.accent.withValues(alpha: KansoEffects.wash),
                      colors.editor,
                    ),
              border: Border(bottom: BorderSide(color: colors.divider)),
            ),
            child: Row(
              children: <Widget>[
                Expanded(
                  child: SingleChildScrollView(
                    controller: _scroll,
                    scrollDirection: Axis.horizontal,
                    child: Row(
                      children: <Widget>[
                        for (var index = 0; index < widget.tabs.length; index++)
                          Padding(
                            padding: const EdgeInsets.only(right: 2),
                            child: _NoteTab(
                              key: _keys.putIfAbsent(
                                widget.tabs[index].id,
                                GlobalKey.new,
                              ),
                              meta: widget.tabs[index],
                              index: index,
                              active: index == widget.active,
                              onActivate: () => widget.onActivate(index),
                              onClose: () => widget.onClose(index),
                              onReorder: widget.onReorder,
                              onMenu: (at) => _openMenu(index, at),
                            ),
                          ),
                      ],
                    ),
                  ),
                ),
                Builder(
                  builder: (button) => Semantics(
                    button: true,
                    label: l10n.tabsAll,
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: () => unawaited(_openAllTabs(button)),
                      child: SizedBox.square(
                        dimension: KansoSizes.tab,
                        child: Center(
                          child: KansoIcon(
                            KansoIcons.ellipsis,
                            size: KansoSizes.iconSmall,
                            color: colors.textSecondary,
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

/// One tab. See [NoteTabBar] for what a press does.
class _NoteTab extends StatefulWidget {
  const _NoteTab({
    required this.meta,
    required this.index,
    required this.active,
    required this.onActivate,
    required this.onClose,
    required this.onReorder,
    required this.onMenu,
    super.key,
  });

  final TabMeta meta;
  final int index;
  final bool active;
  final VoidCallback onActivate;
  final VoidCallback onClose;
  final void Function(int from, int to) onReorder;

  /// Asks for the menu with its top left at the offset.
  final Future<void> Function(Offset at) onMenu;

  @override
  State<_NoteTab> createState() => _NoteTabState();
}

class _NoteTabState extends State<_NoteTab> {
  bool _hovered = false;
  Offset _lifted = Offset.zero;

  Offset _origin() {
    final box = context.findRenderObject();
    return box is RenderBox ? box.localToGlobal(Offset.zero) : Offset.zero;
  }

  void _menuBelow() {
    final box = context.findRenderObject();
    final height = box is RenderBox ? box.size.height : KansoSizes.tab;
    unawaited(widget.onMenu(_origin().translate(0, height)));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final title = widget.meta.label(l10n.noteUntitled);
    final drag = TabDragData(from: widget.index, title: title);
    return Semantics(
      container: true,
      button: true,
      selected: widget.active,
      excludeSemantics: true,
      label: widget.meta.locked ? l10n.tabLocked(title) : title,
      onTap: widget.onActivate,
      onLongPress: _menuBelow,
      customSemanticsActions: <CustomSemanticsAction, VoidCallback>{
        CustomSemanticsAction(label: l10n.tabCloseTooltip): widget.onClose,
      },
      child: LongPressDraggable<TabDragData>(
        data: drag,
        axis: Axis.horizontal,
        feedback: _Lifted(title: title),
        childWhenDragging: Opacity(
          opacity: KansoEffects.disabled,
          child: _body(context, title, over: false),
        ),
        onDragStarted: () => _lifted = _origin(),
        onDragEnd: (details) {
          // Let go where it was lifted from: the press was a long press,
          // and it asks for the menu.
          if (!details.wasAccepted &&
              (details.offset - _lifted).distance < 16) {
            _menuBelow();
          }
        },
        child: DragTarget<TabDragData>(
          onWillAcceptWithDetails: (details) =>
              details.data.from != widget.index,
          onAcceptWithDetails: (details) =>
              widget.onReorder(details.data.from, widget.index),
          builder: (context, over, rejected) =>
              _body(context, title, over: over.isNotEmpty),
        ),
      ),
    );
  }

  Widget _body(BuildContext context, String title, {required bool over}) {
    final colors = context.colors;
    final active = widget.active;
    final showClose = active || _hovered;
    final base = context.typography.label.copyWith(fontSize: 12.5);
    final background = over
        ? colors.accent.withValues(alpha: KansoEffects.wash)
        : active
        ? colors.fill
        : _hovered
        ? colors.rowHover
        : null;
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onActivate,
        onSecondaryTapUp: (details) =>
            unawaited(widget.onMenu(details.globalPosition)),
        child: Container(
          height: KansoSizes.tab,
          constraints: const BoxConstraints(
            minWidth: KansoSizes.tabMinWidth,
            maxWidth: KansoSizes.tabMaxWidth,
          ),
          padding: const EdgeInsets.only(left: 10, right: KansoSpacing.xxs),
          decoration: BoxDecoration(
            color: background,
            borderRadius: BorderRadius.circular(7),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              if (widget.meta.locked) ...<Widget>[
                KansoIcon(
                  KansoIcons.lock,
                  size: 12,
                  color: colors.textMuted,
                ),
                const SizedBox(width: 6),
              ],
              Flexible(
                child: Text(
                  title,
                  maxLines: 1,
                  softWrap: false,
                  overflow: TextOverflow.ellipsis,
                  style: base.copyWith(
                    color: active ? colors.text : colors.textSecondary,
                  ),
                ),
              ),
              const SizedBox(width: 6),
              IgnorePointer(
                ignoring: !showClose,
                child: Opacity(
                  opacity: showClose ? 1 : 0,
                  child: GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onTap: widget.onClose,
                    child: SizedBox.square(
                      dimension: 22,
                      child: Center(
                        child: KansoIcon(
                          KansoIcons.close,
                          size: 12,
                          color: colors.textSecondary,
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// The tab as it is lifted.
class _Lifted extends StatelessWidget {
  const _Lifted({required this.title});

  final String title;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Material(
      type: MaterialType.transparency,
      child: Container(
        constraints: const BoxConstraints(maxWidth: 200),
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
        decoration: BoxDecoration(
          color: colors.elevated,
          borderRadius: BorderRadius.circular(KansoRadii.small),
          border: Border.all(color: colors.accent.withValues(alpha: 0.6)),
          boxShadow: const <BoxShadow>[
            BoxShadow(
              color: Color(0x33000000),
              blurRadius: 8,
              offset: Offset(0, 2),
            ),
          ],
        ),
        child: Text(
          title,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: context.typography.label.copyWith(
            fontSize: 12.5,
            color: colors.text,
          ),
        ),
      ),
    );
  }
}
