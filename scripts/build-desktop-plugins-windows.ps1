$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Project = Join-Path $Root "plugins/desktop/windows/TEKtalk.NativePlugin.vcxproj"
$Variants = @(
  @{ Name = "tektalk_message_plugin"; Define = "TEKTALK_MESSAGE_PLUGIN" },
  @{ Name = "tektalk_ai_plugin"; Define = "TEKTALK_AI_PLUGIN" },
  @{ Name = "tektalk_me_plugin"; Define = "TEKTALK_ME_PLUGIN" }
)
foreach ($Variant in $Variants) {
  msbuild $Project /restore /m /p:Configuration=Release /p:Platform=x64 /p:PluginTargetName=$($Variant.Name) /p:PluginDefine=$($Variant.Define)
}
Write-Host "Windows desktop plugins: plugins/desktop/windows/build/x64/Release"
