# kanso_ocr

Reads the words in a picture, on the device. Nothing leaves it, no permission
is needed (the bytes come from the app), and it works offline.

```dart
final TextReader reader = KansoOcr();
final text = await reader.read(bytes); // "line one\nline two", or ''
```

`read` takes PNG, JPEG or HEIC/HEIF bytes and returns the recognized lines
joined with `\n`, top to bottom; an empty string when there are no words.
Bytes that are no image throw a `PlatformException` with code `unreadable`
(`KansoOcr.unreadable`); callers treat that as "no words". `engine` is
`vision` on iOS and `mlkit` on Android; `vision` is what the desktop's macOS
reader stores as `kansoOcr.engine`.

| Platform | Engine | Notes |
| --- | --- | --- |
| iOS 17+ | Apple Vision `VNRecognizeTextRequest`, `.accurate`, language correction on | System framework. Background queue; EXIF orientation honoured. |
| Android 10+ (minSdk 29) | Google ML Kit, bundled Latin model | Decoded with `BitmapFactory` (`ImageDecoder` for HEIC on API 28/29), EXIF rotation honoured, longest side capped at 4096 px. |

## Licences and size

- Vision is part of iOS.
- ML Kit is Google's, under its own terms (free, no account):
  `com.google.mlkit:text-recognition:16.0.1`, the **bundled** model (not
  `play-services-mlkit-text-recognition`), so there is no Play services
  dependency and no model download. It adds roughly 4 MB per ABI to the APK
  (an estimate from the artifact; not measured with `--analyze-size`).
- `androidx.exifinterface:exifinterface:1.4.2` (Apache-2.0).

## Tests

- Dart: `flutter test`.
- Android JVM: `./gradlew :kanso_ocr:testDebugUnitTest` from `mobile/android`.
- Android on a device or emulator, which draws "KANSO" and has ML Kit read
  it: `./gradlew :kanso_ocr:connectedDebugAndroidTest`.
- iOS: `KansoOcrCoreTests` in the Swift package, which renders "KANSO" and has
  Vision read it. The `KansoOcrCore` target has no Flutter dependency; the
  package's `FlutterFramework` dependency is supplied by the app build, so run
  it from a copy of the package with a stub `FlutterFramework` and
  `xcodebuild test -scheme kanso_ocr -destination 'platform=iOS Simulator,name=...'`.

Regenerate the channel with `dart run pigeon --input pigeons/messages.dart`.
