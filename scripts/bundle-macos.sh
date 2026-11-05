#!/bin/sh
# Builds Kanso.app, or Kanso Dev.app with --dev (docs/DEVELOPING.md).
#
#   scripts/bundle-macos.sh                 target/release/Kanso.app
#   scripts/bundle-macos.sh --dev           target/kanso-dev/release/Kanso Dev.app
#   scripts/bundle-macos.sh --dev --install ...and copies it to ~/Applications
#
# Kanso Dev is its own app: bundle ID app.kanso.Kanso.dev, an icon with an
# amber DEV band, its own data folder and Keychain service, and no updates.
# It is never installed over /Applications/Kanso.app, which only ever comes
# from a release and updates itself.
#
# KANSO_SIGN_IDENTITY signs with that identity, the hardened runtime and a
# secure timestamp (the release workflow's Developer ID); without it the app
# is signed ad hoc, enough to run on this Mac.
#
# KANSO_PROVISION_PROFILE is a Developer ID provisioning profile for
# app.kanso.Kanso with Associated Domains. With it and KANSO_SIGN_IDENTITY,
# it is embedded as Contents/embedded.provisionprofile and native passkeys
# work. Without either, the app is built without the Associated Domains
# entitlement, which macOS refuses to launch an app with when no profile
# authorizes it; everything else works, and passkeys say they aren't available.
set -eu

cd "$(dirname "$0")/.."

dev=0
install=0
for arg in "$@"; do
    case "$arg" in
    --dev) dev=1 ;;
    --install) install=1 ;;
    -h | --help)
        sed -n '2,23p' "$0" | sed 's/^# \{0,1\}//'
        exit 0
        ;;
    *)
        echo "bundle-macos: unknown option $arg" >&2
        exit 1
        ;;
    esac
done

if [ "$dev" = 1 ]; then
    # Its own target folder, so switching between the two never rebuilds
    # everything and a release build never picks up the dev flavour.
    target_dir=target/kanso-dev
    cargo build --release -p kanso-app -p kanso-plugin-host --features kanso-app/dev-flavour --target-dir "$target_dir"
    name="Kanso Dev"
    bundle_id=app.kanso.Kanso.dev
else
    target_dir=target
    if [ "$install" = 1 ]; then
        echo "bundle-macos: Kanso itself installs from a release and updates itself; --install is for --dev builds." >&2
        exit 1
    fi
    cargo build --release -p kanso-app -p kanso-plugin-host
    name=Kanso
    bundle_id=app.kanso.Kanso
fi

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
app="$target_dir/release/$name.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$target_dir/release/kanso" "$app/Contents/MacOS/kanso"
# Plugins run in their own process, beside Kanso (docs/PLUGINS.md).
cp "$target_dir/release/kanso-plugin-host" "$app/Contents/MacOS/kanso-plugin-host"

# The icon; regenerate Kanso's with assets/icon/make-icon.swift and iconutil.
if [ "$dev" = 1 ]; then
    iconset=$(mktemp -d)/Kanso.iconset
    mkdir -p "$iconset"
    KANSO_ICON_DEV=1 swift assets/icon/make-icon.swift "$iconset/icon_512x512@2x.png" 1024
    for size in 16 32 128 256 512; do
        sips -z "$size" "$size" "$iconset/icon_512x512@2x.png" --out "$iconset/icon_${size}x${size}.png" >/dev/null
        double=$((size * 2))
        sips -z "$double" "$double" "$iconset/icon_512x512@2x.png" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
    done
    iconutil -c icns "$iconset" -o "$app/Contents/Resources/Kanso.icns"
    rm -rf "$(dirname "$iconset")"
else
    cp assets/icon/Kanso.icns "$app/Contents/Resources/Kanso.icns"
fi

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>$bundle_id</string>
    <key>CFBundleName</key>
    <string>$name</string>
    <key>CFBundleDisplayName</key>
    <string>$name</string>
    <key>CFBundleExecutable</key>
    <string>kanso</string>
    <key>CFBundleIconFile</key>
    <string>Kanso</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$version</string>
    <key>CFBundleVersion</key>
    <string>$version</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Markdown Document</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>net.daringfireball.markdown</string>
                <string>public.markdown</string>
            </array>
        </dict>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Plain Text Document</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.plain-text</string>
            </array>
        </dict>
    </array>
    <key>UTImportedTypeDeclarations</key>
    <array>
        <dict>
            <key>UTTypeIdentifier</key>
            <string>net.daringfireball.markdown</string>
            <key>UTTypeDescription</key>
            <string>Markdown Document</string>
            <key>UTTypeConformsTo</key>
            <array>
                <string>public.plain-text</string>
            </array>
            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <array>
                    <string>md</string>
                    <string>markdown</string>
                    <string>mdown</string>
                    <string>mkd</string>
                    <string>mkdn</string>
                </array>
                <key>public.mime-type</key>
                <array>
                    <string>text/markdown</string>
                </array>
            </dict>
        </dict>
    </array>
    <key>NSMicrophoneUsageDescription</key>
    <string>Kanso listens only while you dictate or record, and turns what you say into text on this Mac.</string>
    <key>NSScreenCaptureUsageDescription</key>
    <string>Kanso uses Screen Recording only to hear the audio of the app you pick when you record a meeting. It never records or saves your screen.</string>
</dict>
</plist>
PLIST

plutil -lint "$app/Contents/Info.plist" >/dev/null
# The microphone, for voice input, and the passkey domain
# (packaging/macos/Kanso.entitlements). Signed from a copy, which the
# provisioning profile below completes or strips.
plutil -lint packaging/macos/Kanso.entitlements >/dev/null
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
entitlements="$work/Kanso.entitlements"
cp packaging/macos/Kanso.entitlements "$entitlements"

profile=${KANSO_PROVISION_PROFILE:-}
skip_profile() {
    echo "bundle-macos: $1; building without Associated Domains, so native passkeys are unavailable in this build." >&2
    plutil -remove 'com\.apple\.developer\.associated-domains' "$entitlements"
}
# profile_unexpired and profile_has_certificate.
. "$(dirname "$0")/macos-profile.sh"

if [ -z "$profile" ]; then
    skip_profile "no KANSO_PROVISION_PROFILE"
elif [ ! -f "$profile" ]; then
    skip_profile "KANSO_PROVISION_PROFILE ($profile) is not a file"
elif [ -z "${KANSO_SIGN_IDENTITY:-}" ]; then
    skip_profile "a provisioning profile needs KANSO_SIGN_IDENTITY, and this build is signed ad hoc"
elif ! security cms -D -i "$profile" >"$work/profile.plist" 2>/dev/null; then
    skip_profile "KANSO_PROVISION_PROFILE isn't a readable provisioning profile"
else
    team=$(plutil -extract TeamIdentifier.0 raw "$work/profile.plist" 2>/dev/null || true)
    profile_app=$(plutil -extract 'Entitlements.com\.apple\.application-identifier' raw "$work/profile.plist" 2>/dev/null || true)
    if ! profile_unexpired "$work/profile.plist"; then
        skip_profile "the provisioning profile has expired (or has no ExpirationDate)"
    elif ! profile_has_certificate "$work/profile.plist" "$KANSO_SIGN_IDENTITY"; then
        skip_profile "the provisioning profile doesn't list the certificate of $KANSO_SIGN_IDENTITY"
    elif [ -z "$team" ] || [ "$profile_app" != "$team.$bundle_id" ]; then
        skip_profile "the provisioning profile is for ${profile_app:-another app}, not $bundle_id"
    elif ! plutil -extract 'Entitlements.com\.apple\.developer\.associated-domains' raw "$work/profile.plist" >/dev/null 2>&1; then
        skip_profile "the provisioning profile doesn't allow Associated Domains"
    else
        cp "$profile" "$app/Contents/embedded.provisionprofile"
        # A restricted entitlement is honored only with the app and team it
        # was provisioned for.
        plutil -insert 'com\.apple\.application-identifier' -string "$team.$bundle_id" "$entitlements"
        plutil -insert 'com\.apple\.developer\.team-identifier' -string "$team" "$entitlements"
        echo "Embedded the provisioning profile for $team.$bundle_id."
    fi
fi
plutil -lint "$entitlements" >/dev/null

# The plugin host is signed first, with no entitlements: it needs none,
# and the hardened runtime keeps it from loading code it wasn't built with.
host="$app/Contents/MacOS/kanso-plugin-host"
if [ -n "${KANSO_SIGN_IDENTITY:-}" ]; then
    codesign --force --options runtime --timestamp --sign "$KANSO_SIGN_IDENTITY" "$host"
    codesign --force --options runtime --timestamp --entitlements "$entitlements" \
        --sign "$KANSO_SIGN_IDENTITY" "$app"
else
    # Ad hoc: macOS runs it on this Mac; not for distribution.
    codesign --force --sign - "$host"
    codesign --force --entitlements "$entitlements" --sign - "$app"
fi
codesign --verify --deep --strict "$app"
echo "Built $app"

if [ "$install" = 1 ]; then
    dest="$HOME/Applications/$name.app"
    mkdir -p "$HOME/Applications"
    rm -rf "$dest"
    cp -R "$app" "$dest"
    echo "Installed $dest"
fi
