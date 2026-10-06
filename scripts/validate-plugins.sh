#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

ids=''
orders=''
for manifest in "$root"/plugins/*/manifest.json; do
  jq -e '
    .schemaVersion == 2 and
    (.id | test("^[a-z][a-z0-9.-]{2,127}$")) and
    (.version | test("^[0-9]+\\.[0-9]+\\.[0-9]+$")) and
    .pluginType == "first_party_tab" and
    .minimumCoreAbi >= 1 and
    (.valdiRuntime | length > 0) and
    (.capabilities | type == "array") and
    (.networkAllowlist | type == "array") and
    (.rolloutPercentage >= 0 and .rolloutPercentage <= 100) and
    (.tab.order >= 0 and .tab.order <= 9)
  ' "$manifest" >/dev/null
  id=$(jq -r .id "$manifest")
  order=$(jq -r .tab.order "$manifest")
  case " $ids " in *" $id "*) echo "duplicate plugin id: $id" >&2; exit 1;; esac
  case " $orders " in *" $order "*) echo "duplicate tab order: $order" >&2; exit 1;; esac
  ids="$ids $id"
  orders="$orders $order"
done

jq -e '.required == true' "$root/plugins/message/manifest.json" >/dev/null
jq -e '.required == true' "$root/plugins/me/manifest.json" >/dev/null
jq -e '.applicationId and .riskTier and .storageQuotaBytes > 0' "$root/mini-apps/examples/hello/manifest.json" >/dev/null
echo "plugin and mini-app manifests: ok"
