// Kanso — fakes.dart (PDF reader tests)
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:typed_data';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/pdf_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/pdf_reader/models/data/pdf_opening.dart';
import 'package:kanso/pdf_reader/models/repos/abs/pdf_reader_repository.dart';

/// A preferences repository the reader tests can seed and inspect.
class FakePdfPreferencesRepository implements PreferencesRepository {
  FakePdfPreferencesRepository([this.current = const Preferences()]);

  Preferences current;
  int updates = 0;

  @override
  Future<Either<ApplicationError, Preferences>> load() async =>
      Right<ApplicationError, Preferences>(current);

  @override
  Future<Either<ApplicationError, Preferences>> update(
    Preferences Function(Preferences current) change,
  ) async {
    updates++;
    current = change(current);
    return Right<ApplicationError, Preferences>(current);
  }
}

/// A page picture a test can draw: white, with grey lines where a page of
/// text would have them, in the reader's BGRA order.
PdfPageImage fakePageImage({
  int width = 612,
  int height = 792,
  bool inverted = false,
}) {
  final bytes = Uint8List(width * height * 4);
  final paper = inverted ? 24 : 255;
  final ink = inverted ? 220 : 170;
  for (var i = 0; i < bytes.length; i += 4) {
    bytes[i] = paper;
    bytes[i + 1] = paper;
    bytes[i + 2] = paper;
    bytes[i + 3] = 255;
  }
  // A page of text, in the page's own proportions: margins, line height and
  // the short last line of a paragraph all scale with the picture.
  final margin = (width * 0.1).round();
  final lineHeight = math.max(2, (height * 0.008).round());
  final gap = math.max(2, (height * 0.018).round());
  var line = 0;
  for (var top = margin; top + lineHeight < height - margin; line++) {
    final short = line % 5 == 4;
    final right = width - margin - (short ? (width * 0.18).round() : 0);
    for (var y = top; y < top + lineHeight; y++) {
      for (var x = margin; x < right; x++) {
        final at = (y * width + x) * 4;
        bytes[at] = ink;
        bytes[at + 1] = ink;
        bytes[at + 2] = ink;
      }
    }
    top += lineHeight + gap;
  }
  return PdfPageImage(
    page: 0,
    width: width,
    height: height,
    bgra: bytes,
  );
}

/// One page of text, as the reader answers it.
const PdfInfo fakePdfInfo = PdfInfo(
  file: 'file:pdf-one',
  name: 'Field notes.pdf',
  pages: 3,
  sizes: <PdfPageSize>[
    PdfPageSize(width: 612, height: 792),
    PdfPageSize(width: 612, height: 792),
    PdfPageSize(width: 792, height: 612),
  ],
  encrypted: false,
  noteId: 'note-one',
  editable: true,
);

PdfRect _rect(double x0, double y0, double x1, double y1) =>
    PdfRect(x0: x0, y0: y0, x1: x1, y1: y1);

/// A selection of "the words" on page 1.
final PdfSelection fakeSelection = PdfSelection(
  start: const PdfCharPos(page: 1, index: 4),
  end: const PdfCharPos(page: 1, index: 18),
  text: 'the words of a page',
  marks: <PdfPageMarks>[
    PdfPageMarks(
      page: 1,
      rects: <PdfRect>[_rect(0.1, 0.2, 0.8, 0.23), _rect(0.1, 0.24, 0.5, 0.27)],
    ),
  ],
);

/// A highlight on page 1 and one on page 3.
const PdfMark fakeMark = PdfMark(
  id: 'mark-1',
  color: 'yellow',
  page: 1,
  marks: <PdfPageMarks>[],
);

/// The note's highlights for [fakePdfInfo].
PdfMarks fakeMarks({String rev = 'rev-1', bool editable = true}) => PdfMarks(
  rev: rev,
  editable: editable,
  marks: <PdfMark>[
    PdfMark(
      id: 'mark-1',
      color: 'yellow',
      page: 1,
      marks: <PdfPageMarks>[
        PdfPageMarks(page: 1, rects: <PdfRect>[_rect(0.1, 0.2, 0.8, 0.23)]),
      ],
    ),
    PdfMark(
      id: 'mark-2',
      color: 'blue',
      page: 3,
      marks: <PdfPageMarks>[
        PdfPageMarks(page: 3, rects: <PdfRect>[_rect(0.2, 0.5, 0.6, 0.53)]),
      ],
    ),
  ],
  ink: const <PdfInkPage>[],
);

/// A document open for reading, answering what a test set up.
class FakePdfSession implements PdfDocumentSession {
  FakePdfSession({
    PdfSelection? selection,
    this.line,
    this.found,
    this.pageImage,
  }) : selection = selection ?? fakeSelection;

  PdfSelection? selection;
  PdfSelection? line;
  PdfFound? found;
  PdfPageImage? pageImage;

  final List<PdfPageImage> rendered = <PdfPageImage>[];
  final List<
    ({
      String baseRev,
      PdfCharPos start,
      PdfCharPos end,
      String color,
      bool quote,
    })
  >
  highlights =
      <
        ({
          String baseRev,
          PdfCharPos start,
          PdfCharPos end,
          String color,
          bool quote,
        })
      >[];
  final List<({String baseRev, int page, PdfInkStroke stroke})> inkStrokes =
      <({String baseRev, int page, PdfInkStroke stroke})>[];
  String inkRev = 'rev-1';
  List<PdfInkPage> inkPages = <PdfInkPage>[];
  bool disposed = false;

  @override
  Future<Either<ApplicationError, PdfPageImage>> render(
    int page,
    int widthPx, {
    required bool invert,
  }) async {
    final image = pageImage ?? fakePageImage(inverted: invert);
    rendered.add(image);
    return Right<ApplicationError, PdfPageImage>(image);
  }

  @override
  Future<Either<ApplicationError, PdfSelection?>> wordAt(
    int page,
    double x,
    double y,
  ) async => Right<ApplicationError, PdfSelection?>(selection);

  @override
  Future<Either<ApplicationError, PdfSelection?>> lineAt(
    int page,
    double x,
    double y,
  ) async => Right<ApplicationError, PdfSelection?>(line ?? selection);

  @override
  Future<Either<ApplicationError, PdfSelection?>> extend(
    PdfCharPos anchor,
    int page,
    double x,
    double y,
  ) async => Right<ApplicationError, PdfSelection?>(selection);

  @override
  Future<Either<ApplicationError, PdfFound>> find(
    String query,
    int limit,
  ) async => Right<ApplicationError, PdfFound>(
    found ??
        PdfFound(
          total: 2,
          hits: <PdfFindHit>[
            PdfFindHit(
              page: 1,
              marks: <PdfPageMarks>[
                PdfPageMarks(
                  page: 1,
                  rects: <PdfRect>[_rect(0.1, 0.3, 0.4, 0.33)],
                ),
              ],
            ),
            PdfFindHit(
              page: 3,
              marks: <PdfPageMarks>[
                PdfPageMarks(
                  page: 3,
                  rects: <PdfRect>[_rect(0.2, 0.6, 0.5, 0.63)],
                ),
              ],
            ),
          ],
          pagesWithoutText: 1,
        ),
  );

  @override
  Future<Either<ApplicationError, PdfEdited>> addHighlight({
    required String baseRev,
    required PdfCharPos start,
    required PdfCharPos end,
    required String color,
    required bool quote,
  }) async {
    highlights.add((
      baseRev: baseRev,
      start: start,
      end: end,
      color: color,
      quote: quote,
    ));
    return const Right<ApplicationError, PdfEdited>(
      PdfEdited(rev: 'rev-2', id: 'mark-3', merged: false, gone: false),
    );
  }

  @override
  Future<Either<ApplicationError, PdfEdited>> addInkStroke({
    required String baseRev,
    required int page,
    required PdfInkStroke stroke,
  }) async {
    inkStrokes.add((baseRev: baseRev, page: page, stroke: stroke));
    inkRev = 'rev-${inkStrokes.length + 1}';
    inkPages = <PdfInkPage>[
      for (final inkPage in inkPages)
        if (inkPage.page != page) inkPage,
      PdfInkPage(
        page: page,
        strokes: <PdfInkStroke>[
          ...inkPages
              .where((inkPage) => inkPage.page == page)
              .expand((inkPage) => inkPage.strokes),
          stroke,
        ],
      ),
    ];
    return Right<ApplicationError, PdfEdited>(
      PdfEdited(rev: inkRev, merged: baseRev != 'rev-1', gone: false),
    );
  }

  @override
  void dispose() {
    disposed = true;
  }
}

/// A [PdfReaderRepository] answering as a test set up.
class FakePdfReaderRepository implements PdfReaderRepository {
  FakePdfReaderRepository({
    this.opening,
    PdfMarks? marks,
    this.onOpen,
    this.openError,
    this.marksError,
  }) : marksAnswer = marks;

  /// What opening comes to; a session is made when null.
  PdfOpening? opening;

  /// Answers opening with the password it received, when set.
  PdfOpening Function(String? password)? onOpen;

  PdfMarks? marksAnswer;
  final List<String?> passwords = <String?>[];
  final List<String> files = <String>[];
  FakePdfSession? session;
  int opens = 0;
  ApplicationError? openError;
  ApplicationError? marksError;

  @override
  Future<Either<ApplicationError, PdfOpening>> open({
    required String noteId,
    required String file,
    String? password,
  }) async {
    opens++;
    files.add(file);
    passwords.add(password);
    final error = openError;
    if (error != null) {
      return Left<ApplicationError, PdfOpening>(error);
    }
    final answer = onOpen?.call(password) ?? opening;
    if (answer != null) {
      return Right<ApplicationError, PdfOpening>(answer);
    }
    final made = session ?? FakePdfSession();
    session = made;
    return Right<ApplicationError, PdfOpening>(
      PdfOpening.opened(session: made, info: fakePdfInfo),
    );
  }

  @override
  Future<Either<ApplicationError, PdfMarks>> marks({
    required String noteId,
    required String file,
  }) async {
    final error = marksError;
    if (error != null) {
      return Left<ApplicationError, PdfMarks>(error);
    }
    final marks = marksAnswer ?? fakeMarks();
    final opened = opening;
    final openedSession = switch (opened) {
      PdfOpened(:final session) => session,
      _ => session,
    };
    final inkSession = openedSession is FakePdfSession ? openedSession : null;
    final revision = inkSession?.inkRev ?? marksAnswer?.rev ?? 'rev-1';
    return Right<ApplicationError, PdfMarks>(
      PdfMarks(
        rev: revision,
        editable: marks.editable,
        why: marks.why,
        marks: marks.marks,
        ink: inkSession?.inkPages ?? marks.ink,
      ),
    );
  }
}
