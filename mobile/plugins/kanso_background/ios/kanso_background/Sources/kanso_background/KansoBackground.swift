import BackgroundTasks
import Foundation
import UIKit

/// The `BGAppRefreshTask` that syncs with no app.
///
/// `AppDelegate` calls [register] before `didFinishLaunching` returns, as
/// BackgroundTasks requires. The identifier is listed in the app's
/// `BGTaskSchedulerPermittedIdentifiers`, and `fetch` in `UIBackgroundModes`.
public final class KansoBackground {
  /// The task's identifier.
  public static let taskIdentifier = "app.kanso.Kanso.sync"

  /// How soon the next refresh may begin, in seconds.
  static let interval: TimeInterval = 15 * 60

  private static var registered = false

  /// Registers the handler and asks for the first refresh. Call it once, from
  /// `application(_:didFinishLaunchingWithOptions:)`.
  public static func register() {
    guard !registered else { return }
    registered = true
    BGTaskScheduler.shared.register(forTaskWithIdentifier: taskIdentifier, using: nil) { task in
      guard let task = task as? BGAppRefreshTask else {
        task.setTaskCompleted(success: false)
        return
      }
      handle(task)
    }
    NotificationCenter.default.addObserver(
      forName: UIScene.didEnterBackgroundNotification, object: nil, queue: .main
    ) { _ in schedule() }
    schedule()
  }

  /// Asks the system for the next refresh, `interval` from now at the
  /// earliest. A device or simulator that refuses is left alone: the request
  /// is made again at the next chance.
  public static func schedule() {
    let request = BGAppRefreshTaskRequest(identifier: taskIdentifier)
    request.earliestBeginDate = Date(timeIntervalSinceNow: interval)
    try? BGTaskScheduler.shared.submit(request)
  }

  private static func handle(_ task: BGAppRefreshTask) {
    let completion = Completion(task)
    task.expirationHandler = {
      // The Rust side stops within its own budget; the system only needs to
      // hear that we are done.
      completion.finish(success: false)
      schedule()
    }
    Task {
      let code = await KansoBackgroundSync.run(budget: KansoBackgroundSync.refreshBudget)
      completion.finish(success: code >= 0)
      schedule()
    }
  }

  /// Reports a task completed once: a second `setTaskCompleted` is an error.
  private final class Completion {
    private let task: BGTask
    private let lock = NSLock()
    private var done = false

    init(_ task: BGTask) { self.task = task }

    func finish(success: Bool) {
      lock.lock()
      defer { lock.unlock() }
      guard !done else { return }
      done = true
      task.setTaskCompleted(success: success)
    }
  }
}
