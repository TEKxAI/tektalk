import Foundation
import AppKit
import ScreenCaptureKit
import AVFoundation

// Record only the actual application window using Apple's window capture filter.
// https://developer.apple.com/documentation/screencapturekit/sccontentfilter
@available(macOS 15.0, *)
final class RecordingEvents: NSObject, SCRecordingOutputDelegate {
    let ready: URL
    let finished: URL
    let failed: URL
    init(ready: URL, finished: URL, failed: URL) {
        self.ready=ready; self.finished=finished; self.failed=failed
    }
    func recordingOutputDidStartRecording(_ output: SCRecordingOutput) {
        try? Data().write(to: ready)
    }
    func recordingOutputDidFinishRecording(_ output: SCRecordingOutput) {
        try? Data().write(to: finished)
    }
    func recordingOutput(_ output: SCRecordingOutput, didFailWithError error: Error) {
        try? Data(error.localizedDescription.utf8).write(to: failed)
    }
}

@main @available(macOS 15.0, *)
struct RecordWindow {
    @MainActor static func main() async throws {
        // Connect this command-line recorder to WindowServer before ScreenCaptureKit.
        let application=NSApplication.shared
        application.setActivationPolicy(.accessory)
        _ = CGMainDisplayID()
        guard CommandLine.arguments.count == 4 else { fatalError("Usage: record-window OUTPUT STOP READY") }
        let output=URL(fileURLWithPath:CommandLine.arguments[1])
        let stop=CommandLine.arguments[2]
        let ready=URL(fileURLWithPath:CommandLine.arguments[3])
        let finished=output.appendingPathExtension("finished")
        let failed=output.appendingPathExtension("failed")
        var window: SCWindow?
        let deadline=Date().addingTimeInterval(150)
        while window == nil && Date() < deadline {
            let content=try await SCShareableContent.excludingDesktopWindows(true,onScreenWindowsOnly:true)
            window=content.windows.first {
                $0.owningApplication?.bundleIdentifier == "vn.tektalk.macos" && $0.frame.width > 500 && $0.frame.height > 400
            }
            if window == nil { try await Task.sleep(nanoseconds:200_000_000) }
        }
        guard let window else { throw NSError(domain:"MVPRecorder",code:1,userInfo:[NSLocalizedDescriptionKey:"TEKtalk window did not appear"]) }
        let config=SCStreamConfiguration()
        config.scalesToFit=true
        config.width=1440
        config.height=(Int(1440 * window.frame.height / window.frame.width) / 2) * 2
        config.minimumFrameInterval=CMTime(value:1,timescale:24)
        config.showsCursor=true
        config.capturesAudio=false
        let stream=SCStream(filter:SCContentFilter(desktopIndependentWindow:window),configuration:config,delegate:nil)
        let recordingConfig=SCRecordingOutputConfiguration()
        recordingConfig.outputURL=output
        recordingConfig.outputFileType = .mp4
        recordingConfig.videoCodecType = .h264
        let events=RecordingEvents(ready:ready,finished:finished,failed:failed)
        let recording=SCRecordingOutput(configuration:recordingConfig,delegate:events)
        try stream.addRecordingOutput(recording)
        try await stream.startCapture()
        let timeout=Date().addingTimeInterval(600)
        while !FileManager.default.fileExists(atPath:stop) && Date() < timeout && !FileManager.default.fileExists(atPath:failed.path) {
            try await Task.sleep(nanoseconds:100_000_000)
        }
        try await stream.stopCapture()
        let flushDeadline=Date().addingTimeInterval(30)
        while !FileManager.default.fileExists(atPath:finished.path) && !FileManager.default.fileExists(atPath:failed.path) && Date() < flushDeadline {
            try await Task.sleep(nanoseconds:100_000_000)
        }
        if FileManager.default.fileExists(atPath:failed.path) {
            throw NSError(domain:"MVPRecorder",code:2,userInfo:[NSLocalizedDescriptionKey:try String(contentsOf:failed,encoding:.utf8)])
        }
        guard FileManager.default.fileExists(atPath:finished.path), FileManager.default.fileExists(atPath:stop) else {
            throw NSError(domain:"MVPRecorder",code:3,userInfo:[NSLocalizedDescriptionKey:"Recording did not finish with the completed UI flow"])
        }
        print("Completed continuous TEKtalk window recording")
    }
}
