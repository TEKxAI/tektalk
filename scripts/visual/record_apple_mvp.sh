#!/usr/bin/env bash
set -euo pipefail
root=$(pwd)
platform=${1:?Choose ios or macos}
case "$platform" in ios|macos) ;; *) exit 2 ;; esac
mkdir -p recordings
brew install xcodegen ffmpeg
sudo DevToolsSecurity -enable
if [ "$platform" = ios ]; then
python3 scripts/visual/scenario.py prepare --platform ios --fixture clients/ios/TEKtalkFlowTests/fixture.json
cd clients/ios
xcodegen generate
udid=$(xcrun simctl list devices available -j | python3 -c 'import json,sys;print(next(x["udid"] for xs in json.load(sys.stdin)["devices"].values() for x in xs if x["name"].startswith("iPhone") and x["isAvailable"]))')
xcrun simctl boot "$udid" || true
xcrun simctl bootstatus "$udid" -b
xcodebuild build-for-testing -project TEKtalk.xcodeproj -scheme TEKtalkMVP -destination "platform=iOS Simulator,id=$udid" -derivedDataPath build-mvp CODE_SIGNING_ALLOWED=NO > "$root/recordings/ios-build.log" 2>&1
xcrun simctl io "$udid" recordVideo --codec=h264 "$root/recordings/ios.mp4" > "$root/recordings/ios-recorder.log" 2>&1 &
recorder=$!
stop_ios(){ kill -INT "$recorder" 2>/dev/null || true; wait "$recorder" || true; }
trap stop_ios EXIT
xcodebuild test-without-building -parallel-testing-enabled NO -project TEKtalk.xcodeproj -scheme TEKtalkMVP -destination "platform=iOS Simulator,id=$udid" -derivedDataPath build-mvp CODE_SIGNING_ALLOWED=NO > "$root/recordings/ios-test.log" 2>&1
grep -Eq "testRegisterLogoutLoginAndChat.*passed" "$root/recordings/ios-test.log"
kill -0 "$recorder"
stop_ios
trap - EXIT
cd "$root"
python3 scripts/visual/scenario.py verify-existing --fixture clients/ios/TEKtalkFlowTests/fixture.json
python3 -c 'import json,pathlib;pathlib.Path("recordings/ios-flow.json").write_text(json.dumps(dict(platform="ios",registration=True,logout=True,login=True,verification=True,sent_and_received_in_ui=True)))'
else
python3 scripts/visual/scenario.py prepare --platform macos --fixture clients/macos/TEKtalkMacFlowTests/fixture.json
cd clients/macos
xcodegen generate
xcodebuild build-for-testing -project TEKtalkMac.xcodeproj -scheme TEKtalkMacMVP -destination 'platform=macOS' -derivedDataPath build-mvp CODE_SIGNING_ALLOWED=NO > "$root/recordings/macos-build.log" 2>&1
screencapture -v -V 600 -C -x "$root/recordings/macos.mov" > "$root/recordings/macos-recorder.log" 2>&1 &
recorder=$!
stop_mac(){ kill -INT "$recorder" 2>/dev/null || true; wait "$recorder" || true; }
trap stop_mac EXIT
xcodebuild test-without-building -parallel-testing-enabled NO -project TEKtalkMac.xcodeproj -scheme TEKtalkMacMVP -destination 'platform=macOS' -derivedDataPath build-mvp CODE_SIGNING_ALLOWED=NO > "$root/recordings/macos-test.log" 2>&1
grep -Eq "testRegisterLogoutLoginAndChat.*passed" "$root/recordings/macos-test.log"
kill -0 "$recorder"
stop_mac
trap - EXIT
cd "$root"
python3 scripts/visual/scenario.py verify-existing --fixture clients/macos/TEKtalkMacFlowTests/fixture.json
ffmpeg -y -i recordings/macos.mov -c:v libx264 -pix_fmt yuv420p -movflags +faststart recordings/macos.mp4
python3 -c 'import json,pathlib;pathlib.Path("recordings/macos-flow.json").write_text(json.dumps(dict(platform="macos",registration=True,logout=True,login=True,verification=True,sent_and_received_in_ui=True)))'
fi
