import Flutter
import UIKit
import UserNotifications

/// Reminders on iOS, as local notifications: the `app.kanso/reminders` method
/// channel (docs/journal-tasks.md). Dart decides what is due and sends the
/// words; this schedules, shows and removes them, and hands the taps back.
///
/// A reminder carries only `key`, `noteId` and `line` besides its words, and
/// every identifier starts with `kanso-reminder:`, so `cancelAll` leaves
/// anything else in the notification centre alone.
///
/// A tap on the notification (`open`), on Mark Done or on Dismiss is answered
/// once: live, over the channel, when the app runs and can take it, or kept in
/// UserDefaults until `takeResponses`. Never both, as Dart applies whatever it
/// receives exactly once.
final class KansoReminders: NSObject {
  static let shared = KansoReminders()

  static let channelName = "app.kanso/reminders"
  static let category = "kanso-reminder"
  static let identifierPrefix = "kanso-reminder:"
  static let markDone = "mark-done"
  static let dismiss = "dismiss"
  static let open = "open"
  static let storeKey = "app.kanso.reminders.responses"

  /// The most iOS keeps pending for an app; the soonest ones win.
  static let pendingLimit = 64

  private var channel: FlutterMethodChannel?

  /// Answers the `app.kanso/reminders` channel and registers the category.
  func register(with messenger: FlutterBinaryMessenger) {
    let channel = FlutterMethodChannel(
      name: KansoReminders.channelName,
      binaryMessenger: messenger
    )
    self.channel = channel
    channel.setMethodCallHandler { [weak self] call, result in
      self?.handle(call, result: result)
    }
    registerCategory()
  }

  // MARK: Channel

  private func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    let center = UNUserNotificationCenter.current()
    switch call.method {
    case "permission":
      center.getNotificationSettings { settings in
        let answer = KansoReminders.permission(settings.authorizationStatus)
        DispatchQueue.main.async { result(answer) }
      }
    case "requestPermission":
      center.requestAuthorization(options: [.alert, .sound]) { granted, _ in
        DispatchQueue.main.async { result(granted) }
      }
    case "show":
      let reminders = KansoReminders.reminders(call.arguments)
      add(reminders, scheduled: false) { result(nil) }
    case "schedule":
      let reminders = KansoReminders.reminders(call.arguments)
      center.getPendingNotificationRequests { pending in
        let ours = pending.map(\.identifier)
          .filter { $0.hasPrefix(KansoReminders.identifierPrefix) }
        center.removePendingNotificationRequests(withIdentifiers: ours)
        self.add(reminders, scheduled: true) { result(nil) }
      }
    case "cancelAll":
      cancelAll { result(nil) }
    case "takeResponses":
      result(KansoReminders.takeStored())
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  private func registerCategory() {
    let actions = [
      UNNotificationAction(
        identifier: KansoReminders.markDone,
        title: "Mark Done",
        options: []
      ),
      UNNotificationAction(
        identifier: KansoReminders.dismiss,
        title: "Dismiss",
        options: []
      ),
    ]
    let category = UNNotificationCategory(
      identifier: KansoReminders.category,
      actions: actions,
      intentIdentifiers: [],
      options: []
    )
    let center = UNUserNotificationCenter.current()
    center.getNotificationCategories { existing in
      center.setNotificationCategories(
        existing.filter { $0.identifier != KansoReminders.category }
          .union([category])
      )
    }
  }

  private func add(
    _ reminders: [Reminder],
    scheduled: Bool,
    done: @escaping () -> Void
  ) {
    var wanted = reminders
    if scheduled {
      let now = Date()
      wanted = reminders
        .compactMap { reminder -> (Reminder, Date)? in
          guard let date = reminder.fireDate else { return nil }
          return date > now ? (reminder, date) : nil
        }
        .sorted { $0.1 < $1.1 }
        .prefix(KansoReminders.pendingLimit)
        .map { $0.0 }
    }
    let center = UNUserNotificationCenter.current()
    let group = DispatchGroup()
    for reminder in wanted {
      let content = UNMutableNotificationContent()
      content.title = reminder.title
      content.body = reminder.body
      content.sound = .default
      content.categoryIdentifier = KansoReminders.category
      content.userInfo = reminder.userInfo
      let trigger: UNNotificationTrigger? =
        scheduled
        ? UNCalendarNotificationTrigger(
          dateMatching: reminder.dateComponents,
          repeats: false
        )
        : nil
      let request = UNNotificationRequest(
        identifier: KansoReminders.identifierPrefix + reminder.key,
        content: content,
        trigger: trigger
      )
      group.enter()
      center.add(request) { _ in group.leave() }
    }
    group.notify(queue: .main, execute: done)
  }

  private func cancelAll(done: @escaping () -> Void) {
    let center = UNUserNotificationCenter.current()
    let prefix = KansoReminders.identifierPrefix
    center.getPendingNotificationRequests { pending in
      center.removePendingNotificationRequests(
        withIdentifiers: pending.map(\.identifier).filter { $0.hasPrefix(prefix) }
      )
      center.getDeliveredNotifications { delivered in
        center.removeDeliveredNotifications(
          withIdentifiers: delivered.map(\.request.identifier)
            .filter { $0.hasPrefix(prefix) }
        )
        DispatchQueue.main.async(execute: done)
      }
    }
  }

  // MARK: Taps

  /// Whether `notification` is one of ours.
  func isReminder(_ notification: UNNotification) -> Bool {
    notification.request.identifier.hasPrefix(KansoReminders.identifierPrefix)
  }

  /// A tap on a reminder or one of its buttons; calls `done` when it is dealt
  /// with, whichever way.
  func respond(to response: UNNotificationResponse, done: @escaping () -> Void) {
    let info = response.notification.request.content.userInfo
    guard
      let action = KansoReminders.action(for: response.actionIdentifier),
      let answer = KansoReminders.answer(userInfo: info, action: action)
    else {
      done()
      return
    }
    let active = UIApplication.shared.applicationState == .active
    switch KansoReminders.delivery(
      action: action,
      engineUp: channel != nil,
      appActive: active
    ) {
    case .live:
      channel?.invokeMethod("response", arguments: answer) { reply in
        // Dart isn't listening yet (or failed): keep it for takeResponses.
        if reply is FlutterError || (reply as? NSObject) === FlutterMethodNotImplemented {
          KansoReminders.store(answer)
        }
        done()
      }
    case .persist:
      KansoReminders.store(answer)
      done()
    }
  }

  // MARK: Pure parts

  enum Delivery: Equatable {
    case live
    case persist
  }

  /// One answer goes one way. A tap on the notification is sent live whenever
  /// the engine is up (the app is coming forward); a button is sent live only
  /// when the app is in front, as a suspended app may not get to read it.
  /// Otherwise it is kept until `takeResponses`.
  static func delivery(action: String, engineUp: Bool, appActive: Bool) -> Delivery {
    guard engineUp else { return .persist }
    if action == open { return .live }
    return appActive ? .live : .persist
  }

  /// `open`, `mark-done` or `dismiss` for a notification action identifier;
  /// nil for a swipe away or anything else.
  static func action(for identifier: String) -> String? {
    switch identifier {
    case UNNotificationDefaultActionIdentifier: return open
    case markDone: return markDone
    case dismiss: return dismiss
    default: return nil
    }
  }

  /// `{key, noteId, line, action}` from a notification's `userInfo`.
  static func answer(userInfo: [AnyHashable: Any], action: String) -> [String: Any]? {
    guard
      let key = userInfo["key"] as? String,
      let noteId = userInfo["noteId"] as? String,
      let line = (userInfo["line"] as? NSNumber)?.intValue
    else { return nil }
    return ["key": key, "noteId": noteId, "line": line, "action": action]
  }

  static func permission(_ status: UNAuthorizationStatus) -> String {
    switch status {
    case .authorized, .provisional, .ephemeral: return "granted"
    case .denied: return "denied"
    default: return "notDetermined"
    }
  }

  /// The reminder maps of a `schedule` or `show` call; a malformed one is left out.
  static func reminders(_ arguments: Any?) -> [Reminder] {
    guard let list = arguments as? [Any] else { return [] }
    return list.compactMap { ($0 as? [String: Any]).flatMap(Reminder.init) }
  }

  // MARK: Kept answers

  static func store(_ answer: [String: Any]) {
    let defaults = UserDefaults.standard
    var kept = defaults.array(forKey: storeKey) as? [[String: Any]] ?? []
    kept.append(answer)
    defaults.set(kept, forKey: storeKey)
  }

  /// Returns what was kept, and forgets it.
  static func takeStored() -> [[String: Any]] {
    let defaults = UserDefaults.standard
    let kept = defaults.array(forKey: storeKey) as? [[String: Any]] ?? []
    defaults.removeObject(forKey: storeKey)
    return kept
  }
}

/// A reminder as Dart sends it.
struct Reminder {
  let key: String
  let noteId: String
  let line: Int
  let title: String
  let body: String
  let dateComponents: DateComponents

  init?(_ map: [String: Any]) {
    func int(_ name: String) -> Int? { (map[name] as? NSNumber)?.intValue }
    guard
      let key = map["key"] as? String,
      let noteId = map["noteId"] as? String,
      let line = int("line"),
      let year = int("year"), let month = int("month"), let day = int("day"),
      let hour = int("hour"), let minute = int("minute")
    else { return nil }
    self.key = key
    self.noteId = noteId
    self.line = line
    self.title = map["title"] as? String ?? ""
    self.body = map["body"] as? String ?? ""
    self.dateComponents = DateComponents(
      year: year,
      month: month,
      day: day,
      hour: hour,
      minute: minute
    )
  }

  /// Nothing but the three ids travels with the notification.
  var userInfo: [String: Any] {
    ["key": key, "noteId": noteId, "line": line]
  }

  var fireDate: Date? { Calendar.current.date(from: dateComponents) }
}
