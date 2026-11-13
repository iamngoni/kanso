// Kanso — board_settings_sheet.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../state/board/board_bloc.dart';

/// How a notebook's board is laid out: what it is grouped by and, when that
/// is a property, its lanes, which can be renamed, reordered, added or
/// removed, and whether the lane of cards with no value shows while empty.
///
/// It only asks: every change is an event of the `BoardBloc`, which has the
/// core write it (the notes in a renamed or removed lane are rewritten
/// there) and say why when it is refused. The sheet reads the bloc, so it
/// shows the board as the core has it after each change.
class BoardSettingsSheet extends StatelessWidget {
  const BoardSettingsSheet({super.key});

  /// Shows the sheet over the whole app, for the board [context] is in.
  static Future<void> show(BuildContext context) async {
    final bloc = context.read<BoardBloc>();
    await showKansoBottomSheet<void>(
      context,
      builder: (_) => BlocProvider<BoardBloc>.value(
        value: bloc,
        child: const BoardSettingsSheet(),
      ),
    );
    bloc.add(const BoardEvent.settingsClosed());
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<BoardBloc, BoardState>(
      builder: (context, state) {
        final setup = state.data?.setup;
        final key = setup?.group;
        return ListView(
          shrinkWrap: true,
          padding: const EdgeInsets.only(bottom: KansoSpacing.md),
          children: <Widget>[
            SheetHeader(
              title: l10n.boardSettingsTitle,
              actionLabel: l10n.boardSettingsDone,
            ),
            if (setup != null) ...<Widget>[
              _GroupBy(setup: setup),
              if (key != null) ...<Widget>[
                _Lanes(setup: setup, key: const ValueKey<String>('lanes')),
                if (state.removal case final removal?)
                  _Removal(removal: removal, propertyKey: key),
                _ShowUnset(setup: setup, propertyKey: key),
              ],
            ],
            if (state.settingsError case final error?)
              Padding(
                padding: const EdgeInsets.fromLTRB(
                  KansoSpacing.md,
                  KansoSpacing.sm,
                  KansoSpacing.md,
                  0,
                ),
                child: KansoInlineMessage(message: error),
              ),
          ],
        );
      },
    );
  }
}

/// A group of rows with a heading, and room around it.
class _Section extends StatelessWidget {
  const _Section({required this.title, required this.children});

  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.sm,
        KansoSpacing.md,
        0,
      ),
      child: KansoGroupedSection(title: title, children: children),
    );
  }
}

/// Status, or one of the notebook's select properties.
class _GroupBy extends StatelessWidget {
  const _GroupBy({required this.setup});

  final BoardSetup setup;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<BoardBloc>();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        _Section(
          title: l10n.boardSettingsGroupBy,
          children: <Widget>[
            KansoCheckRow(
              label: l10n.boardSettingsStatus,
              checked: setup.group == null,
              onTap: () => bloc.add(const BoardEvent.groupChanged(null)),
            ),
            for (final key in setup.groupable)
              KansoCheckRow(
                label: key,
                checked: setup.group == key,
                onTap: () => bloc.add(BoardEvent.groupChanged(key)),
              ),
          ],
        ),
        if (setup.group == null)
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.lg,
              KansoSpacing.xs,
              KansoSpacing.lg,
              0,
            ),
            child: Text(
              l10n.boardSettingsGroupHint,
              style: context.typography.meta,
            ),
          ),
      ],
    );
  }
}

/// The lanes of a board grouped by a property, with a field to add one.
class _Lanes extends StatelessWidget {
  const _Lanes({required this.setup, super.key});

  final BoardSetup setup;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<BoardBloc>();
    final lanes = setup.lanes;
    return _Section(
      title: l10n.boardSettingsLanes,
      children: <Widget>[
        ReorderableListView.builder(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          buildDefaultDragHandles: false,
          itemCount: lanes.length,
          // The index is that of the lane after it is lifted out, as the
          // core's is.
          onReorderItem: (from, to) => bloc.add(
            BoardEvent.laneEdited(LaneChange.move(from: from, to: to)),
          ),
          itemBuilder: (context, index) => _LaneRow(
            key: ValueKey<String>('lane:${lanes[index]}'),
            index: index,
            lane: lanes[index],
            count: lanes.length,
          ),
        ),
        _AddLane(lanes: lanes),
      ],
    );
  }
}

/// One lane: a handle to reorder it, its name (changed by editing it) and a
/// button that removes it. A board keeps at least one lane.
class _LaneRow extends StatefulWidget {
  const _LaneRow({
    required this.index,
    required this.lane,
    required this.count,
    super.key,
  });

  final int index;
  final String lane;
  final int count;

  @override
  State<_LaneRow> createState() => _LaneRowState();
}

class _LaneRowState extends State<_LaneRow> {
  late final TextEditingController _name = TextEditingController(
    text: widget.lane,
  );

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  /// The name last asked for: a name sent by Return is not sent again when
  /// the field then loses focus.
  String? _sent;

  /// Asks for the lane to have the name typed, when it is another.
  void _commit() {
    final name = _name.text;
    if (name != widget.lane && name != _sent) {
      _sent = name;
      context.read<BoardBloc>().add(
        BoardEvent.laneEdited(
          LaneChange.rename(from: widget.lane, to: name),
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<BoardBloc>();
    final colors = context.colors;
    final lane = widget.lane;
    // A refused rename puts the name back as it was. The list gives each row
    // the actions that move it up, down, to the start and to the end for a
    // screen reader; the parts of the row are separate for it.
    return BlocListener<BoardBloc, BoardState>(
      listenWhen: (before, after) =>
          after.settingsError != null &&
          after.settingsError != before.settingsError,
      listener: (context, state) {
        _sent = null;
        _name.text = lane;
      },
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: KansoSizes.settingsRow),
        child: Row(
          children: <Widget>[
            ReorderableDragStartListener(
              index: widget.index,
              child: Semantics(
                container: true,
                label: l10n.boardSettingsMoveLane(lane),
                child: SizedBox(
                  width: KansoSizes.touchTarget,
                  height: KansoSizes.touchTarget,
                  child: Center(
                    child: KansoIcon(
                      KansoIcons.arrowUpDown,
                      color: colors.textSecondary,
                    ),
                  ),
                ),
              ),
            ),
            Expanded(
              child: Semantics(
                container: true,
                child: Focus(
                  onFocusChange: (focused) {
                    if (!focused) {
                      _commit();
                    }
                  },
                  child: KansoTextField(
                    controller: _name,
                    hint: l10n.boardSettingsLaneName,
                    semanticLabel: l10n.boardSettingsLaneName,
                    onSubmitted: (_) => _commit(),
                  ),
                ),
              ),
            ),
            Semantics(
              container: true,
              child: KansoIconButton(
                icon: KansoIcons.trash,
                semanticLabel: l10n.boardSettingsRemoveLane(lane),
                // A board needs one lane.
                onPressed: widget.count > 1
                    ? () => bloc.add(BoardEvent.laneRemovalRequested(lane))
                    : null,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// A field that adds the lane typed to the end of the board.
class _AddLane extends StatefulWidget {
  const _AddLane({required this.lanes});

  final List<String> lanes;

  @override
  State<_AddLane> createState() => _AddLaneState();
}

class _AddLaneState extends State<_AddLane> {
  final TextEditingController _name = TextEditingController();

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  void _add() {
    final name = _name.text;
    if (name.trim().isEmpty) {
      return;
    }
    context.read<BoardBloc>().add(
      BoardEvent.laneEdited(LaneChange.add(name: name)),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocListener<BoardBloc, BoardState>(
      // The lane typed is in the board now: the field is ready for another.
      listenWhen: (before, after) =>
          after.data?.setup?.lanes.length != before.data?.setup?.lanes.length,
      listener: (context, state) => _name.clear(),
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: KansoSizes.settingsRow),
        child: Row(
          children: <Widget>[
            const SizedBox(width: KansoSpacing.md),
            Expanded(
              child: Semantics(
                container: true,
                child: KansoTextField(
                  controller: _name,
                  hint: l10n.boardSettingsAddLane,
                  semanticLabel: l10n.boardSettingsAddLane,
                  textInputAction: TextInputAction.done,
                  onSubmitted: (_) => _add(),
                ),
              ),
            ),
            Semantics(
              container: true,
              child: KansoIconButton(
                icon: KansoIcons.plus,
                semanticLabel: l10n.boardSettingsAddLaneButton,
                onPressed: _add,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// The question that comes before a lane with notes in it goes: where do
/// they go?
class _Removal extends StatelessWidget {
  const _Removal({required this.removal, required this.propertyKey});

  final BoardRemoval removal;
  final String propertyKey;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<BoardBloc>();
    final target = removal.target;
    return _Section(
      title: l10n.boardSettingsRemoveLane(removal.lane),
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: Semantics(
            liveRegion: true,
            child: Text(removal.plan.prompt, style: context.typography.body),
          ),
        ),
        for (final lane in removal.plan.targets)
          KansoCheckRow(
            label: lane,
            checked: target == lane,
            onTap: () => bloc.add(BoardEvent.removalTargetChosen(lane)),
          ),
        KansoCheckRow(
          label: l10n.boardSettingsNoValue(propertyKey),
          checked: target == null,
          onTap: () => bloc.add(const BoardEvent.removalTargetChosen(null)),
        ),
        Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: Row(
            children: <Widget>[
              Expanded(
                child: KansoButton(
                  label: l10n.cancel,
                  variant: KansoButtonVariant.secondary,
                  onPressed: () =>
                      bloc.add(const BoardEvent.removalCancelled()),
                ),
              ),
              const SizedBox(width: KansoSpacing.sm),
              Expanded(
                child: KansoButton(
                  label: l10n.boardSettingsRemove,
                  variant: KansoButtonVariant.danger,
                  onPressed: () =>
                      bloc.add(const BoardEvent.removalConfirmed()),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

/// Whether the lane of cards with no value shows while it is empty.
class _ShowUnset extends StatelessWidget {
  const _ShowUnset({required this.setup, required this.propertyKey});

  final BoardSetup setup;
  final String propertyKey;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<BoardBloc>();
    void toggle() =>
        bloc.add(BoardEvent.showUnsetChanged(show: !setup.showUnset));
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.sm,
        KansoSpacing.md,
        0,
      ),
      child: KansoGroupedSection(
        children: <Widget>[
          Semantics(
            toggled: setup.showUnset,
            child: KansoSettingsRow(
              label: l10n.boardSettingsShowUnset(propertyKey),
              onTap: toggle,
              trailing: ExcludeSemantics(
                child: KansoSwitch(
                  value: setup.showUnset,
                  onChanged: (_) => toggle(),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
