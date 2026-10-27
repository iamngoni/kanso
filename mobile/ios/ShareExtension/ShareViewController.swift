// Kanso — ShareViewController.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import UIKit
import UniformTypeIdentifiers

/// The share sheet's Kanso: saves what was shared into the Inbox.
///
/// It reads the shared items' attachments (a link, text, images, PDFs), writes
/// them as one drop in the library's inbox, asks the library to save it, says
/// what happened in a sentence and closes. Files are copied straight into the
/// drop by URL: a share extension is allowed little memory, so nothing large
/// is ever loaded.
final class ShareViewController: UIViewController {
  private let label = UILabel()
  private let spinner = UIActivityIndicatorView(style: .medium)
  private let button = UIButton(type: .system)

  /// Set once the person cancels, so work that finishes later is dropped.
  private var cancelled = false
  private var drop: DropWriter?

  override func viewDidLoad() {
    super.viewDidLoad()
    buildInterface()
    save()
  }

  // MARK: - Interface

  private func buildInterface() {
    view.backgroundColor = .systemBackground

    label.text = "Saving to your Inbox…"
    label.font = .preferredFont(forTextStyle: .body)
    label.adjustsFontForContentSizeCategory = true
    label.textAlignment = .center
    label.numberOfLines = 0

    spinner.startAnimating()

    button.setTitle("Cancel", for: .normal)
    button.titleLabel?.font = .preferredFont(forTextStyle: .body)
    button.addTarget(self, action: #selector(buttonTapped), for: .touchUpInside)

    let stack = UIStackView(arrangedSubviews: [spinner, label, button])
    stack.axis = .vertical
    stack.alignment = .center
    stack.spacing = 16
    stack.translatesAutoresizingMaskIntoConstraints = false
    view.addSubview(stack)

    NSLayoutConstraint.activate([
      stack.centerYAnchor.constraint(equalTo: view.safeAreaLayoutGuide.centerYAnchor),
      stack.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor, constant: 16),
      stack.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor, constant: -16),
    ])
  }

  /// Shows a sentence and closes the sheet after a moment, or sooner when the
  /// person taps Done.
  private func show(_ message: String) {
    spinner.stopAnimating()
    label.text = message
    button.setTitle("Done", for: .normal)
    UIAccessibility.post(notification: .announcement, argument: message)
    DispatchQueue.main.asyncAfter(deadline: .now() + 1.8) { [weak self] in self?.complete() }
  }

  @objc private func buttonTapped() {
    if spinner.isAnimating {
      cancelled = true
      drop?.discard()
      extensionContext?.cancelRequest(
        withError: NSError(domain: NSCocoaErrorDomain, code: NSUserCancelledError))
    } else {
      complete()
    }
  }

  private var completed = false

  private func complete() {
    guard !completed, !cancelled else { return }
    completed = true
    extensionContext?.completeRequest(returningItems: nil)
  }

  // MARK: - Reading the share

  /// What the attachments held, gathered from callbacks on any thread.
  private struct Gathered {
    var subject: String?
    var text: String?
    var url: String?
  }

  private func save() {
    guard let location = KansoLibraryLocation.shared() else {
      show("Kanso couldn't open its library.")
      return
    }
    let items = (extensionContext?.inputItems as? [NSExtensionItem]) ?? []
    let writer: DropWriter
    do {
      writer = try DropWriter(inbox: location.inbox)
    } catch {
      show("Kanso couldn't save this.")
      return
    }
    drop = writer

    var gathered = Gathered()
    let lock = NSLock()
    let group = DispatchGroup()
    var skipped = 0

    for item in items {
      let title = item.attributedTitle?.string.trimmingCharacters(in: .whitespacesAndNewlines)
      let content = item.attributedContentText?.string.trimmingCharacters(in: .whitespacesAndNewlines)
      let providers = item.attachments ?? []
      let hasLink = providers.contains { $0.hasItemConformingToTypeIdentifier(UTType.url.identifier) }
      lock.lock()
      if gathered.subject == nil {
        gathered.subject = [title, hasLink ? content : nil]
          .compactMap { $0 }.first { !$0.isEmpty }
      }
      lock.unlock()

      for provider in providers {
        group.enter()
        read(provider, into: writer) { part in
          lock.lock()
          switch part {
          case .url(let value): if gathered.url == nil { gathered.url = value }
          case .text(let value): if gathered.text == nil { gathered.text = value }
          case .file: break
          case .skipped: skipped += 1
          }
          lock.unlock()
          group.leave()
        }
      }
    }

    group.notify(queue: .global(qos: .userInitiated)) { [weak self] in
      guard let self, !self.cancelled else { return }
      let nothing = gathered.url == nil && gathered.text == nil && writer.fileCount == 0
      if nothing {
        writer.discard()
        DispatchQueue.main.async { self.show("Kanso can't save this kind of item.") }
        return
      }
      do {
        try writer.finish(subject: gathered.subject, text: gathered.text, url: gathered.url)
      } catch {
        writer.discard()
        DispatchQueue.main.async { self.show("Kanso couldn't save this.") }
        return
      }
      let result = KansoCore.captureInbox(in: location)
      var message = KansoCore.summary(of: result)
      if skipped > 0 && result?.problems.isEmpty != false {
        message += " Some items couldn't be added."
      }
      DispatchQueue.main.async { self.show(message) }
    }
  }

  private enum Part {
    case url(String)
    case text(String)
    case file
    case skipped
  }

  /// Reads one attachment: images and PDFs are copied into the drop inside
  /// the provider's callback (the file is gone after it returns), links and
  /// text are read as strings.
  private func read(
    _ provider: NSItemProvider, into writer: DropWriter, done: @escaping (Part) -> Void
  ) {
    if let type = fileType(of: provider) {
      provider.loadFileRepresentation(forTypeIdentifier: type.identifier) { url, _ in
        guard let url else { return done(.skipped) }
        do {
          try writer.addFile(from: url, type: type.preferredMIMEType)
          done(.file)
        } catch {
          done(.skipped)
        }
      }
    } else if provider.hasItemConformingToTypeIdentifier(UTType.url.identifier) {
      provider.loadItem(forTypeIdentifier: UTType.url.identifier) { value, _ in
        if let url = value as? URL, !url.isFileURL {
          done(.url(url.absoluteString))
        } else if let text = value as? String, URL(string: text)?.scheme != nil {
          done(.url(text))
        } else {
          done(.skipped)
        }
      }
    } else if provider.hasItemConformingToTypeIdentifier(UTType.plainText.identifier) {
      provider.loadItem(forTypeIdentifier: UTType.plainText.identifier) { value, _ in
        let text = (value as? String) ?? (value as? URL).flatMap { try? String(contentsOf: $0) }
        let trimmed = text?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        done(trimmed.isEmpty ? .skipped : .text(trimmed))
      }
    } else {
      done(.skipped)
    }
  }

  /// The type to load a file attachment as: a PNG or JPEG if the provider has
  /// one, else any other image, else a PDF. `nil` for links and text.
  private func fileType(of provider: NSItemProvider) -> UTType? {
    let registered = provider.registeredContentTypes
    if provider.hasItemConformingToTypeIdentifier(UTType.image.identifier) {
      return registered.first { $0.conforms(to: .png) }
        ?? registered.first { $0.conforms(to: .jpeg) }
        ?? registered.first { $0.conforms(to: .image) }
        ?? .image
    }
    if provider.hasItemConformingToTypeIdentifier(UTType.pdf.identifier) {
      return registered.first { $0.conforms(to: .pdf) } ?? .pdf
    }
    return nil
  }
}
