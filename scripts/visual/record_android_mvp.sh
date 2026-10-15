#!/usr/bin/env bash
set -euo pipefail
mkdir -p recordings
trap 'status=$?; if ((status)); then adb logcat -d -t 1000 > recordings/android-logcat.log || true; adb shell uiautomator dump /sdcard/failure.xml >/dev/null || true; adb exec-out cat /sdcard/failure.xml > recordings/android-ui.log || true; fi; exit "$status"' EXIT
adb wait-for-device
adb install -r clients/android/app/build/outputs/apk/debug/app-debug.apk
adb shell input keyevent KEYCODE_WAKEUP
adb shell wm dismiss-keyguard
python3 scripts/visual/record_android_mvp.py
