#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

required='clients/windows/TEKtalk.Windows/Domain/ClientLogic.h clients/windows/TEKtalk.Windows/Data/ApiClient.h clients/windows/TEKtalk.Windows/Infrastructure/RustCore.h clients/android/app/src/main/java/vn/tektalk/presentation/MainActivity.kt clients/android/app/src/main/java/vn/tektalk/domain/Models.kt clients/android/app/src/main/java/vn/tektalk/domain/Repositories.kt clients/android/app/src/main/java/vn/tektalk/domain/Logic.kt clients/android/app/src/main/java/vn/tektalk/data/HttpRepositories.kt clients/ios/TEKtalk/Presentation/ContentView.swift clients/ios/TEKtalk/Domain/ClientLogic.swift clients/ios/TEKtalk/Domain/Models.swift clients/ios/TEKtalk/Data/APIClient.swift clients/ios/TEKtalk/Infrastructure/NativeCore.swift clients/macos/TEKtalkMac/Presentation/ContentView.swift clients/macos/TEKtalkMac/Domain/Models.swift clients/macos/TEKtalkMac/Data/APIClient.swift clients/macos/TEKtalkMac/Infrastructure/RustCore.swift docs/client-clean-architecture.md'
for file in $required; do
  test -s "$root/$file" || { echo "missing client layer: $file" >&2; exit 1; }
done

if grep -RqsE '^import (android|androidx|okhttp3|kotlinx\.serialization)' "$root/clients/android/app/src/main/java/vn/tektalk/domain"; then
  echo 'Android Domain depends on Presentation/Data framework' >&2
  exit 1
fi
if grep -RqsE '^import (SwiftUI|UIKit|AppKit)' "$root/clients/ios/TEKtalk/Domain" "$root/clients/macos/TEKtalkMac/Domain"; then
  echo 'Apple Domain depends on a presentation framework' >&2
  exit 1
fi
echo 'client clean-architecture boundaries: ok'

