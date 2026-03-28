#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="Lily Nest"
APP_DIR="$ROOT_DIR/dist/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
BIN_NAME="lily-nest"
BIN_SOURCE="$ROOT_DIR/target/release/$BIN_NAME"
BIN_TARGET="$RESOURCES_DIR/bin/$BIN_NAME"
LAUNCHER_TARGET="$MACOS_DIR/LilyNest"
PLIST_TARGET="$CONTENTS_DIR/Info.plist"
ICONSET_DIR="$ROOT_DIR/dist/AppIcon.iconset"
ICON_PNG_SOURCE="$ROOT_DIR/AppIcon.png"
ICON_ICNS_SOURCE="$ROOT_DIR/AppIcon.icns"
ICON_ICNS_TARGET="$RESOURCES_DIR/AppIcon.icns"

mkdir -p "$ROOT_DIR/dist"

echo ">> Building release binary"
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

if [[ ! -f "$BIN_SOURCE" ]]; then
  echo "Release binary not found: $BIN_SOURCE" >&2
  exit 1
fi

echo ">> Preparing app bundle"
rm -rf "$APP_DIR"
rm -rf "$ICONSET_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR/bin"

cp "$BIN_SOURCE" "$BIN_TARGET"
chmod +x "$BIN_TARGET"

cat > "$LAUNCHER_TARGET" <<'EOF'
#!/bin/zsh
set -euo pipefail

APP_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$APP_ROOT/Resources/bin/lily-nest"
LOG_DIR="$HOME/Library/Logs/LilyNest"
mkdir -p "$LOG_DIR"

exec "$BIN" >> "$LOG_DIR/stdout.log" 2>> "$LOG_DIR/stderr.log"
EOF
chmod +x "$LAUNCHER_TARGET"

if [[ -f "$ICON_ICNS_SOURCE" ]]; then
  echo ">> Copying app icon from AppIcon.icns"
  cp "$ICON_ICNS_SOURCE" "$ICON_ICNS_TARGET"
elif [[ -f "$ICON_PNG_SOURCE" ]]; then
  echo ">> Building app icon from AppIcon.png"
  mkdir -p "$ICONSET_DIR"

  sips -z 16 16     "$ICON_PNG_SOURCE" --out "$ICONSET_DIR/icon_16x16.png" >/dev/null
  sips -z 32 32     "$ICON_PNG_SOURCE" --out "$ICONSET_DIR/icon_16x16@2x.png" >/dev/null
  sips -z 32 32     "$ICON_PNG_SOURCE" --out "$ICONSET_DIR/icon_32x32.png" >/dev/null
  sips -z 64 64     "$ICON_PNG_SOURCE" --out "$ICONSET_DIR/icon_32x32@2x.png" >/dev/null
  sips -z 128 128   "$ICON_PNG_SOURCE" --out "$ICONSET_DIR/icon_128x128.png" >/dev/null
  sips -z 256 256   "$ICON_PNG_SOURCE" --out "$ICONSET_DIR/icon_128x128@2x.png" >/dev/null
  sips -z 256 256   "$ICON_PNG_SOURCE" --out "$ICONSET_DIR/icon_256x256.png" >/dev/null
  sips -z 512 512   "$ICON_PNG_SOURCE" --out "$ICONSET_DIR/icon_256x256@2x.png" >/dev/null
  sips -z 512 512   "$ICON_PNG_SOURCE" --out "$ICONSET_DIR/icon_512x512.png" >/dev/null
  cp "$ICON_PNG_SOURCE" "$ICONSET_DIR/icon_512x512@2x.png"

  iconutil -c icns "$ICONSET_DIR" -o "$ICON_ICNS_TARGET"
else
  echo ">> No AppIcon.png or AppIcon.icns found, continuing without a custom icon"
fi

cat > "$PLIST_TARGET" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>zh_CN</string>
    <key>CFBundleDisplayName</key>
    <string>Lily Nest</string>
    <key>CFBundleExecutable</key>
    <string>LilyNest</string>
    <key>CFBundleIdentifier</key>
    <string>cn.sulyhub.lilynest</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleName</key>
    <string>Lily Nest</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo ">> App bundle ready:"
echo "$APP_DIR"
echo
echo "Double-click the app in Finder to launch it."
