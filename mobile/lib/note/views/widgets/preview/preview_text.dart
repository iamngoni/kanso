// Kanso — preview_text.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../../../core/bridge/api/types.dart';
import 'preview_due_chip.dart';
import 'preview_style.dart';

/// A line of a note's text: its runs in [style], with links that hand their
/// address to the preview, due chips, inline math and images.
///
/// It keeps the recognizers of its links while its text stays the same, so
/// a link held while the text is drawn again still answers its tap, and
/// lets them go when its text changes or it leaves.
class PreviewText extends StatefulWidget {
  const PreviewText(this.inlines, {this.style, this.textAlign, super.key});

  final List<PreviewInline> inlines;

  /// The text's style; the preview's running text when null.
  final TextStyle? style;

  final TextAlign? textAlign;

  @override
  State<PreviewText> createState() => _PreviewTextState();
}

class _PreviewTextState extends State<PreviewText> {
  final List<TapGestureRecognizer> _recognizers = <TapGestureRecognizer>[];

  /// How many of [_recognizers] the text being built has used.
  int _used = 0;

  /// The recognizer of the next link in the text, whose tap hands [url]
  /// over: one kept from the last build when there is one.
  TapGestureRecognizer _recognizer(PreviewScope scope, String url) {
    final TapGestureRecognizer recognizer;
    if (_used < _recognizers.length) {
      recognizer = _recognizers[_used];
    } else {
      recognizer = TapGestureRecognizer();
      _recognizers.add(recognizer);
    }
    _used++;
    return recognizer..onTap = () => scope.onLink(url);
  }

  void _forget() {
    for (final recognizer in _recognizers) {
      recognizer.dispose();
    }
    _recognizers.clear();
  }

  @override
  void didUpdateWidget(PreviewText oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!listEquals(oldWidget.inlines, widget.inlines)) {
      _forget();
    }
  }

  @override
  void dispose() {
    _forget();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = widget.style ?? scope.style.body;
    _used = 0;
    final spans = _spans(scope, style, widget.inlines, null);
    // Links the text no longer has.
    while (_recognizers.length > _used) {
      _recognizers.removeLast().dispose();
    }
    return Text.rich(
      TextSpan(style: style, children: spans),
      textAlign: widget.textAlign,
    );
  }

  List<InlineSpan> _spans(
    PreviewScope scope,
    TextStyle base,
    List<PreviewInline> inlines,
    String? url,
  ) {
    final look = scope.style;
    final recognizer = url == null ? null : _recognizer(scope, url);
    Widget tappable(Widget child) => url == null
        ? child
        : GestureDetector(onTap: () => scope.onLink(url), child: child);
    final size = base.fontSize ?? look.size;
    return <InlineSpan>[
      for (final inline in inlines)
        switch (inline) {
          PreviewInline_Text(
            :final text,
            :final bold,
            :final italic,
            :final strike,
            :final code,
          ) =>
            TextSpan(
              text: text,
              recognizer: recognizer,
              style: TextStyle(
                fontWeight: bold ? FontWeight.w700 : null,
                fontStyle: italic ? FontStyle.italic : null,
                decoration: strike ? TextDecoration.lineThrough : null,
              ).merge(code ? look.inlineCode : null),
            ),
          PreviewInline_Link(:final url, :final inlines) => TextSpan(
            style: look.link,
            children: _spans(scope, base, inlines, url),
          ),
          PreviewInline_Due(
            :final url,
            :final label,
            :final title,
            :final tone,
          ) =>
            WidgetSpan(
              alignment: PlaceholderAlignment.middle,
              child: PreviewDueChip(
                label: label,
                title: title,
                tone: tone,
                size: size,
                url: url,
                onTap: () => scope.onLink(url),
              ),
            ),
          PreviewInline_Math(:final svg) => WidgetSpan(
            alignment: PlaceholderAlignment.baseline,
            baseline: TextBaseline.alphabetic,
            child: tappable(PreviewInlineMath(svg)),
          ),
          PreviewInline_Image(
            :final contentType,
            :final bytes,
            :final alt,
            :final width,
          ) =>
            WidgetSpan(
              child: tappable(
                _InlineImage(
                  contentType: contentType,
                  bytes: bytes,
                  alt: alt,
                  width: width?.toDouble(),
                ),
              ),
            ),
          PreviewInline_Missing(:final text) => TextSpan(
            text: text,
            recognizer: recognizer,
            style: TextStyle(color: look.muted, fontStyle: FontStyle.italic),
          ),
          PreviewInline_Break() => const TextSpan(text: '\n'),
        },
    ];
  }
}

/// Inline math: its SVG set on the text's baseline. The text it is in
/// scales it, and its baseline, with the system's text, as Flutter scales
/// every widget in a line of text.
class PreviewInlineMath extends StatelessWidget {
  const PreviewInlineMath(this.svg, {super.key});

  final PreviewSvg svg;

  @override
  Widget build(BuildContext context) {
    return _Baseline(
      baseline: svg.baseline ?? svg.height,
      child: SizedBox(
        width: svg.width,
        height: svg.height,
        child: SvgPicture.string(svg.markup, width: svg.width),
      ),
    );
  }
}

/// [child], saying its text's baseline is [baseline] below its top.
class _Baseline extends SingleChildRenderObjectWidget {
  const _Baseline({required this.baseline, required super.child});

  final double baseline;

  @override
  RenderObject createRenderObject(BuildContext context) =>
      _RenderBaseline(baseline);

  @override
  void updateRenderObject(BuildContext context, _RenderBaseline renderObject) {
    renderObject.baseline = baseline;
  }
}

class _RenderBaseline extends RenderProxyBox {
  _RenderBaseline(this._baseline);

  double _baseline;
  set baseline(double value) {
    if (value != _baseline) {
      _baseline = value;
      markNeedsLayout();
    }
  }

  @override
  double? computeDistanceToActualBaseline(TextBaseline baseline) => _baseline;
}

/// An attached image, at its width when it has one, never wider than the
/// text.
class _InlineImage extends StatelessWidget {
  const _InlineImage({
    required this.contentType,
    required this.bytes,
    required this.alt,
    this.width,
  });

  final String contentType;
  final Uint8List bytes;
  final String alt;
  final double? width;

  @override
  Widget build(BuildContext context) {
    final data = bytes;
    if (contentType.startsWith('image/svg')) {
      return Semantics(
        label: alt,
        image: true,
        child: SvgPicture.memory(data, width: width),
      );
    }
    return Image.memory(
      data,
      width: width == null ? null : math.max(1, width!),
      semanticLabel: alt,
      fit: BoxFit.contain,
      errorBuilder: (context, error, stack) => Text(
        alt,
        style: TextStyle(
          color: PreviewScope.of(context).style.muted,
          fontStyle: FontStyle.italic,
        ),
      ),
    );
  }
}
