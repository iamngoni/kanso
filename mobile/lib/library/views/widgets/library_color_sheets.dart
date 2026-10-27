// Kanso — library_color_sheets.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart' show Either;
import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_text_input.dart';
import '../../../injection.dart';

/// What the colour picker gave back: `#rrggbb`, or null for no colour of
/// its own.
class ColorPick {
  const ColorPick(this.hex);

  final String? hex;
}

/// The title row every picker shares: its name and a way out.
class _PickerHeader extends StatelessWidget {
  const _PickerHeader({required this.title});

  final String title;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Row(
      children: <Widget>[
        Expanded(
          child: Text(title, style: context.typography.barTitle),
        ),
        KansoPressable(
          onTap: () => Navigator.of(context).maybePop(),
          child: Padding(
            padding: const EdgeInsets.all(KansoSpacing.sm),
            child: Text(
              l10n.searchCancel,
              style: context.typography.body.copyWith(color: colors.accent),
            ),
          ),
        ),
      ],
    );
  }
}

/// One swatch of a picker: its colour, or the crossed circle that stands
/// for none, with a ring while it is the one in use.
class _Swatch extends StatelessWidget {
  const _Swatch({
    required this.color,
    required this.selected,
    required this.label,
    required this.onTap,
    this.none = false,
  });

  final Color? color;
  final bool selected;
  final String label;
  final VoidCallback onTap;

  /// Whether this is the "no colour" swatch.
  final bool none;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final fill = color;
    return KansoPressable(
      onTap: onTap,
      semanticLabel: label,
      child: Container(
        width: KansoSizes.touchTarget,
        height: KansoSizes.touchTarget,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          border: Border.all(
            color: selected ? colors.accent : colors.divider,
            width: selected ? 2 : 1,
          ),
          borderRadius: KansoRadii.smallBorder,
        ),
        child: Container(
          width: 26,
          height: 26,
          decoration: BoxDecoration(
            color: fill,
            shape: BoxShape.circle,
            border: Border.all(color: colors.divider),
          ),
          child: none
              ? Icon(Icons.close, size: 16, color: colors.textMuted)
              : null,
        ),
      ),
    );
  }
}

/// The colours a notebook's row can be washed in, as the desktop's grid has
/// them: no colour, the presets, and any colour typed as hex.
class NotebookColorSheet extends StatefulWidget {
  const NotebookColorSheet({this.current, super.key});

  /// The `#rrggbb` the notebook keeps now, if any.
  final String? current;

  /// Shows the picker over the whole app; null when it was closed without
  /// a choice.
  static Future<ColorPick?> show(BuildContext context, {String? current}) {
    return showKansoBottomSheet<ColorPick>(
      context,
      builder: (_) => NotebookColorSheet(current: current),
    );
  }

  @override
  State<NotebookColorSheet> createState() => _NotebookColorSheetState();
}

class _NotebookColorSheetState extends State<NotebookColorSheet> {
  final LibraryRepository library = getIt<LibraryRepository>();
  final TextEditingController hex = TextEditingController();
  List<ColorChoice> swatches = const <ColorChoice>[];
  bool invalid = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    hex.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    final Either<ApplicationError, List<ColorChoice>> result =
        await library.notebookColors();
    if (!mounted) {
      return;
    }
    setState(() {
      swatches = result.getOrElse(() => const <ColorChoice>[]);
    });
  }

  void submitHex() {
    final parsed = library.parseColor(hex.text);
    if (parsed == null) {
      setState(() => invalid = true);
      return;
    }
    Navigator.of(context).pop(ColorPick(parsed));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final mine = widget.current?.toLowerCase();
    return Padding(
      padding: const EdgeInsetsDirectional.fromSTEB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          _PickerHeader(title: l10n.libraryChangeColorTitle),
          const SizedBox(height: KansoSpacing.xs),
          SizedBox(
            height: 190,
            child: GridView.count(
              crossAxisCount: 7,
              mainAxisSpacing: KansoSpacing.xxs,
              crossAxisSpacing: KansoSpacing.xxs,
              padding: EdgeInsets.zero,
              children: <Widget>[
                _Swatch(
                  color: null,
                  none: true,
                  selected: widget.current == null,
                  label: l10n.libraryColorDefault,
                  onTap: () =>
                      Navigator.of(context).pop(const ColorPick(null)),
                ),
                for (final swatch in swatches)
                  _Swatch(
                    color: Color(0xFF000000 | swatch.rgb),
                    selected: mine == _hexOf(swatch.rgb),
                    label: swatch.name,
                    onTap: () => Navigator.of(
                      context,
                    ).pop(ColorPick(_hexOf(swatch.rgb))),
                  ),
              ],
            ),
          ),
          const SizedBox(height: KansoSpacing.sm),
          KansoTextInput(
            controller: hex,
            hint: l10n.libraryColorHexHint,
            semanticLabel: l10n.libraryColorHexHint,
            onChanged: (_) {
              if (invalid) {
                setState(() => invalid = false);
              }
            },
            onSubmitted: (_) => submitHex(),
          ),
          if (invalid)
            Padding(
              padding: const EdgeInsets.only(top: KansoSpacing.xxs),
              child: Text(
                l10n.libraryColorInvalid,
                style: context.typography.secondary.copyWith(
                  color: colors.danger,
                ),
              ),
            ),
          const SizedBox(height: KansoSpacing.sm),
          KansoButton(
            label: l10n.librarySaveAction,
            onPressed: submitHex,
          ),
        ],
      ),
    );
  }
}

/// The colours a tag can have, by the names the desktop writes: every one,
/// including the default that takes a colour away.
class TagColorSheet extends StatefulWidget {
  const TagColorSheet({required this.current, super.key});

  /// The colour the tag is drawn in now, as `0xRRGGBB`.
  final int current;

  /// Shows the colours over the whole app; the colour chosen by name, or
  /// null when it was closed without a choice.
  static Future<String?> show(BuildContext context, {required int current}) {
    return showKansoBottomSheet<String>(
      context,
      builder: (_) => TagColorSheet(current: current),
    );
  }

  @override
  State<TagColorSheet> createState() => _TagColorSheetState();
}

class _TagColorSheetState extends State<TagColorSheet> {
  final LibraryRepository library = getIt<LibraryRepository>();
  List<ColorChoice> colors = const <ColorChoice>[];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final Either<ApplicationError, List<ColorChoice>> result =
        await library.tagColors();
    if (!mounted) {
      return;
    }
    setState(() {
      colors = result.getOrElse(() => const <ColorChoice>[]);
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Padding(
      padding: const EdgeInsetsDirectional.fromSTEB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          _PickerHeader(title: l10n.libraryTagColor),
          const SizedBox(height: KansoSpacing.xs),
          SizedBox(
            height: 190,
            child: GridView.count(
              crossAxisCount: 7,
              mainAxisSpacing: KansoSpacing.xxs,
              crossAxisSpacing: KansoSpacing.xxs,
              padding: EdgeInsets.zero,
              children: <Widget>[
                for (final color in colors)
                  _Swatch(
                    color: color.rgb == 0
                        ? null
                        : Color(0xFF000000 | color.rgb),
                    none: color.rgb == 0,
                    selected: color.rgb == widget.current,
                    label: color.name,
                    onTap: () => Navigator.of(context).pop(color.name),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// `rgb` as the colour picker keeps it: `#rrggbb`, lower case.
String _hexOf(int rgb) =>
    '#${(rgb & 0xffffff).toRadixString(16).padLeft(6, '0')}';
