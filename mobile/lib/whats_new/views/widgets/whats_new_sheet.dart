// Kanso — whats_new_sheet.dart
// Created 2026-11-12.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/whats_new_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/sheet_header.dart';

/// What's new in a release, as the desktop shows it: the release's video
/// on top (an animated WebP, muted and looping), then the title, a version
/// chip and the date, a one-line summary and the notes. Releases skipped
/// since the last update follow, folded. Without a video, the notes lead.
class WhatsNewSheet extends StatefulWidget {
  const WhatsNewSheet({required this.whatsNew, super.key});

  final PhoneWhatsNew whatsNew;

  @override
  State<WhatsNewSheet> createState() => _WhatsNewSheetState();
}

class _WhatsNewSheetState extends State<WhatsNewSheet> {
  bool _earlierOpen = false;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final whatsNew = widget.whatsNew;
    const inset = EdgeInsetsDirectional.fromSTEB(
      KansoSpacing.md,
      KansoSpacing.sm,
      KansoSpacing.md,
      0,
    );
    return ListView(
      shrinkWrap: true,
      padding: const EdgeInsets.only(bottom: KansoSpacing.lg),
      children: <Widget>[
        SheetHeader(
          title: l10n.whatsNewSheetTitle,
          actionLabel: l10n.noticeSheetDone,
        ),
        if (whatsNew.videoPath.isNotEmpty)
          Padding(
            padding: inset,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(12),
              child: AspectRatio(
                aspectRatio: 16 / 9,
                child: ColoredBox(
                  color: colors.fill,
                  child: Image.file(
                    File(whatsNew.videoPath),
                    key: const ValueKey<String>('whats-new-video'),
                    fit: BoxFit.cover,
                    gaplessPlayback: true,
                    excludeFromSemantics: true,
                    // A file that can't be read leaves the card empty
                    // rather than showing an error.
                    errorBuilder: (_, _, _) => const SizedBox.shrink(),
                  ),
                ),
              ),
            ),
          ),
        Padding(
          padding: inset,
          child: Semantics(
            header: true,
            child: Text(
              l10n.whatsNewTitle(whatsNew.version),
              style: typography.heading2,
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            0,
          ),
          child: Row(
            children: <Widget>[
              Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.xs,
                  vertical: 2,
                ),
                decoration: BoxDecoration(
                  color: colors.text.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(999),
                ),
                child: Text(
                  'v${whatsNew.version}',
                  style: typography.meta.copyWith(color: colors.text),
                ),
              ),
              if (_date(context, whatsNew.date) case final date?) ...<Widget>[
                const SizedBox(width: KansoSpacing.xs),
                Text(date, style: typography.meta),
              ],
            ],
          ),
        ),
        if (whatsNew.summary.isNotEmpty)
          Padding(
            padding: inset,
            child: Text(whatsNew.summary, style: typography.secondary),
          ),
        for (final section in whatsNew.sections) ..._section(context, section),
        if (whatsNew.earlier.isNotEmpty) ...<Widget>[
          Padding(
            padding: const EdgeInsetsDirectional.fromSTEB(
              KansoSpacing.xs,
              KansoSpacing.md,
              KansoSpacing.md,
              0,
            ),
            child: Semantics(
              button: true,
              expanded: _earlierOpen,
              child: Material(
                type: MaterialType.transparency,
                child: InkWell(
                  key: const ValueKey<String>('whats-new-earlier'),
                  borderRadius: BorderRadius.circular(8),
                  onTap: () => setState(() => _earlierOpen = !_earlierOpen),
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.xs),
                    child: Row(
                      children: <Widget>[
                        Icon(
                          _earlierOpen
                              ? Icons.expand_more_rounded
                              : Icons.chevron_right_rounded,
                          size: 20,
                          color: colors.textSecondary,
                        ),
                        const SizedBox(width: KansoSpacing.xxs),
                        Expanded(
                          child: Text(
                            l10n.whatsNewEarlier(whatsNew.earlier.length),
                            style: typography.label,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
          if (_earlierOpen)
            for (final earlier in whatsNew.earlier) ...<Widget>[
              Padding(
                padding: inset,
                child: Text(
                  l10n.whatsNewEarlierVersion(earlier.version),
                  style: typography.listTitle,
                ),
              ),
              for (final section in earlier.sections)
                ..._section(context, section),
            ],
        ],
      ],
    );
  }

  List<Widget> _section(BuildContext context, PhoneWhatsNewSection section) {
    final colors = context.colors;
    final typography = context.typography;
    return <Widget>[
      Padding(
        padding: const EdgeInsetsDirectional.fromSTEB(
          KansoSpacing.md,
          KansoSpacing.md,
          KansoSpacing.md,
          KansoSpacing.xxs,
        ),
        child: Text(section.title, style: typography.sectionLabel),
      ),
      for (final item in section.items)
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.xxs,
            KansoSpacing.md,
            0,
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Container(
                  width: 5,
                  height: 5,
                  decoration: BoxDecoration(
                    color: colors.textSecondary,
                    shape: BoxShape.circle,
                  ),
                ),
              ),
              const SizedBox(width: KansoSpacing.xs),
              Expanded(child: Text(item, style: typography.body)),
            ],
          ),
        ),
    ];
  }

  /// `YYYY-MM-DD` in the phone's words, or null.
  static String? _date(BuildContext context, String date) {
    final parsed = DateTime.tryParse(date);
    if (parsed == null) {
      return null;
    }
    return MaterialLocalizations.of(context).formatMediumDate(parsed);
  }
}
