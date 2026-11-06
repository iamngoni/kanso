import EventKit
import Flutter
import Foundation
import UIKit

/// Read-only EventKit access for Overview. Calendar events stay on this device.
final class KansoCalendar {
  static let shared = KansoCalendar()
  private let store = EKEventStore()
  private let channelName = "app.kanso.kanso/system_calendar"

  func register(messenger: FlutterBinaryMessenger) {
    let channel = FlutterMethodChannel(name: channelName, binaryMessenger: messenger)
    channel.setMethodCallHandler { [weak self] call, result in
      guard let self else {
        result(FlutterError(code: "calendar.unavailable", message: "Calendar access is unavailable.", details: nil))
        return
      }
      switch call.method {
      case "requestAccess":
        self.requestAccess(result)
      case "calendars":
        guard self.hasAccess else {
          result(FlutterError(code: "calendar.permission", message: "Allow calendar access in Settings to read events.", details: nil))
          return
        }
        result(self.calendars())
      case "setCalendarEnabled":
        guard let args = call.arguments as? [String: Any],
              let id = args["id"] as? String,
              let enabled = args["enabled"] as? Bool,
              !id.isEmpty
        else {
          result(FlutterError(code: "calendar.selection", message: "Invalid calendar selection.", details: nil))
          return
        }
        var disabled = self.disabledCalendarIds
        if enabled { disabled.remove(id) } else { disabled.insert(id) }
        UserDefaults.standard.set(Array(disabled), forKey: "overview.calendar.disabled")
        result(nil)
      case "openEvent":
        guard let args = call.arguments as? [String: Any],
              let start = args["start"] as? Int64
        else {
          result(FlutterError(code: "calendar.event", message: "Invalid calendar event.", details: nil))
          return
        }
        guard let url = URL(string: "calshow:\(start / 1000)") else {
          result(FlutterError(code: "calendar.open", message: "Couldn't open the event in Calendar.", details: nil))
          return
        }
        UIApplication.shared.open(url) { opened in
          result(opened ? nil : FlutterError(code: "calendar.open", message: "Couldn't open the event in Calendar.", details: nil))
        }
      case "events":
        guard self.hasAccess else {
          result(FlutterError(code: "calendar.permission", message: "Allow calendar access in Settings to read events.", details: nil))
          return
        }
        guard let args = call.arguments as? [String: Any],
              let start = args["start"] as? Int64,
              let end = args["end"] as? Int64,
              start < end
        else {
          result(FlutterError(code: "calendar.range", message: "Invalid calendar range.", details: nil))
          return
        }
        result(self.events(start: start, end: end))
      default:
        result(FlutterMethodNotImplemented)
      }
    }
  }

  private var hasAccess: Bool {
    if #available(iOS 17.0, *) {
      return EKEventStore.authorizationStatus(for: .event) == .fullAccess
    }
    return EKEventStore.authorizationStatus(for: .event) == .authorized
  }

  private func requestAccess(_ result: @escaping FlutterResult) {
    if hasAccess {
      result(true)
      return
    }
    let completion: (Bool, Error?) -> Void = { granted, error in
      DispatchQueue.main.async {
        if let error {
          result(FlutterError(code: "calendar.permission", message: error.localizedDescription, details: nil))
        } else {
          result(granted)
        }
      }
    }
    if #available(iOS 17.0, *) {
      store.requestFullAccessToEvents(completion: completion)
    } else {
      store.requestAccess(to: .event, completion: completion)
    }
  }

  private func calendars() -> [[String: Any]] {
    let all = store.calendars(for: .event)
    let disabled = disabledCalendarIds
    return all.map { calendar in
      [
        "id": calendar.calendarIdentifier,
        "name": calendar.title,
        "color": hex(calendar.cgColor),
        "enabled": !disabled.contains(calendar.calendarIdentifier),
      ]
    }
  }

  private var disabledCalendarIds: Set<String> {
    Set(UserDefaults.standard.stringArray(forKey: "overview.calendar.disabled") ?? [])
  }

  private func events(start: Int64, end: Int64) -> [[String: Any]] {
    let from = Date(timeIntervalSince1970: Double(start) / 1000)
    let to = Date(timeIntervalSince1970: Double(end) / 1000)
    let disabled = disabledCalendarIds
    let calendars = store.calendars(for: .event).filter {
      !disabled.contains($0.calendarIdentifier)
    }
    let predicate = store.predicateForEvents(withStart: from, end: to, calendars: calendars)
    return store.events(matching: predicate).map { event in
      let attendees: [[String: String]] = (event.attendees ?? []).compactMap { attendee in
        guard let name = attendee.name, !name.isEmpty else { return nil }
        let address = attendee.url.absoluteString
        let email = address.hasPrefix("mailto:") ? String(address.dropFirst(7)) : ""
        return ["name": name, "email": email]
      }
      let declined = (event.attendees ?? []).contains { attendee in
        attendee.isCurrentUser && attendee.participantStatus == .declined
      }
      return [
        "uid": event.calendarItemExternalIdentifier ?? event.eventIdentifier ?? "",
        "start": Int64(event.startDate.timeIntervalSince1970 * 1000),
        "end": Int64(event.endDate.timeIntervalSince1970 * 1000),
        "title": event.title ?? "",
        "location": event.location ?? "",
        "conferenceUrl": event.url?.absoluteString ?? "",
        "attendees": attendees,
        "calendarId": event.calendar.calendarIdentifier,
        "calendar": event.calendar.title,
        "color": hex(event.calendar.cgColor),
        "allDay": event.isAllDay,
        "free": event.availability == .free,
        "declined": declined,
      ]
    }
  }

  private func hex(_ color: CGColor) -> String {
    guard let rgb = color.converted(to: CGColorSpaceCreateDeviceRGB(), intent: .defaultIntent, options: nil),
          let components = rgb.components,
          components.count >= 3
    else { return "" }
    let red = Int(components[0] * 255)
    let green = Int(components[1] * 255)
    let blue = Int(components[2] * 255)
    return String(format: "#%02x%02x%02x", red, green, blue)
  }
}
