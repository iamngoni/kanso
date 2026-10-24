#!/usr/bin/env bash
# Makes sure a Linux runner has the packages Kanso builds against.
# Usage: scripts/linux-deps.sh
#
# The self-hosted runners are containers where sudo may be blocked ("no new
# privileges"), so: nothing to do when every package is already there,
# apt-get directly when running as root, sudo otherwise, and a clear error
# naming the missing packages when none of that works. To skip the install
# on every run, bake these packages into the runner image.
set -euo pipefail

packages=(
	pkg-config clang cmake
	libxkbcommon-dev libxkbcommon-x11-dev
	libwayland-dev libvulkan-dev libegl-dev
	libx11-dev libx11-xcb-dev libxcb1-dev
	libxcb-randr0-dev libxcb-shape0-dev libxcb-shm0-dev libxcb-xfixes0-dev
	libfontconfig-dev libfreetype-dev
	libssl-dev libasound2-dev libdbus-1-dev libzstd-dev
	# kanso-fido links libfido2 for Linux security keys.
	libfido2-dev
)

missing=()
for package in "${packages[@]}"; do
	dpkg-query -W -f='${Status}' "$package" 2>/dev/null | grep -q 'install ok installed' ||
		missing+=("$package")
done

if [ ${#missing[@]} -eq 0 ]; then
	echo "Every build package is already installed."
	exit 0
fi
echo "Installing: ${missing[*]}"

# The self-hosted Linux runners don't allow sudo, but apt can still download
# packages as the runner user. libfido2 is a build-time link dependency, so
# unpack the dev package, its runtime library and libcbor into a cache rather
# than requiring root. Keep this narrow: packages with install scripts or
# system-wide configuration still belong in the runner image.
install_libfido2_without_root() {
	local cache="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/kanso-libfido2"
	local debs="$cache/debs"
	local prefix="$cache/root"
	local apt="$cache/apt"
	local deb dependency fido_lib libdir pkgconfig
	local apt_options=(
		-o "Dir::State::lists=$apt/lists"
		-o "Dir::Cache::archives=$apt/archives"
		-o "APT::Get::List-Cleanup=0"
		-o "Debug::NoLocking=1"
	)

	rm -rf "$cache"
	mkdir -p "$debs" "$prefix" "$apt/lists/partial" "$apt/archives/partial"
	(
		cd "$debs"
		# The runner user cannot refresh /var/lib/apt/lists. Give apt its own
		# indexes and archive directory so downloads still work without root.
		apt-get "${apt_options[@]}" update
		apt-get "${apt_options[@]}" download libfido2-dev libfido2-1
		# libfido2-1 depends on libcbor, whose package name differs between
		# Debian releases (libcbor0.8 on bookworm, libcbor0.10 on noble).
		for deb in libfido2-1_*.deb; do
			dependency=$(dpkg-deb -f "$deb" Depends |
				tr ',' '\n' |
				sed -n 's/^[[:space:]]*\(libcbor[^[:space:](]*\).*/\1/p')
			if [ -n "$dependency" ]; then
				apt-get "${apt_options[@]}" download "$dependency"
			fi
		done
		for deb in ./*.deb; do
			dpkg-deb -x "$deb" "$prefix"
		done
	)

	fido_lib=$(find "$prefix/usr/lib" -name 'libfido2.so.1.*' -type f | head -1)
	libdir=$(dirname "$fido_lib")
	pkgconfig=$(find "$prefix/usr/lib" -type d -name pkgconfig | head -1)
	if [ -z "$fido_lib" ] || [ ! -e "$prefix/usr/include/fido.h" ]; then
		echo "::error::Could not unpack libfido2 into $prefix."
		exit 1
	fi
	if [ -z "${GITHUB_ENV:-}" ]; then
		echo "::error::The libfido2 fallback needs GITHUB_ENV; run this script under GitHub Actions or install libfido2-dev."
		exit 1
	fi
	{
		echo "LIBRARY_PATH=$libdir${LIBRARY_PATH:+:$LIBRARY_PATH}"
		echo "LD_LIBRARY_PATH=$libdir${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
		echo "CPATH=$prefix/usr/include${CPATH:+:$CPATH}"
		if [ -n "$pkgconfig" ]; then
			echo "PKG_CONFIG_PATH=$pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
		fi
	} >>"$GITHUB_ENV"
	echo "Using libfido2 from $prefix (no root needed)."
}

if [ "$(id -u)" -eq 0 ]; then
	export DEBIAN_FRONTEND=noninteractive
	apt-get update
	apt-get install -y --no-install-recommends "${missing[@]}"
elif sudo -n true 2>/dev/null; then
	export DEBIAN_FRONTEND=noninteractive
	sudo -n apt-get update
	sudo -n apt-get install -y --no-install-recommends "${missing[@]}"
elif [ "${#missing[@]}" -eq 1 ] && [ "${missing[0]}" = "libfido2-dev" ]; then
	install_libfido2_without_root
else
	echo "::error::This runner lacks ${missing[*]} and can't install them (not root, and sudo is blocked). Add them to the runner image."
	exit 1
fi
