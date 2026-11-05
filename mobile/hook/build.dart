// Kanso — build.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:code_assets/code_assets.dart';
import 'package:flutter_rust_bridge_hooks/flutter_rust_bridge_hooks.dart';
import 'package:path/path.dart' as p;

/// Builds the Rust library (`rust/`, crate `kanso-mobile-bridge`) for the
/// platform Flutter is building for, and hands it over as a native asset.
///
/// `flutter run`, `flutter build` and `flutter test` run this. The asset name
/// is the path of the generated binding (`lib/core/bridge/frb_generated.io.dart`)
/// so its `ExternalLibrary.open` finds the right library.
///
/// What it adds to the builder's defaults:
///
/// - On iOS and Android, the cargo profile of `[profile.mobile]` in the root
///   `Cargo.toml` (size-optimised, one codegen unit, fat LTO, unwinding
///   panics). The builder always builds `--release`, so the same settings are
///   given as `CARGO_PROFILE_RELEASE_*`. Keep the two lists the same.
/// - On every platform, `--locked`: the build uses the workspace's
///   `Cargo.lock`, as CI does.
/// - On iOS and Android, the compilers and system headers of
///   `scripts/mobile/env.sh`, the one place that knows them. The builder
///   links Android with the NDK's API 35 compiler, which would load on
///   Android 10 (API 29, the app's minimum) only until the first symbol that
///   Android 10 lacks; `env.sh` names the API 29 ones.
///
/// A computer's own build (`flutter test`) stays a plain release build: no
/// fat LTO, so tests start sooner.
Future<void> main(List<String> args) async {
  await build(args, (input, output) async {
    // `flutter run` also runs the hooks for data assets only; there is no
    // library to build then.
    if (!input.config.buildCodeAssets) {
      return;
    }
    final code = input.config.code;
    final os = code.targetOS;
    final onPhone = os == OS.iOS || os == OS.android;
    final environment = <String, String>{
      if (onPhone) ..._mobileProfile,
      if (onPhone) ...await _toolchainFromScript(input, output),
      // Flutter wraps the library in a framework whose Info.plist it stamps
      // with the native-assets iOS version (13.0 today), while the linkers
      // stamp the binary with IPHONEOS_DEPLOYMENT_TARGET (the app's 17.0,
      // from scripts/mobile/env.sh). App Store Connect refuses that pair
      // (ITMS-90208: the framework does not support the minimum OS version
      // in its Info.plist). Build the library for the version Flutter will
      // declare; the app as a whole still requires the project's version.
      if (os == OS.iOS)
        'IPHONEOS_DEPLOYMENT_TARGET': '${code.iOS.targetVersion}.0',
    };
    await FlutterRustBridgeNativeAssetsBuilder(
      assetName: 'core/bridge/frb_generated.io.dart',
      extraCargoBuildArgs: const <String>['--locked'],
      extraCargoEnvironmentVariables: environment,
    ).run(input: input, output: output);
    await _bundlePdfium(input, output);
  });
}

/// The PDFium build for the platform being built, as `scripts/fetch-pdfium.sh`
/// names it (the script pins its version and checksum, and refuses a build
/// with V8 in it). `null` where the app has no PDF reader (Windows).
String? _pdfiumPlatform(CodeConfig code) {
  final arch = code.targetArchitecture;
  switch (code.targetOS) {
    case OS.iOS:
      final simulator = code.iOS.targetSdk == IOSSdk.iPhoneSimulator;
      return switch (arch) {
        Architecture.arm64 =>
          simulator ? 'ios-simulator-arm64' : 'ios-device-arm64',
        Architecture.x64 when simulator => 'ios-simulator-x64',
        _ => null,
      };
    case OS.android:
      return switch (arch) {
        Architecture.arm64 => 'android-arm64',
        Architecture.arm => 'android-arm',
        Architecture.x64 => 'android-x64',
        _ => null,
      };
    case OS.macOS:
      // The computer `flutter test` runs on: the bridge's tests read real
      // PDFs.
      return switch (arch) {
        Architecture.arm64 => 'mac-arm64',
        Architecture.x64 => 'mac-x64',
        _ => null,
      };
    case OS.linux:
      return switch (arch) {
        Architecture.arm64 => 'linux-arm64',
        Architecture.x64 => 'linux-x64',
        _ => null,
      };
    default:
      return null;
  }
}

/// Bundles the PDFium library the PDF reader binds at run time
/// (`kanso_pdf::local`), next to the bridge's own library, as a second native
/// asset. It is fetched by `scripts/fetch-pdfium.sh` (the one place that pins
/// its version, checksum and the no-V8 check) into the repository's
/// `vendor/pdfium`, which is not committed. `KANSO_PDFIUM_DIR` names another
/// folder (CI caches one).
Future<void> _bundlePdfium(BuildInput input, BuildOutputBuilder output) async {
  final code = input.config.code;
  final platform = _pdfiumPlatform(code);
  if (platform == null) {
    return;
  }
  final root = input.packageRoot.resolve('../');
  final script = File.fromUri(root.resolve('scripts/fetch-pdfium.sh'));
  final into =
      Platform.environment['KANSO_PDFIUM_DIR'] ??
      root.resolve('vendor/pdfium').toFilePath();
  final result = await Process.run(
    'bash',
    <String>[script.path, platform],
    workingDirectory: root.toFilePath(),
    environment: <String, String>{'PDFIUM_DIR': into},
  );
  if (result.exitCode != 0) {
    throw ProcessException(
      'bash',
      <String>[script.path, platform],
      '${result.stdout}${result.stderr}',
      result.exitCode,
    );
  }
  final library = File(
    p.join(
      into,
      platform,
      'lib',
      switch (code.targetOS) {
        OS.android || OS.linux => 'libpdfium.so',
        _ => 'libpdfium.dylib',
      },
    ),
  );
  output.dependencies.add(script.uri);
  output.assets.code.add(
    CodeAsset(
      package: input.packageName,
      name: 'pdfium',
      linkMode: DynamicLoadingBundled(),
      file: library.uri,
    ),
  );
}

/// `[profile.mobile]` of the root `Cargo.toml`, as the release profile.
const Map<String, String> _mobileProfile = <String, String>{
  'CARGO_PROFILE_RELEASE_OPT_LEVEL': 'z',
  'CARGO_PROFILE_RELEASE_LTO': 'fat',
  'CARGO_PROFILE_RELEASE_CODEGEN_UNITS': '1',
  'CARGO_PROFILE_RELEASE_PANIC': 'unwind',
  'CARGO_PROFILE_RELEASE_STRIP': 'symbols',
};

/// The variables `scripts/mobile/env.sh` sets that cargo and the C compilers
/// of the Rust dependencies read, for the target being built.
Future<Map<String, String>> _toolchainFromScript(
  BuildInput input,
  BuildOutputBuilder output,
) async {
  final root = input.packageRoot.resolve('../');
  final script = File.fromUri(root.resolve('scripts/mobile/env.sh'));
  output.dependencies.add(script.uri);
  final compiler = input.config.code.cCompiler?.compiler;
  final environment = <String, String>{
    if (Platform.environment['HOME'] case final home?) 'HOME': home,
    if (Platform.environment['PATH'] case final path?) 'PATH': path,
    if (Platform.environment['ANDROID_HOME'] case final sdk?)
      'ANDROID_HOME': sdk,
    // The compiler Flutter chose is inside the NDK it chose; the script
    // finds the same one.
    if (input.config.code.targetOS == OS.android && compiler != null)
      'ANDROID_NDK_HOME': _ndkOf(compiler.toFilePath()),
  };
  final result = await Process.run(
    'bash',
    <String>['-c', r'. "$1" && env -0', 'env', script.path],
    environment: environment,
    includeParentEnvironment: false,
    stdoutEncoding: null,
  );
  if (result.exitCode != 0) {
    throw ProcessException(
      'bash',
      <String>[script.path],
      '${result.stderr}',
      result.exitCode,
    );
  }
  final variables = <String, String>{};
  for (final entry in String.fromCharCodes(result.stdout as List<int>).split(
    '\u0000',
  )) {
    final at = entry.indexOf('=');
    if (at <= 0) continue;
    final name = entry.substring(0, at);
    if (_fromScript.any(name.startsWith)) {
      variables[name] = entry.substring(at + 1);
    }
  }
  // The C++ compiler beside each C one, at the same API level.
  for (final name in variables.keys.toList()) {
    if (name.startsWith('CC_') && variables[name]!.endsWith('-clang')) {
      variables['CXX_${name.substring(3)}'] = '${variables[name]}++';
    }
  }
  return variables;
}

/// The prefixes of the variables taken from the script.
const List<String> _fromScript = <String>[
  'CC_',
  'AR_',
  'CARGO_TARGET_',
  'BINDGEN_EXTRA_CLANG_ARGS_',
  'IPHONEOS_DEPLOYMENT_TARGET',
  'LIBCLANG_PATH',
];

/// The NDK folder that holds [compiler]
/// (`<ndk>/toolchains/llvm/prebuilt/<host>/bin/<compiler>`).
String _ndkOf(String compiler) {
  var directory = p.dirname(compiler);
  for (var up = 0; up < 5; up++) {
    directory = p.dirname(directory);
  }
  return directory;
}
