#!/usr/bin/env bash
set -euo pipefail
mkdir -p recordings
trap 'status=$?; if ((status)); then adb logcat -d -t 1000 > recordings/android-logcat.log || true; adb shell uiautomator dump /sdcard/failure.xml >/dev/null || true; adb exec-out cat /sdcard/failure.xml > recordings/android-ui.log || true; fi; exit "$status"' EXIT
adb wait-for-device
adb install -r clients/android/app/build/outputs/apk/debug/app-debug.apk
adb shell input keyevent KEYCODE_WAKEUP
adb shell wm dismiss-keyguard
python3 scripts/visual/secret_peer.py serve --fixture build/visual-fixtures/android.json > recordings/android-secret-peer.log 2>&1 &
for attempt in $(seq 1 30); do
  if curl --fail --silent http://127.0.0.1:8766/ >/dev/null; then break; fi
  sleep 1
done
curl --fail --silent http://127.0.0.1:8766/ >/dev/null
python3 scripts/visual/record_android_mvp.py
