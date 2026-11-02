// Kanso — presentation_deck.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import 'preview/native_preview.dart';
import 'preview/preview_style.dart';

/// How long moving to another slide takes.
const Duration _slideChange = Duration(milliseconds: 280);

/// The note's [blocks] as slides, by the rules the desktop cuts its text by
/// (`kanso_core::slides`): a rule starts a new slide and isn't on either; a
/// note with no rule starts one at each top-level heading of level 1 or 2.
/// Empty slides are left out. The blocks are the core's, so a rule in a code
/// block, or `---` under a line of text, was never a rule.
List<List<PreviewBlock>> slidesOf(List<PreviewBlock> blocks) {
  final byRules = blocks.any((block) => block is PreviewBlock_Rule);
  final slides = <List<PreviewBlock>>[];
  var slide = <PreviewBlock>[];
  void finish() {
    if (slide.isNotEmpty) {
      slides.add(slide);
      slide = <PreviewBlock>[];
    }
  }

  for (final block in blocks) {
    if (block is PreviewBlock_Rule) {
      finish();
      continue;
    }
    if (!byRules && block is PreviewBlock_Heading && block.level <= 2) {
      finish();
    }
    slide.add(block);
  }
  finish();
  return slides;
}

/// How much larger than the note slide text is on a screen of [size]: the
/// note's own size on a phone, either way up, and larger on a tablet.
double slideTextScale(Size size) =>
    math.min(size.width / 640, size.height / 400).clamp(1.0, 2.5);

/// The room beside a slide on a screen [width] wide.
double _margin(double width) => width * 0.08;

/// [slides] one at a time, full screen, in the note's preview palette. A
/// swipe or a tap on a side moves between them; a slide too long for the
/// screen scrolls. A keyboard's arrows, Page Up, Page Down, Space, Home and
/// End move too, and Esc asks to close. Tapping changes nothing in the note.
class PresentationDeck extends StatefulWidget {
  const PresentationDeck({
    required this.slides,
    required this.palette,
    required this.onClose,
    super.key,
  });

  final List<List<PreviewBlock>> slides;

  /// The preview theme's colours, fitted with the interface and syntax
  /// themes in force.
  final PaletteDto palette;

  final VoidCallback onClose;

  @override
  State<PresentationDeck> createState() => _PresentationDeckState();
}

class _PresentationDeckState extends State<PresentationDeck> {
  final PageController _pages = PageController();
  int _current = 0;

  int get _last => math.max(widget.slides.length - 1, 0);

  /// The slide on show, should an edit elsewhere have taken slides away.
  int get _shown => math.min(_current, _last);

  @override
  void dispose() {
    _pages.dispose();
    super.dispose();
  }

  void _go(int slide) {
    final target = slide.clamp(0, _last);
    if (target != _shown && _pages.hasClients) {
      unawaited(
        _pages.animateToPage(
          target,
          duration: _slideChange,
          curve: Curves.easeOutCubic,
        ),
      );
    }
  }

  void _tapped(TapUpDetails details, double width) {
    _go(details.localPosition.dx < width / 3 ? _shown - 1 : _shown + 1);
  }

  KeyEventResult _key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }
    final key = event.logicalKey;
    if (key == LogicalKeyboardKey.arrowRight ||
        key == LogicalKeyboardKey.pageDown ||
        key == LogicalKeyboardKey.space) {
      _go(_shown + 1);
    } else if (key == LogicalKeyboardKey.arrowLeft ||
        key == LogicalKeyboardKey.pageUp) {
      _go(_shown - 1);
    } else if (key == LogicalKeyboardKey.home) {
      _go(0);
    } else if (key == LogicalKeyboardKey.end) {
      _go(_last);
    } else if (key == LogicalKeyboardKey.escape) {
      widget.onClose();
    } else {
      return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final style = PreviewStyle(
      palette: widget.palette,
      typography: context.typography,
    );
    final count = widget.slides.length;
    final position = count == 0 ? 0 : _shown + 1;
    return Focus(
      autofocus: true,
      onKeyEvent: _key,
      child: ColoredBox(
        color: style.background,
        child: SafeArea(
          child: Stack(
            children: <Widget>[
              Positioned.fill(
                child: count == 0
                    ? Center(
                        child: Text(
                          l10n.presentationEmpty,
                          style: context.typography.secondary,
                        ),
                      )
                    : LayoutBuilder(
                        builder: (context, box) => GestureDetector(
                          behavior: HitTestBehavior.translucent,
                          onTapUp: (details) => _tapped(details, box.maxWidth),
                          child: PageView.builder(
                            controller: _pages,
                            itemCount: count,
                            onPageChanged: (slide) =>
                                setState(() => _current = slide),
                            itemBuilder: (context, slide) => _Slide(
                              key: ValueKey<int>(slide),
                              blocks: widget.slides[slide],
                              palette: widget.palette,
                            ),
                          ),
                        ),
                      ),
              ),
              PositionedDirectional(
                top: 4,
                end: 4,
                child: KansoIconButton(
                  icon: KansoIcons.close,
                  semanticLabel: l10n.presentationClose,
                  onPressed: widget.onClose,
                ),
              ),
              if (count > 0)
                PositionedDirectional(
                  bottom: 10,
                  end: 16,
                  child: Text(
                    l10n.presentationPosition(position, count),
                    style: context.typography.secondary.copyWith(
                      color: style.muted,
                    ),
                  ),
                ),
              if (count > 0)
                Positioned(
                  left: 0,
                  right: 0,
                  bottom: 0,
                  child: Align(
                    alignment: AlignmentDirectional.centerStart,
                    child: FractionallySizedBox(
                      widthFactor: position / count,
                      child: SizedBox(
                        height: 3,
                        child: ColoredBox(color: style.accent),
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

/// One slide's blocks, in a scroll of its own, set larger on a large screen.
class _Slide extends StatelessWidget {
  const _Slide({required this.blocks, required this.palette, super.key});

  final List<PreviewBlock> blocks;
  final PaletteDto palette;

  @override
  Widget build(BuildContext context) {
    final size = MediaQuery.sizeOf(context);
    final people = MediaQuery.textScalerOf(context).scale(1);
    final margin = _margin(size.width);
    return MediaQuery(
      data: MediaQuery.of(
        context,
      ).copyWith(textScaler: TextScaler.linear(people * slideTextScale(size))),
      child: CustomScrollView(
        slivers: <Widget>[
          SliverPadding(
            padding: EdgeInsets.fromLTRB(margin, 56, margin, 48),
            sliver: NativePreview(
              blocks: blocks,
              palette: palette,
              onLink: _ignore,
            ),
          ),
        ],
      ),
    );
  }

  /// A tap on a slide moves to the next one: boxes, links and copy buttons on
  /// it do nothing, so presenting can't change the note.
  static void _ignore(String url) {}
}
