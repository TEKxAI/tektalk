#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
source_file="$root/plugins/desktop/macos/NativeTabPlugin.swift"
output="$root/build/desktop-plugins/macos"
mkdir -p "$output"
for item in 'message:TEKTALK_MESSAGE_PLUGIN' 'ai:TEKTALK_AI_PLUGIN' 'me:TEKTALK_ME_PLUGIN'; do
  name=${item%%:*}; flag=${item#*:}
  xcrun swiftc -parse-as-library -emit-library -O -D "$flag" -framework AppKit "$source_file" -o "$output/tektalk_${name}_plugin.dylib"
  codesign --force --sign - "$output/tektalk_${name}_plugin.dylib"
done
echo "macOS desktop plugins: $output"
