// Kanso — KansoPushPlugin.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Flutter
import UIKit

/// Asks the OS for the APNs token and passes it to Dart. Silent pushes are
/// not received here but by the app delegate, through ``KansoPush/receive(_:)``:
/// a push may launch the app in the background before any Flutter engine,
/// and so before this plugin, exists. No notification permission is asked:
/// a silent push shows nothing, so it needs none.
public final class KansoPushPlugin: NSObject, FlutterPlugin, KansoPushHostApi {
    private var flutterApi: KansoPushFlutterApi?

    public static func register(with registrar: FlutterPluginRegistrar) {
        let plugin = KansoPushPlugin()
        plugin.flutterApi = KansoPushFlutterApi(binaryMessenger: registrar.messenger())
        KansoPushHostApiSetup.setUp(binaryMessenger: registrar.messenger(), api: plugin)
        registrar.addApplicationDelegate(plugin)
    }

    // MARK: Dart to native

    func register() throws {
        // The OS answers through the application delegate's callbacks below.
        UIApplication.shared.registerForRemoteNotifications()
    }

    func unavailableReason() throws -> String? {
        nil
    }

    // MARK: Application delegate

    public func application(
        _ application: UIApplication,
        didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
    ) {
        let token = deviceToken.map { String(format: "%02x", $0) }.joined()
        #if DEBUG
            let environment = "development"
        #else
            let environment = "production"
        #endif
        guard let api = flutterApi else {
            return
        }
        Task { @MainActor in
            // Push is a convenience; a failed call must never crash the app.
            try? await api.onToken(platform: "apns", token: token, environment: environment)
        }
    }

    public func application(
        _ application: UIApplication,
        didFailToRegisterForRemoteNotificationsWithError error: Error
    ) {
        // Nothing to do: the next launch asks again. The error is not logged
        // with anything that identifies the phone.
        NSLog("Kanso: the OS did not issue a push token (%ld).", (error as NSError).code)
    }
}

/// A completion handler that runs at most once, from any thread.
final class OnceAnswer: @unchecked Sendable {
    private let lock = NSLock()
    private var handler: ((UIBackgroundFetchResult) -> Void)?

    init(_ handler: @escaping (UIBackgroundFetchResult) -> Void) {
        self.handler = handler
    }

    func send(_ result: UIBackgroundFetchResult) {
        lock.lock()
        let pending = handler
        handler = nil
        lock.unlock()
        if let pending {
            DispatchQueue.main.async {
                pending(result)
            }
        }
    }
}
