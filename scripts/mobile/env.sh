# Sourced by the mobile scripts and CI: the compilers and settings that
# building the core (#48) for iOS and Android needs.
#
# What it sets, and why:
# - The Android NDK's clang for each Android target. The C code in SQLite,
#   ring, QuickJS and tree-sitter is built with it, and it links the tests.
#   API 29 is the lowest Android version the core builds for.
# - bindgen's header paths. rquickjs ships no bindings for phones, so
#   kanso-render generates them with libclang (Xcode's on a Mac, or the
#   NDK's), which needs to be told where each target's system headers are.
# - The iOS deployment target, so Xcode's linker and cc agree with the
#   simulator.
#
# The Android NDK is ANDROID_NDK_HOME, else ANDROID_NDK_LATEST_HOME, else the
# newest one in the SDK. Without one, only the iOS settings are made.

kanso_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export KANSO_ROOT="$kanso_root"

# A bindgen setting for every target (a shell profile may have one for the
# Mac's own headers) would be passed to the phones' targets too.
unset BINDGEN_EXTRA_CLANG_ARGS

# The lowest iOS the app supports.
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-17.0}"

if [ "$(uname -s)" = Darwin ] && command -v xcrun >/dev/null 2>&1; then
	ios_sdk="$(xcrun --sdk iphoneos --show-sdk-path 2>/dev/null || true)"
	sim_sdk="$(xcrun --sdk iphonesimulator --show-sdk-path 2>/dev/null || true)"
	[ -n "$ios_sdk" ] && export BINDGEN_EXTRA_CLANG_ARGS_aarch64_apple_ios="-isysroot $ios_sdk"
	[ -n "$sim_sdk" ] && export BINDGEN_EXTRA_CLANG_ARGS_aarch64_apple_ios_sim="-isysroot $sim_sdk"
fi

ndk="${ANDROID_NDK_HOME:-${ANDROID_NDK_LATEST_HOME:-}}"
if [ -z "$ndk" ]; then
	sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}"
	ndk="$(ls -d "$sdk"/ndk/*/ 2>/dev/null | sort -V | tail -n 1 || true)"
	ndk="${ndk%/}"
fi
if [ -n "$ndk" ] && [ -d "$ndk/toolchains/llvm/prebuilt" ]; then
	export ANDROID_NDK_HOME="$ndk"
	ndk_llvm="$(echo "$ndk"/toolchains/llvm/prebuilt/*)"
	ndk_bin="$ndk_llvm/bin"
	for pair in aarch64_linux_android:aarch64-linux-android armv7_linux_androideabi:armv7a-linux-androideabi x86_64_linux_android:x86_64-linux-android; do
		under="${pair%%:*}"
		clang_triple="${pair##*:}"
		upper="$(echo "$under" | tr '[:lower:]' '[:upper:]')"
		export "CC_$under=$ndk_bin/${clang_triple}29-clang"
		export "AR_$under=$ndk_bin/llvm-ar"
		export "CARGO_TARGET_${upper}_LINKER=$ndk_bin/${clang_triple}29-clang"
		export "BINDGEN_EXTRA_CLANG_ARGS_$under=--sysroot=$ndk_llvm/sysroot"
		# C libraries built with CMake (opusic-sys) use the NDK's own toolchain
		# file; without it a Mac's CMake treats the build as a macOS one and
		# passes -arch/-isysroot to the Android compiler. The NDK's file needs
		# the ABI (it defaults to armeabi-v7a), so each target gets a small
		# file that sets it and the API level, then includes the NDK's.
		case "$under" in
		aarch64_linux_android) abi=arm64-v8a ;;
		armv7_linux_androideabi) abi=armeabi-v7a ;;
		x86_64_linux_android) abi=x86_64 ;;
		esac
		cmake_dir="${TMPDIR:-/tmp}/kanso-android-cmake"
		mkdir -p "$cmake_dir"
		printf 'set(ANDROID_ABI %s)\nset(ANDROID_PLATFORM android-29)\ninclude("%s/build/cmake/android.toolchain.cmake")\n' \
			"$abi" "$ndk" >"$cmake_dir/$abi.cmake"
		export "CMAKE_TOOLCHAIN_FILE_$under=$cmake_dir/$abi.cmake"
	done
	# Where the NDK ships a libclang of its own (it is x86-64 only on a
	# Linux runner, which has the toolchain's anyway), bindgen may use it.
	if [ -z "${LIBCLANG_PATH:-}" ] && [ "$(uname -s)" = Linux ] && ls "$ndk_llvm"/lib/libclang.so* >/dev/null 2>&1; then
		export LIBCLANG_PATH="$ndk_llvm/lib"
	fi
fi
