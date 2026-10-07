import AppKit

#if TEKTALK_MESSAGE_PLUGIN
private let pluginID = "tektalk.message"
private let titles = (en: "Message", vi: "Tin nhắn")
private let details = (en: "Native TEKtalk plugin • ABI 1", vi: "Plugin TEKtalk native • ABI 1")
#elseif TEKTALK_AI_PLUGIN
private let pluginID = "tektalk.ai"
private let titles = (en: "AI", vi: "AI")
private let details = (en: "Native TEKtalk plugin • ABI 1", vi: "Plugin TEKtalk native • ABI 1")
#else
private let pluginID = "tektalk.me"
private let titles = (en: "Me", vi: "Tôi")
private let details = (en: "Native TEKtalk plugin • ABI 1", vi: "Plugin TEKtalk native • ABI 1")
#endif

@_cdecl("tektalk_plugin_abi_version") public func pluginABIVersion() -> UInt32 { 1 }
@_cdecl("tektalk_plugin_id") public func nativePluginID() -> UnsafePointer<CChar>? { (pluginID as NSString).utf8String }
@_cdecl("tektalk_plugin_health_check") public func pluginHealthCheck() -> Int32 { 1 }

@_cdecl("tektalk_plugin_create_view") public func createPluginView(_ host: UnsafeRawPointer?) -> UnsafeMutableRawPointer? {
    let vietnamese = Locale.current.language.languageCode?.identifier == "vi"
    let stack = NSStackView()
    stack.orientation = .vertical
    stack.alignment = .leading
    stack.spacing = 12
    stack.edgeInsets = NSEdgeInsets(top: 24, left: 24, bottom: 24, right: 24)
    let heading = NSTextField(labelWithString: vietnamese ? titles.vi : titles.en)
    heading.font = .systemFont(ofSize: 30, weight: .bold)
    stack.addArrangedSubview(heading)
    let detail = NSTextField(labelWithString: vietnamese ? details.vi : details.en)
    detail.textColor = .secondaryLabelColor
    stack.addArrangedSubview(detail)
    return Unmanaged.passRetained(stack).toOpaque()
}

@_cdecl("tektalk_plugin_destroy_view") public func destroyPluginView(_ pointer: UnsafeMutableRawPointer?) {
    guard let pointer else { return }
    Unmanaged<NSView>.fromOpaque(pointer).release()
}
