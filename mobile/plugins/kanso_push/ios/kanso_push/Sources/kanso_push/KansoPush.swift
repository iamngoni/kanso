// Kanso — KansoPush.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import UIKit

/// The hooks the app sets so a push can start its work.
public enum KansoPush {
    /// How long a silent push may keep the app awake before it is told it
    /// failed. The OS gives about 30 seconds; this stays inside that.
    public static let wakeLimit: TimeInterval = 25

    /// Called, on the main thread, for each silent push, which carries
    /// nothing and only says "sync". The app calls `done` once with the
    /// result when it has synced; the plugin calls it itself after
    /// ``wakeLimit`` seconds if the app has not. Left nil, a push is
    /// answered with `.noData`.
    public static var onWake: ((_ done: @escaping (UIBackgroundFetchResult) -> Void) -> Void)?

    /// Answers one silent push: the app delegate's
    /// `application(_:didReceiveRemoteNotification:fetchCompletionHandler:)`
    /// calls this, so it works with no Flutter engine running. Calls
    /// ``onWake``, and `completion` exactly once, within ``wakeLimit``.
    public static func receive(_ completion: @escaping (UIBackgroundFetchResult) -> Void) {
        guard let wake = onWake else {
            completion(.noData)
            return
        }
        let answer = OnceAnswer(completion)
        // Whatever the app does, the OS is answered in time.
        DispatchQueue.main.asyncAfter(deadline: .now() + wakeLimit) {
            answer.send(.failed)
        }
        wake { result in
            answer.send(result)
        }
    }
}
