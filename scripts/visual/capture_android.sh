#!/usr/bin/env bash
# Keep this in one Bash process: emulator-runner executes inline lines separately.
set -euo pipefail

mkdir -p screenshots
diagnostics() {
  local status=$?
  if (( status != 0 )); then
    adb logcat -d -t 500 > screenshots/android-logcat.txt 2>&1 || true
    adb shell dumpsys window > screenshots/android-windows.txt 2>&1 || true
  fi
  exit "$status"
}
trap diagnostics EXIT

adb wait-for-device
adb install -r clients/android/app/build/outputs/apk/debug/app-debug.apk
adb shell input keyevent KEYCODE_WAKEUP
adb shell wm dismiss-keyguard
adb shell am start -W -n vn.tektalk/.presentation.MainActivity

focused=false
for attempt in {1..30}; do
  focus=$(adb shell dumpsys window windows | grep -E 'mCurrentFocus|mFocusedApp' || true)
  if [[ "$focus" == *vn.tektalk* ]]; then
    focused=true
    break
  fi
  sleep 2
done
if [[ "$focused" != true ]]; then
  echo 'TEKtalk did not become the focused Android application' >&2
  exit 1
fi

sleep 3
adb exec-out screencap -p > screenshots/android.png
test -s screenshots/android.png
