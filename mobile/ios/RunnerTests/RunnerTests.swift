import Flutter
import UIKit
import UserNotifications
import XCTest

@testable import Runner

class RunnerTests: XCTestCase {

  func testReminderAnswerKeepsOnlyTheIds() {
    let info: [AnyHashable: Any] = ["key": "k", "noteId": "n", "line": 4, "extra": "x"]
    let answer = KansoReminders.answer(userInfo: info, action: "open")
    XCTAssertEqual(answer?["key"] as? String, "k")
    XCTAssertEqual(answer?["noteId"] as? String, "n")
    XCTAssertEqual(answer?["line"] as? Int, 4)
    XCTAssertEqual(answer?["action"] as? String, "open")
    XCTAssertEqual(answer?.count, 4)
    XCTAssertNil(KansoReminders.answer(userInfo: ["key": "k"], action: "open"))
  }

  func testReminderActions() {
    XCTAssertEqual(
      KansoReminders.action(for: UNNotificationDefaultActionIdentifier),
      "open"
    )
    XCTAssertEqual(KansoReminders.action(for: "mark-done"), "mark-done")
    XCTAssertEqual(KansoReminders.action(for: "dismiss"), "dismiss")
    XCTAssertNil(KansoReminders.action(for: UNNotificationDismissActionIdentifier))
  }

  func testReminderPermissionMapping() {
    XCTAssertEqual(KansoReminders.permission(.authorized), "granted")
    XCTAssertEqual(KansoReminders.permission(.provisional), "granted")
    XCTAssertEqual(KansoReminders.permission(.ephemeral), "granted")
    XCTAssertEqual(KansoReminders.permission(.denied), "denied")
    XCTAssertEqual(KansoReminders.permission(.notDetermined), "notDetermined")
  }

  func testReminderDeliveryIsLiveOrKept() {
    XCTAssertEqual(
      KansoReminders.delivery(action: "open", engineUp: false, appActive: false),
      .persist
    )
    XCTAssertEqual(
      KansoReminders.delivery(action: "open", engineUp: true, appActive: false),
      .live
    )
    XCTAssertEqual(
      KansoReminders.delivery(action: "mark-done", engineUp: true, appActive: false),
      .persist
    )
    XCTAssertEqual(
      KansoReminders.delivery(action: "dismiss", engineUp: true, appActive: true),
      .live
    )
  }

  func testReminderMapParses() {
    let map: [String: Any] = [
      "key": "k", "noteId": "n", "line": 2, "title": "T", "body": "B",
      "year": 2026, "month": 10, "day": 25, "hour": 9, "minute": 0,
    ]
    let reminder = Reminder(map)
    XCTAssertEqual(reminder?.dateComponents.hour, 9)
    XCTAssertEqual(reminder?.userInfo.count, 3)
    XCTAssertNil(Reminder(["key": "k"]))
  }
}
