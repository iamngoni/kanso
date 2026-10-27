// Kanso — capture_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../models/data/capture_first_row.dart';
import '../../models/data/media_source.dart';

/// What can be added from the phone: [first] (a new note, from the list's
/// button; an image link, from the editor's), then a photo taken now,
/// photos from the library, or a PDF. A choice closes the sheet, then says
/// which it was.
class CaptureSheet extends StatelessWidget {
  const CaptureSheet({required this.first, required this.onSource, super.key});

  /// Shows the sheet over [context].
  static Future<void> show(
    BuildContext context, {
    required CaptureFirstRow first,
    required ValueChanged<MediaSource> onSource,
  }) => showKansoBottomSheet<void>(
    context,
    builder: (_) => CaptureSheet(first: first, onSource: onSource),
  );

  /// The row above the camera's, with what it does.
  final CaptureFirstRow first;

  final ValueChanged<MediaSource> onSource;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    void choose(VoidCallback then) {
      Navigator.of(context).pop();
      then();
    }

    return SafeArea(
      top: false,
      child: Padding(
        padding: const EdgeInsets.only(bottom: KansoSpacing.md),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            KansoSheetGroup(
              children: <Widget>[
                KansoSettingsRow(
                  icon: first.icon,
                  label: first.label,
                  onTap: () => choose(first.onTap),
                ),
              ],
            ),
            KansoSheetGroup(
              children: <Widget>[
                KansoSettingsRow(
                  icon: KansoIcons.camera,
                  label: l10n.captureTakePhoto,
                  onTap: () => choose(() => onSource(MediaSource.camera)),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.image,
                  label: l10n.captureChoosePhotos,
                  onTap: () => choose(() => onSource(MediaSource.photos)),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.fileText,
                  label: l10n.captureAttachPdf,
                  onTap: () => choose(() => onSource(MediaSource.pdf)),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
