import AppKit

#if TEKTALK_MESSAGE_PLUGIN
private let pluginID = "tektalk.message"
private let title = "Message"
#elseif TEKTALK_AI_PLUGIN
private let pluginID = "tektalk.ai"
private let title = "AI"
#else
private let pluginID = "tektalk.me"
private let title = "Me"
#endif

@_cdecl("tektalk_plugin_abi_version") public func pluginABIVersion() -> UInt32 { 1 }
@_cdecl("tektalk_plugin_id") public func nativePluginID() -> UnsafePointer<CChar>? { (pluginID as NSString).utf8String }
@_cdecl("tektalk_plugin_health_check") public func pluginHealthCheck() -> Int32 { 1 }

@_cdecl("tektalk_plugin_create_view") public func createPluginView(_ host: UnsafeRawPointer?) -> UnsafeMutableRawPointer? {
    let stack = NSStackView()
    stack.orientation = .vertical
    stack.alignment = .leading
    stack.spacing = 12
    let heading = NSTextField(labelWithString: title)
    heading.font = .systemFont(ofSize: 30, weight: .bold)
    stack.addArrangedSubview(heading)
    stack.addArrangedSubview(NSTextField(labelWithString: "Native TEKtalk plugin • ABI 1"))
    return Unmanaged.passRetained(stack).toOpaque()
}

@_cdecl("tektalk_plugin_destroy_view") public func destroyPluginView(_ pointer: UnsafeMutableRawPointer?) {
    guard let pointer else { return }
    Unmanaged<NSView>.fromOpaque(pointer).release()
}
