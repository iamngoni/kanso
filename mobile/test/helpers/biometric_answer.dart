// Kanso — biometric_answer.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ffi';
import 'dart:typed_data';

/// What the native lock plugin does after a biometric check: hands the
/// answer for [ticket] to the library's C entry `kanso_biometric_answer`.
/// [status] is 0 ok (with the 32-byte [secret]), 1 cancelled, 2 invalidated,
/// 3 unavailable, 4 failed. Returns 1 applied, 0 no such ticket, -1 the
/// secret didn't open the kept keys, -5 a fault.
typedef BiometricAnswer =
    int Function(int ticket, int status, [Uint8List? secret]);

/// The C entry of [library], as a Dart function. The secret is copied into
/// memory the C allocator owns, and zeroed and freed after the call, as the
/// plugin does.
BiometricAnswer biometricAnswerOf(DynamicLibrary library) {
  final answer = library
      .lookupFunction<
        Int32 Function(Uint64, Int32, Pointer<Uint8>, IntPtr),
        int Function(int, int, Pointer<Uint8>, int)
      >('kanso_biometric_answer');
  final process = DynamicLibrary.process();
  final malloc = process
      .lookupFunction<
        Pointer<Void> Function(IntPtr),
        Pointer<Void> Function(int)
      >('malloc');
  final free = process
      .lookupFunction<
        Void Function(Pointer<Void>),
        void Function(Pointer<Void>)
      >(
        'free',
      );
  return (ticket, status, [secret]) {
    if (secret == null) {
      return answer(ticket, status, nullptr, 0);
    }
    final memory = malloc(secret.length).cast<Uint8>();
    try {
      memory.asTypedList(secret.length).setAll(0, secret);
      return answer(ticket, status, memory, secret.length);
    } finally {
      memory.asTypedList(secret.length).fillRange(0, secret.length, 0);
      free(memory.cast());
    }
  };
}
