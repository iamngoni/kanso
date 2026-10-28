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
    };
    await FlutterRustBridgeNativeAssetsBuilder(
      assetName: 'core/bridge/frb_generated.io.dart',
      extraCargoBuildArgs: const <String>['--locked'],
      extraCargoEnvironmentVariables: environment,
    ).run(input: input, output: output);
  });
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
