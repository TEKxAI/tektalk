import XCTest
import Foundation

final class MVPFlowTests: XCTestCase {
    private var fixture: [String:String] = [:]
    private let app = XCUIApplication()
    override func setUpWithError() throws {
        continueAfterFailure = false
        let url = try XCTUnwrap(Bundle(for: Self.self).url(forResource: "fixture", withExtension: "json"))
        fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? [String:String])
        app.launchArguments = ["-AppleLanguages", "(en)", "-AppleLocale", "en_US"]
        app.launch()
        #if os(macOS)
        if let ready=fixture["recording_ready"] {
            let deadline=Date().addingTimeInterval(120)
            while !FileManager.default.fileExists(atPath:ready) && Date() < deadline { Thread.sleep(forTimeInterval:0.2) }
            XCTAssertTrue(FileManager.default.fileExists(atPath:ready), "Window recording did not start")
        }
        #endif
    }
    private func element(_ id: String) -> XCUIElement {
        let e = app.descendants(matching: .any).matching(identifier:id).firstMatch
        XCTAssertTrue(e.waitForExistence(timeout:30), "Missing UI element \(id): \(app.debugDescription)")
        return e
    }
    private func tap(_ id: String) {
        let e = element(id)
        #if os(iOS)
        for _ in 0..<4 { if e.isHittable { break }; app.swipeUp() }
        #endif
        e.tap();Thread.sleep(forTimeInterval:0.7)
    }
    private func fill(_ id: String, _ value: String) {
        let e=element(id);e.tap()
        #if os(macOS)
        e.typeKey("a", modifierFlags:.command)
        #else
        // Tap the trailing edge so deletion starts after the whole existing value.
        e.coordinate(withNormalizedOffset:CGVector(dx:0.98,dy:0.5)).tap()
        let count=(e.value as? String)?.count ?? 0
        e.typeText(String(repeating:XCUIKeyboardKey.delete.rawValue,count:count))
        #endif
        e.typeText(value);Thread.sleep(forTimeInterval:0.5)
        if id.hasPrefix("chat.") {
            XCTAssertEqual(e.value as? String, value, "Input did not reach \(id)")
            #if os(iOS)
            tap("chat.keyboard.done")
            #endif
        }
    }
    private func post(_ path: String, _ body: [String:Any]) throws -> Any {
        let base=path=="/reply" ? "http://127.0.0.1:8766" : "http://127.0.0.1:8080"
        var request=URLRequest(url:URL(string:base+path)!)
        request.httpMethod="POST";request.setValue("application/json",forHTTPHeaderField:"Content-Type")
        request.setValue("Bearer "+fixture["peer_token"]!,forHTTPHeaderField:"Authorization")
        request.httpBody=try JSONSerialization.data(withJSONObject:body)
        let done=expectation(description:path)
        var result:Data?;var response:URLResponse?;var failure:Error?
        URLSession.shared.dataTask(with:request){data,r,error in result=data;response=r;failure=error;done.fulfill()}.resume()
        wait(for:[done],timeout:30)
        if let failure {throw failure}
        XCTAssertEqual((response as? HTTPURLResponse)?.statusCode,200)
        return try JSONSerialization.jsonObject(with:try XCTUnwrap(result))
    }
    func testRegisterLogoutLoginAndChat() throws {
        let f=fixture
        Thread.sleep(forTimeInterval:2);tap("auth.toggle")
        for (id,key) in [("auth.phone","phone"),("auth.password","password"),("auth.question","question"),("auth.answer","answer")] {
            if id=="auth.password" { fill("auth.name","MVP Learner") }
            fill(id,f[key]!)
        }
        Thread.sleep(forTimeInterval:1);tap("auth.submit")
        #if os(macOS)
        let tabs=element("navigation.bottom").frame
        let sidebar=element("sidebar.content").frame
        XCTAssertGreaterThanOrEqual(tabs.minY, sidebar.maxY - 1, "Tabs must sit below history")
        tap("tab.ai")
        XCTAssertTrue(app.staticTexts["TEK AI"].firstMatch.waitForExistence(timeout:10))
        tap("tab.me")
        #else
        let me=app.tabBars.buttons["Me"];XCTAssertTrue(me.waitForExistence(timeout:30));me.tap()
        #endif
        tap("auth.logout");fill("auth.phone",f["phone"]!);fill("auth.password",f["password"]!);tap("auth.submit")
        _=element("auth.answer");Thread.sleep(forTimeInterval:2);fill("auth.answer",f["answer"]!);tap("auth.submit")
        #if os(iOS)
        tap("chat.open")
        #endif
        fill("chat.conversation",f["conversation"]!);fill("chat.recipient",f["peer_id"]!);fill("chat.draft",f["outgoing"]!)
        XCTAssertTrue(element("chat.send").isEnabled, "Send disabled: \(app.debugDescription)")
        tap("chat.send")
        #if os(macOS)
        let sent=element("chat.messages").staticTexts[f["outgoing"]!].firstMatch
        #else
        let sent=app.staticTexts[f["outgoing"]!].firstMatch
        #endif
        XCTAssertTrue(sent.waitForExistence(timeout:30))
        XCTAssertTrue(sent.isHittable, "Outgoing message is outside the visible chat")
        Thread.sleep(forTimeInterval:2)
        let rows=try XCTUnwrap(post("/v1/chat/messages/history",["conversation_id":f["conversation"]!,"limit":50]) as? [[String:Any]])
        let outgoing=try XCTUnwrap(rows.first {($0["body"] as? String)==f["outgoing"]})
        _=try post("/v1/chat/messages/send",["conversation_id":f["conversation"]!,"recipient_id":try XCTUnwrap(outgoing["sender_id"]),"client_message_id":UUID().uuidString,"text":f["incoming"]!])
        tap("chat.sync")
        #if os(macOS)
        let received=element("chat.messages").staticTexts[f["incoming"]!].firstMatch
        #else
        let received=app.staticTexts[f["incoming"]!].firstMatch
        #endif
        XCTAssertTrue(received.waitForExistence(timeout:30))
        XCTAssertTrue(received.isHittable, "Reply is outside the visible chat")
        received.tap()
        #if os(iOS)
        // Exercise the visible opt-in with a real independent Rust Core peer.
        tap("chat.secret")
        for kind in ["one_to_one", "group"] {
            let conversation=UUID().uuidString.lowercased()
            fill("chat.conversation",conversation)
            if kind=="group" { tap("chat.secret.group");fill("chat.secret.members",f["peer_id"]!) }
            tap("chat.secret.enable")
            Thread.sleep(forTimeInterval:2)
            let outgoing="Encrypted "+kind+" from iOS"
            let incoming="Verified encrypted "+kind+" reply"
            fill("chat.draft",outgoing);tap("chat.send")
            XCTAssertTrue(app.staticTexts[outgoing].firstMatch.waitForExistence(timeout:30))
            _=try post("/reply",["conversation_id":conversation,"outgoing":outgoing,"incoming":incoming])
            tap("chat.sync")
            XCTAssertTrue(app.staticTexts[incoming].firstMatch.waitForExistence(timeout:30))
            tap("chat.sync")
            XCTAssertTrue(app.staticTexts[incoming].firstMatch.waitForExistence(timeout:30))
            Thread.sleep(forTimeInterval:2)
        }
        #endif
        #if os(macOS)
        fill("chat.draft","Draft preserved across tabs")
        tap("tab.ai");tap("tab.me");tap("tab.message")
        XCTAssertEqual(element("chat.draft").value as? String,"Draft preserved across tabs")
        XCTAssertTrue(received.isHittable, "History must survive switching tabs")
        XCTAssertTrue(element("history."+f["conversation"]!).isHittable)
        #endif
        let proof=XCTAttachment(screenshot:app.screenshot())
        proof.name="visible-bidirectional-chat";proof.lifetime = .keepAlways;add(proof)
        Thread.sleep(forTimeInterval:8)
        #if os(macOS)
        if let stop=fixture["recording_stop"] {
            try Data().write(to:URL(fileURLWithPath:stop))
            Thread.sleep(forTimeInterval:2)
        }
        #endif
    }
}
