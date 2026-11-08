#!/bin/sh
# Installs Kanso for the current user from an unpacked release archive:
# the binary in ~/.local/bin, the launcher in ~/.local/share/applications
# and the icon in ~/.local/share/icons. Run it again to update; run it with
# --uninstall to remove those files (your notes stay where they are).
set -eu

here=$(cd "$(dirname "$0")" && pwd)
bin_dir="$HOME/.local/bin"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
apps_dir="$data_home/applications"
icon_dir="$data_home/icons/hicolor/512x512/apps"
# PDFs are drawn by a helper and its library, and voice input has ONNX
# Runtime; Kanso finds them in ../lib/kanso.
lib_dir="$HOME/.local/lib/kanso"

if [ "${1:-}" = "--uninstall" ]; then
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
    mkdir -p "$bin_dir"
    install -m 755 "$here/kanso-headless" "$bin_dir/kanso"
    # ONNX Runtime, which the binary finds in ../lib/kanso.
    if [ -f "$here/libonnxruntime.so.1" ]; then
        mkdir -p "$lib_dir"
        install -m 755 "$here/libonnxruntime.so.1" "$lib_dir/libonnxruntime.so.1"
    fi
    echo "Installed the headless Kanso CLI in $bin_dir/kanso. It provides serve, account, service, api-key and mcp."
    exit 0
fi

mkdir -p "$bin_dir" "$apps_dir" "$icon_dir"
install -m 755 "$here/kanso" "$bin_dir/kanso"
# Plugins run in their own process, beside Kanso.
[ -f "$here/kanso-plugin-host" ] && install -m 755 "$here/kanso-plugin-host" "$bin_dir/kanso-plugin-host"
# Voice input's ONNX Runtime, which the binary finds in ../lib/kanso.
if [ -f "$here/libonnxruntime.so.1" ]; then
    mkdir -p "$lib_dir"
    install -m 755 "$here/libonnxruntime.so.1" "$lib_dir/libonnxruntime.so.1"
    if [ -d "$here/onnxruntime-licenses" ]; then
        mkdir -p "$lib_dir/onnxruntime-licenses"
        cp -R "$here/onnxruntime-licenses/." "$lib_dir/onnxruntime-licenses/"
    fi
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
install -m 644 "$here/kanso.png" "$icon_dir/kanso.png"
# The launcher runs the installed binary by its full path, since a desktop
# session's PATH often leaves out ~/.local/bin.
sed "s|^Exec=kanso|Exec=\"$bin_dir/kanso\"|" "$here/kanso.desktop" >"$apps_dir/kanso.desktop"
chmod 644 "$apps_dir/kanso.desktop"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$apps_dir" || true
# kanso:// opens Kanso: Sign in with Apple returns through it.
command -v xdg-mime >/dev/null 2>&1 && xdg-mime default kanso.desktop x-scheme-handler/kanso || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q "$data_home/icons/hicolor" || true

echo "Installed Kanso in $bin_dir/kanso; it's in your applications menu."
