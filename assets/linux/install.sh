#!/bin/sh
# Installs Kanso for the current user from an unpacked release archive:
# the binary in ~/.local/bin, the launcher in ~/.local/share/applications
# and the icon in ~/.local/share/icons. Run it again to update; run it with
# --uninstall to remove those files (your notes stay where they are).
#
# --nightly installs Kanso Nightly beside Kanso, from its own archive, in the
# same layout: the binary is ~/.local/bin/kanso-nightly and its libraries
# and PDF helper are in ~/.local/lib/kanso-nightly, so neither app replaces
# the other's binary or libraries, and Kanso Nightly can update itself. The
# plugin host is in that folder too, so an update of one app never swaps
# the other's host; ~/.local/bin/kanso-plugin-host belongs to Kanso and is
# never touched. Use --nightly --uninstall to remove it.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
bin_dir="$HOME/.local/bin"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
apps_dir="$data_home/applications"
icon_dir="$data_home/icons/hicolor/512x512/apps"

nightly=0
for arg in "$@"; do
    case "$arg" in
    --nightly) nightly=1 ;;
    esac
done

if [ "$nightly" = 1 ]; then
    # Kanso's layout under its own names; the updater finds the support
    # folder as ../lib/<binary name> and the binary's rpath names it too.
    bin_name=kanso-nightly
    lib_dir="$HOME/.local/lib/kanso-nightly"
    host_dir="$lib_dir"
    desktop_name=kanso-nightly.desktop
    icon_name=kanso-nightly.png
    scheme=kanso-nightly
    display_name="Kanso Nightly"
else
    bin_name=kanso
    # PDFs are drawn by a helper and its library, and voice input has ONNX
    # Runtime; Kanso finds them in ../lib/kanso.
    lib_dir="$HOME/.local/lib/kanso"
    host_dir="$bin_dir"
    desktop_name=kanso.desktop
    icon_name=kanso.png
    scheme=kanso
    display_name=Kanso
fi

# package_for_library prints the Ubuntu/Debian package that provides a
# shared library Kanso links against. The tarball carries ONNX Runtime,
# PDFium and the XKB/XCB bridge; the rest of the desktop libraries come from
# the distribution. Keep this list in step with the binary's NEEDED entries.
# Package names changed in the 64-bit time_t transition (Ubuntu 24.04,
# for example): prefer the t64 name when apt knows it and the original
# name is gone.
package_available() {
    command -v apt-cache >/dev/null 2>&1 || return 1
    LC_ALL=C apt-cache policy "$1" 2>/dev/null | grep -q 'Candidate: [^(]'
}

preferred_package() {
    package=$1
    if package_available "$package"; then
        printf '%s\n' "$package"
        return 0
    fi
    case "$package" in
        *t64) ;;
        *)
            if package_available "${package}t64"; then
                printf '%s\n' "${package}t64"
                return 0
            fi
            ;;
    esac
    printf '%s\n' "$package"
}

package_for_library() {
    case "$1" in
        libasound.so.2) echo libasound2 ;;
        libdbus-1.so.3) echo libdbus-1-3 ;;
        libfido2.so.1) echo libfido2-1 ;;
        libgcc_s.so.1) echo libgcc-s1 ;;
        libstdc++.so.6) echo libstdc++6 ;;
        libxcb.so.1) echo libxcb1 ;;
        libxcb-xkb.so.1) echo libxcb-xkb1 ;;
        libxkbcommon.so.0) echo libxkbcommon0 ;;
        libxkbcommon-x11.so.0) echo libxkbcommon-x11-0 ;;
        libvulkan.so.1) echo libvulkan1 ;;
        *) return 1 ;;
    esac
}

# check_runtime_dependencies reports every shared library that ldd cannot
# resolve before installing anything. This is what turns a bare
# "error while loading shared libraries" into the package that fixes it.
# Pass "gui" to include Vulkan, which wgpu loads dynamically rather than as a
# NEEDED entry.
check_runtime_dependencies() {
    binary=$1
    kind=$2
    missing=
    if ! command -v ldd >/dev/null 2>&1; then
        echo "install.sh: ldd is required to check Linux libraries before installing (on Ubuntu/Debian it is in libc-bin)." >&2
        return 1
    fi
    ldd_status=0
    ldd_output=$(ldd "$binary" 2>&1) || ldd_status=$?
    # ldd exits 1 when a library is missing, which is the case this check
    # exists for. Any other failure means ldd could not inspect the binary,
    # so don't treat an empty result as a clean bill of health.
    if [ "$ldd_status" -ne 0 ] && ! printf '%s\n' "$ldd_output" | grep -q 'not found'; then
        echo "install.sh: could not inspect $binary with ldd:" >&2
        printf '%s\n' "$ldd_output" >&2
        return 1
    fi
    missing=$(printf '%s\n' "$ldd_output" | awk '/=> not found/ { print $1 }' | sort -u)
    if [ "$kind" = "gui" ] && command -v ldconfig >/dev/null 2>&1; then
        if ! ldconfig -p 2>/dev/null | grep -q 'libvulkan\.so\.1'; then
            missing=$(printf '%s\n%s\n' "$missing" libvulkan.so.1 | sed '/^$/d' | sort -u)
        fi
    fi
    [ -n "$missing" ] || return 0

    echo "Kanso is missing Linux libraries it needs to start:" >&2
    # shellcheck disable=SC2086 # sonames are simple words; split the list.
    for library in $missing; do
        printf '  %s\n' "$library" >&2
    done
    packages=
    # shellcheck disable=SC2086 # sonames are simple words; split the list.
    for library in $missing; do
        package=$(package_for_library "$library") || continue
        package=$(preferred_package "$package")
        case " $packages " in
            *" $package "*) ;;
            *) packages="$packages $package" ;;
        esac
    done
    if [ -n "$packages" ]; then
        echo >&2
        echo "On Ubuntu/Debian, install them with:" >&2
        echo "  sudo apt install$packages" >&2
    fi
    echo >&2
    echo "The .deb package installs Kanso's system dependencies automatically." >&2
    echo "Install the packages above, then run this script again." >&2
    return 1
}

if [ "$nightly" = 1 ]; then
    for arg in "$@"; do
        if [ "$arg" = "--uninstall" ]; then
            rm -f "$bin_dir/kanso-nightly" "$apps_dir/$desktop_name" "$icon_dir/$icon_name"
            rm -rf "$lib_dir"
            echo "Removed Kanso Nightly. Your notes are still in ${XDG_DATA_HOME:-~/.local/share}/Kanso Nightly."
            exit 0
        fi
    done
elif [ "${1:-}" = "--uninstall" ]; then
    rm -f "$bin_dir/kanso" "$bin_dir/kanso-plugin-host" "$apps_dir/kanso.desktop" "$icon_dir/kanso.png"
    rm -rf "$lib_dir"
    echo "Removed Kanso. Your notes are still in ${XDG_DATA_HOME:-~/.local/share}/kanso."
    exit 0
fi

if [ "${1:-}" = "--headless" ]; then
    if [ ! -f "$here/kanso-headless" ]; then
        echo "This archive has no kanso-headless binary. Download the Linux headless release archive." >&2
        exit 1
    fi
    check_runtime_dependencies "$here/kanso-headless" headless || exit 1
    mkdir -p "$bin_dir"
    install -m 755 "$here/kanso-headless" "$bin_dir/kanso"
    echo "Installed the headless Kanso CLI in $bin_dir/kanso. It provides serve, account, service, api-key and mcp."
    exit 0
fi

if [ ! -f "$here/kanso" ]; then
    echo "This archive has no kanso binary. Download the Linux desktop release archive." >&2
    exit 1
fi
check_runtime_dependencies "$here/kanso" gui || exit 1

mkdir -p "$bin_dir" "$apps_dir" "$icon_dir"
if [ "$nightly" = 1 ]; then
    # An earlier Kanso Nightly install left a link here to a binary in its
    # lib folder; install through it would write into the wrong place.
    rm -f "$bin_dir/$bin_name" "$lib_dir/kanso"
fi
install -m 755 "$here/kanso" "$bin_dir/$bin_name"
exec_path="$bin_dir/$bin_name"
# Plugins run in their own process, beside Kanso.
if [ -f "$here/kanso-plugin-host" ]; then
    mkdir -p "$host_dir"
    install -m 755 "$here/kanso-plugin-host" "$host_dir/kanso-plugin-host"
fi
# Voice input's ONNX Runtime, which the binary finds beside it or in
# ../lib/kanso.
if [ -f "$here/libonnxruntime.so.1" ]; then
    mkdir -p "$lib_dir"
    install -m 755 "$here/libonnxruntime.so.1" "$lib_dir/libonnxruntime.so.1"
    if [ -d "$here/onnxruntime-licenses" ]; then
        mkdir -p "$lib_dir/onnxruntime-licenses"
        cp -R "$here/onnxruntime-licenses/." "$lib_dir/onnxruntime-licenses/"
    fi
fi
# The X11 keyboard bridge is a hard NEEDED entry on the GUI, even on
# Wayland. It and libxcb-xkb ride with the app; the bridge's RUNPATH is
# $ORIGIN, so it finds libxcb-xkb beside it after installation.
for library in libxkbcommon-x11.so.0 libxcb-xkb.so.1; do
    if [ -f "$here/$library" ]; then
        mkdir -p "$lib_dir"
        install -m 755 "$here/$library" "$lib_dir/$library"
    fi
done
if [ -d "$here/xkb-licenses" ]; then
    mkdir -p "$lib_dir/xkb-licenses"
    cp -R "$here/xkb-licenses/." "$lib_dir/xkb-licenses/"
fi
if [ -f "$here/kanso-pdf-worker" ] && [ -f "$here/libpdfium.so" ]; then
    mkdir -p "$lib_dir"
    install -m 755 "$here/kanso-pdf-worker" "$lib_dir/kanso-pdf-worker"
    install -m 755 "$here/libpdfium.so" "$lib_dir/libpdfium.so"
    if [ -d "$here/pdfium-licenses" ]; then
        mkdir -p "$lib_dir/pdfium-licenses"
        cp -R "$here/pdfium-licenses/." "$lib_dir/pdfium-licenses/"
    fi
fi
install -m 644 "$here/$icon_name" "$icon_dir/$icon_name"
# The launcher runs the installed binary by its full path, since a desktop
# session's PATH often leaves out ~/.local/bin.
sed "s|^Exec=[^ ]*|Exec=\"$exec_path\"|" "$here/$desktop_name" >"$apps_dir/$desktop_name"
chmod 644 "$apps_dir/$desktop_name"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$apps_dir" || true
# The app's scheme opens it: Sign in with Apple returns through it.
command -v xdg-mime >/dev/null 2>&1 && xdg-mime default "$desktop_name" "x-scheme-handler/$scheme" || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q "$data_home/icons/hicolor" || true

echo "Installed $display_name in $exec_path; it's in your applications menu."
