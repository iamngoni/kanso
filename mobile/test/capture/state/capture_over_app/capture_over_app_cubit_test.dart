// Kanso — capture_over_app_cubit_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/models/data/capture_over_app_status.dart';
import 'package:kanso/capture/models/data/media_source.dart';
import 'package:kanso/capture/state/capture_over_app/capture_over_app_cubit.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fake_media_picker_repository.dart';
import '../../../helpers/mock_repositories.dart';

final ShareFile _photo = ShareFile(
  name: 'photo.jpg',
  contentType: 'image/jpeg',
  bytes: Uint8List.fromList(<int>[1, 2, 3]),
);

void main() {
  late MockCaptureRepository capture;
  late FakeMediaPickerRepository picker;

  setUpAll(() => registerFallbackValue(const Share(files: <ShareFile>[])));

  setUp(() {
    capture = MockCaptureRepository();
    picker = FakeMediaPickerRepository();
    when(() => capture.capture(any())).thenAnswer(
      (_) async => const Right<ApplicationError, CaptureOutcome>(
        CaptureOutcome(noteId: 'note:cap-1', problems: <String>[]),
      ),
    );
  });

  CaptureOverAppCubit make() {
    final cubit = CaptureOverAppCubit(capture, picker);
    addTearDown(cubit.close);
    return cubit;
  }

  test('starts idle, which is not finished', () {
    expect(make().state, CaptureOverAppStatus.idle);
    expect(CaptureOverAppStatus.idle.finished, isFalse);
    expect(CaptureOverAppStatus.working.finished, isFalse);
  });

  test('what was picked is saved as one note in the Inbox', () async {
    picker.answer = Right<ApplicationError, List<ShareFile>>(<ShareFile>[
      _photo,
    ]);
    final cubit = make();
    final states = <CaptureOverAppStatus>[];
    final sub = cubit.stream.listen(states.add);
    await cubit.sourceChosen(MediaSource.photos);
    // The stream hands on the last state in a microtask.
    await Future<void>.delayed(Duration.zero);
    await sub.cancel();
    expect(states, <CaptureOverAppStatus>[
      CaptureOverAppStatus.working,
      CaptureOverAppStatus.saved,
    ]);
    expect(picker.sources, <MediaSource>[MediaSource.photos]);
    final shared = verify(() => capture.capture(captureAny())).captured.single;
    expect((shared as Share).files, <ShareFile>[_photo]);
  });

  test('nothing picked saves nothing', () async {
    final cubit = make();
    await cubit.sourceChosen(MediaSource.camera);
    expect(cubit.state, CaptureOverAppStatus.cancelled);
    expect(cubit.state.finished, isTrue);
    verifyNever(() => capture.capture(any()));
  });

  test('a locked Inbox is said as locked', () async {
    picker.answer = Right<ApplicationError, List<ShareFile>>(<ShareFile>[
      _photo,
    ]);
    when(() => capture.capture(any())).thenAnswer(
      (_) async => const Left<ApplicationError, CaptureOutcome>(
        ApplicationError.locked(message: 'locked'),
      ),
    );
    final cubit = make();
    await cubit.sourceChosen(MediaSource.pdf);
    expect(cubit.state, CaptureOverAppStatus.locked);
  });

  test('a picker or a save that fails is said as failed', () async {
    picker.answer = const Left<ApplicationError, List<ShareFile>>(
      ApplicationError.unexpected(message: 'no camera'),
    );
    final cubit = make();
    await cubit.sourceChosen(MediaSource.camera);
    expect(cubit.state, CaptureOverAppStatus.failed);

    picker.answer = Right<ApplicationError, List<ShareFile>>(<ShareFile>[
      _photo,
    ]);
    when(() => capture.capture(any())).thenAnswer(
      (_) async => const Left<ApplicationError, CaptureOutcome>(
        ApplicationError.unexpected(message: 'disk'),
      ),
    );
    final again = make();
    await again.sourceChosen(MediaSource.photos);
    expect(again.state, CaptureOverAppStatus.failed);
  });

  test('a second choice while one is under way is ignored', () async {
    picker.answer = Right<ApplicationError, List<ShareFile>>(<ShareFile>[
      _photo,
    ]);
    final cubit = make();
    final first = cubit.sourceChosen(MediaSource.photos);
    await cubit.sourceChosen(MediaSource.pdf);
    await first;
    expect(picker.sources, <MediaSource>[MediaSource.photos]);
    verify(() => capture.capture(any())).called(1);
  });
}
