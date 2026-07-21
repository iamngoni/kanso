import SwiftUI
import AppKit
import WebKit

struct MarkdownPreviewView: View {
    let html: String
    let markdown: String
    let attachments: [AttachmentDto]
    var onOpenURL: ((URL) -> Void)?
    var sketchPreview: ((String) -> Data?)?

    init(
        html: String,
        markdown: String = "",
        attachments: [AttachmentDto] = [],
        onOpenURL: ((URL) -> Void)? = nil,
        sketchPreview: ((String) -> Data?)? = nil
    ) {
        self.html = html
        self.markdown = markdown
        self.attachments = attachments
        self.onOpenURL = onOpenURL
        self.sketchPreview = sketchPreview
    }

    var body: some View {
        EngineMarkdownWebView(html: renderedDocumentHTML, onOpenURL: onOpenURL)
            .background(Theme.editor)
    }

    private var renderedDocumentHTML: String {
        Self.wrapEngineHTML(html.trimmingCharacters(in: .whitespacesAndNewlines))
    }

    static func wrapEngineHTML(_ bodyHTML: String) -> String {
        let content = bodyHTML.isEmpty ? "<p>Preview unavailable.</p>" : bodyHTML
        return """
        <!doctype html>
        <html>
        <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <style>
        :root {
          color-scheme: light dark;
          --bg: #FBF9F4;
          --surface: #FFFFFF;
          --surface-2: rgba(36, 35, 31, 0.045);
          --text: #24231F;
          --secondary: #5F5B52;
          --muted: #8D877C;
          --divider: #D8D0C4;
          --accent: #4F6F95;
          --code: rgba(36, 35, 31, 0.075);
        }
        @media (prefers-color-scheme: dark) {
          :root {
            --bg: #1B1C1F;
            --surface: #26282C;
            --surface-2: rgba(255, 255, 255, 0.045);
            --text: #F2F0E8;
            --secondary: #C6C1B7;
            --muted: #89857B;
            --divider: #383A3E;
            --accent: #8BA6CF;
            --code: rgba(0, 0, 0, 0.28);
          }
        }
        * { box-sizing: border-box; }
        html, body {
          margin: 0;
          min-height: 100%;
          background: var(--bg);
          color: var(--text);
          font: 16px/1.58 -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
          letter-spacing: 0;
        }
        body {
          padding: 30px;
          overflow-wrap: anywhere;
        }
        h1, h2, h3, h4, h5, h6 {
          margin: 1.25em 0 0.48em;
          line-height: 1.15;
          font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", system-ui, sans-serif;
          font-weight: 650;
        }
        h1:first-child, h2:first-child, h3:first-child { margin-top: 0; }
        h1 { font-size: 30px; }
        h2 { font-size: 23px; }
        h3 { font-size: 19px; }
        p { margin: 0 0 0.9em; }
        a { color: var(--accent); text-decoration: none; }
        a:hover { text-decoration: underline; }
        ul, ol { margin: 0.4em 0 1em; padding-left: 1.45em; }
        li { margin: 0.22em 0; }
        li.task-list-item { list-style: none; margin-left: -1.3em; }
        input[type="checkbox"] {
          width: 14px;
          height: 14px;
          margin: 0 8px 0 0;
          accent-color: var(--accent);
          vertical-align: -2px;
        }
        blockquote {
          margin: 1em 0;
          padding: 0.05em 0 0.05em 14px;
          border-left: 3px solid var(--accent);
          color: var(--secondary);
        }
        code {
          padding: 0.12em 0.34em;
          border-radius: 5px;
          background: var(--code);
          color: var(--text);
          font: 0.88em/1.45 "SF Mono", Menlo, Consolas, monospace;
        }
        pre {
          margin: 1em 0;
          padding: 13px 14px;
          overflow: auto;
          border: 1px solid var(--divider);
          border-radius: 7px;
          background: var(--code);
        }
        pre code { padding: 0; background: transparent; }
        table {
          width: 100%;
          margin: 1em 0;
          border-collapse: collapse;
          border: 1px solid var(--divider);
          border-radius: 7px;
          overflow: hidden;
        }
        th, td {
          padding: 8px 10px;
          border: 1px solid var(--divider);
          text-align: left;
          vertical-align: top;
        }
        th { background: var(--surface); font-weight: 650; }
        tr:nth-child(even) td { background: var(--surface-2); }
        hr {
          height: 1px;
          margin: 1.3em 0;
          border: 0;
          background: var(--divider);
        }
        img {
          max-width: 100%;
          height: auto;
          border-radius: 7px;
        }
        .kanso-block-link {
          display: block;
          color: inherit;
          text-decoration: none;
        }
        .kanso-block-link:hover .kanso-block {
          border-color: rgba(126, 150, 179, 0.72);
        }
        .kanso-block {
          margin: 1em 0;
          padding: 12px;
          border: 1px solid var(--divider);
          border-radius: 7px;
          background: var(--surface);
        }
        .kanso-block-icon {
          display: inline-block;
          margin-bottom: 9px;
          padding: 3px 8px;
          border-radius: 999px;
          background: rgba(126, 150, 179, 0.16);
          color: var(--accent);
          font-size: 12px;
          font-weight: 650;
        }
        .kanso-block figcaption {
          margin-top: 8px;
          color: var(--secondary);
          font-size: 13px;
        }
        .kanso-attachment-meta {
          margin-top: 3px;
          color: var(--muted);
          font-size: 12px;
        }
        </style>
        </head>
        <body>
        \(content)
        </body>
        </html>
        """
    }

    @ViewBuilder
    private var nativeFallbackPreview: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                ForEach(PreviewBlock.parse(markdown.isEmpty ? fallbackPlainText : markdown)) { block in
                    blockView(block)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 28)
            .padding(.vertical, 24)
        }
        .background(Theme.editor)
        .environment(\.openURL, OpenURLAction { url in
            onOpenURL?(url)
            return .handled
        })
    }

    @ViewBuilder
    private func blockView(_ block: PreviewBlock) -> some View {
        switch block.kind {
        case .frontMatter(let lines):
            frontMatterView(lines)
        case .heading(let level, let text):
            InlineMarkdownText(text)
                .font(headingFont(level))
                .foregroundStyle(Theme.textPrimary)
                .padding(.top, level == 1 ? 0 : 8)
        case .paragraph(let text):
            InlineMarkdownText(text)
                .font(.system(size: 16))
                .foregroundStyle(Theme.textPrimary)
                .lineSpacing(4)
        case .task(let done, let text):
            HStack(alignment: .firstTextBaseline, spacing: 9) {
                Image(systemName: done ? "checkmark.circle.fill" : "circle")
                    .font(.system(size: 13))
                    .foregroundStyle(done ? Theme.accent : Theme.textMuted)
                InlineMarkdownText(text)
                    .font(.system(size: 16))
                    .foregroundStyle(Theme.textPrimary)
            }
        case .bullet(let text):
            HStack(alignment: .firstTextBaseline, spacing: 9) {
                Text("•")
                    .font(.system(size: 16))
                    .foregroundStyle(Theme.textSecondary)
                InlineMarkdownText(text)
                    .font(.system(size: 16))
                    .foregroundStyle(Theme.textPrimary)
            }
        case .ordered(let marker, let text):
            HStack(alignment: .firstTextBaseline, spacing: 9) {
                Text(marker)
                    .font(.system(size: 16))
                    .foregroundStyle(Theme.textSecondary)
                    .frame(minWidth: 24, alignment: .trailing)
                InlineMarkdownText(text)
                    .font(.system(size: 16))
                    .foregroundStyle(Theme.textPrimary)
            }
        case .quote(let text):
            HStack(alignment: .top, spacing: 12) {
                Rectangle()
                    .fill(Theme.accent)
                    .frame(width: 3)
                InlineMarkdownText(text)
                    .font(.system(size: 16))
                    .foregroundStyle(Theme.textSecondary)
                    .lineSpacing(4)
            }
        case .callout(let type, let title, let body, let folded):
            calloutView(type: type, title: title, body: body, folded: folded)
        case .code(let code):
            ScrollView(.horizontal, showsIndicators: false) {
                Text(code)
                    .font(.system(size: 13, design: .monospaced))
                    .foregroundStyle(Theme.textPrimary)
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .background(
                RoundedRectangle(cornerRadius: 7)
                    .fill(Color.black.opacity(0.22))
                    .overlay(RoundedRectangle(cornerRadius: 7).stroke(Theme.divider, lineWidth: 1))
            )
        case .table(let rows):
            tableView(rows)
        case .image(let alt, let source):
            imageEmbed(alt: alt, source: source)
        case .embed(let kind, let target):
            embedView(kind: kind, target: target)
        case .divider:
            Rectangle()
                .fill(Theme.divider)
                .frame(height: 1)
                .padding(.vertical, 6)
        }
    }

    private func frontMatterView(_ lines: [String]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Label("Properties", systemImage: "slider.horizontal.3")
                .font(.system(size: 12, weight: .semibold))
                .foregroundStyle(Theme.textSecondary)
            if lines.isEmpty {
                Text("No properties")
                    .font(.system(size: 12))
                    .foregroundStyle(Theme.textMuted)
            } else {
                VStack(alignment: .leading, spacing: 4) {
                    ForEach(Array(lines.enumerated()), id: \.offset) { _, line in
                        Text(line)
                            .font(.system(size: 12, design: .monospaced))
                            .foregroundStyle(Theme.textMuted)
                            .lineLimit(1)
                            .truncationMode(.tail)
                    }
                }
            }
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 7)
                .fill(Theme.elevated.opacity(0.7))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 7)
                .stroke(Theme.divider, lineWidth: 1)
        )
    }

    private func calloutView(type: String, title: String, body: String, folded: Bool?) -> AnyView {
        let style = calloutStyle(for: type)
        let nestedBlocks = PreviewBlock.parse(body)

        return AnyView(VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                Image(systemName: style.icon)
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundStyle(style.tint)
                    .frame(width: 18)
                InlineMarkdownText(title)
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundStyle(Theme.textPrimary)
                Spacer(minLength: 0)
                if folded != nil {
                    Image(systemName: "chevron.down")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(Theme.textMuted)
                }
            }

            if !nestedBlocks.isEmpty {
                VStack(alignment: .leading, spacing: 10) {
                    ForEach(nestedBlocks) { block in
                        AnyView(blockView(block))
                    }
                }
                .padding(.leading, 26)
            }
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 7)
                .fill(style.tint.opacity(0.12))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 7)
                .stroke(style.tint.opacity(0.38), lineWidth: 1)
        ))
    }

    private func calloutStyle(for type: String) -> (icon: String, tint: Color) {
        switch type.localizedLowercase {
        case "abstract", "summary", "tldr":
            return ("doc.text", .teal)
        case "info":
            return ("info.circle", .blue)
        case "todo":
            return ("checklist", Theme.accent)
        case "tip", "hint", "important":
            return ("lightbulb", .yellow)
        case "success", "check", "done":
            return ("checkmark.circle", .green)
        case "question", "help", "faq":
            return ("questionmark.circle", .purple)
        case "warning", "caution", "attention":
            return ("exclamationmark.triangle", .orange)
        case "failure", "fail", "missing", "danger", "error":
            return ("xmark.octagon", .red)
        case "bug":
            return ("ladybug", .pink)
        case "example":
            return ("list.bullet.rectangle", .indigo)
        case "quote", "cite":
            return ("quote.bubble", Theme.textMuted)
        default:
            return ("note.text", Theme.accent)
        }
    }

    @ViewBuilder
    private func embedView(kind: PreviewBlock.EmbedKind, target: String) -> some View {
        switch kind {
        case .attachment:
            attachmentEmbed(target)
        case .sketch:
            sketchEmbed(target)
        case .unknown:
            fileBlock(icon: "square.dashed", title: target, subtitle: "Embedded reference")
        }
    }

    @ViewBuilder
    private func attachmentEmbed(_ target: String) -> some View {
        if let attachment = attachment(matching: target) {
            attachmentCard(attachment: attachment, target: target)
        } else {
            fileBlock(icon: "paperclip", title: target, subtitle: "Attachment not available locally")
        }
    }

    @ViewBuilder
    private func imageEmbed(alt: String, source: String) -> some View {
        if let attachment = attachment(matching: source) {
            attachmentCard(attachment: attachment, target: source)
        } else if let image = localImage(from: source) {
            VStack(alignment: .leading, spacing: 8) {
                Image(nsImage: image)
                    .resizable()
                    .scaledToFit()
                    .frame(maxHeight: 360)
                    .clipShape(RoundedRectangle(cornerRadius: 7))
                    .overlay(RoundedRectangle(cornerRadius: 7).stroke(Theme.divider, lineWidth: 1))
                if !alt.isEmpty {
                    Text(alt)
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.textMuted)
                }
            }
        } else {
            Button {
                openExternalImageSource(source)
            } label: {
                fileBlockContent(
                    icon: "photo",
                    title: alt.isEmpty ? displayName(forPath: source) : alt,
                    subtitle: source
                )
                .padding(12)
                .background(Theme.elevated)
                .clipShape(RoundedRectangle(cornerRadius: 7))
                .overlay(RoundedRectangle(cornerRadius: 7).stroke(Theme.divider, lineWidth: 1))
            }
            .buttonStyle(.plain)
            .disabled(URL(string: source)?.scheme == nil)
            .help("Open image source")
        }
    }

    private func attachmentCard(attachment: AttachmentDto, target: String) -> some View {
        Button {
            openKansoURL(kind: "attachment", target: target)
        } label: {
            VStack(alignment: .leading, spacing: 10) {
                if attachment.mimeType.hasPrefix("image/"),
                   let path = attachment.localPath,
                   let image = NSImage(contentsOf: URL(fileURLWithPath: path)) {
                    Image(nsImage: image)
                        .resizable()
                        .scaledToFit()
                        .frame(maxHeight: 320)
                        .clipShape(RoundedRectangle(cornerRadius: 7))
                        .overlay(RoundedRectangle(cornerRadius: 7).stroke(Theme.divider, lineWidth: 1))
                }
                fileBlockContent(
                    icon: attachment.mimeType.hasPrefix("image/") ? "photo" : "doc",
                    title: attachment.filename,
                    subtitle: "\(formatSize(attachment.sizeBytes)) · \(attachment.mimeType)"
                )
            }
        }
        .buttonStyle(.plain)
        .help(attachment.localPath == nil ? "File will download on sync" : "Open attachment")
    }

    @ViewBuilder
    private func sketchEmbed(_ target: String) -> some View {
        if let data = sketchPreview?(target),
           let image = NSImage(data: data) {
            Button {
                openKansoURL(kind: "sketch", target: target)
            } label: {
                VStack(alignment: .leading, spacing: 10) {
                    Image(nsImage: image)
                        .resizable()
                        .scaledToFit()
                        .frame(maxHeight: 320)
                        .padding(10)
                        .background(Theme.elevated)
                        .clipShape(RoundedRectangle(cornerRadius: 7))
                        .overlay(RoundedRectangle(cornerRadius: 7).stroke(Theme.divider, lineWidth: 1))
                    fileBlockContent(icon: "scribble", title: "Sketch", subtitle: target)
                }
            }
            .buttonStyle(.plain)
            .help("Edit sketch")
        } else {
            Button {
                openKansoURL(kind: "sketch", target: target)
            } label: {
                fileBlock(icon: "scribble", title: "Sketch", subtitle: target)
            }
            .buttonStyle(.plain)
            .help("Edit sketch")
        }
    }

    private func fileBlock(icon: String, title: String, subtitle: String) -> some View {
        fileBlockContent(icon: icon, title: title, subtitle: subtitle)
            .padding(12)
            .background(Theme.elevated)
            .clipShape(RoundedRectangle(cornerRadius: 7))
            .overlay(RoundedRectangle(cornerRadius: 7).stroke(Theme.divider, lineWidth: 1))
    }

    private func fileBlockContent(icon: String, title: String, subtitle: String) -> some View {
        HStack(spacing: 10) {
            Image(systemName: icon)
                .font(.system(size: 18))
                .foregroundStyle(Theme.accent)
                .frame(width: 26)
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundStyle(Theme.textPrimary)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Text(subtitle)
                    .font(.system(size: 12))
                    .foregroundStyle(Theme.textMuted)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            Spacer(minLength: 0)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func attachment(matching target: String) -> AttachmentDto? {
        let canonical = target
            .removingPercentEncoding?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            ?? target.trimmingCharacters(in: .whitespacesAndNewlines)
        let suffix = canonical.hasPrefix("attachment:")
            ? String(canonical.dropFirst("attachment:".count))
            : canonical
        let basename = displayName(forPath: suffix)
        return attachments.first { attachment in
            attachment.id == canonical
                || attachment.id == "attachment:\(suffix)"
                || attachment.filename == canonical
                || attachment.filename == basename
                || attachment.contentHash == canonical
                || attachment.localPath == canonical
                || attachment.localPath == suffix
        }
    }

    private func localImage(from source: String) -> NSImage? {
        let decoded = source.removingPercentEncoding ?? source
        if let url = URL(string: decoded), url.isFileURL {
            return NSImage(contentsOf: url)
        }
        guard decoded.hasPrefix("/") else { return nil }
        return NSImage(contentsOf: URL(fileURLWithPath: decoded))
    }

    private func openExternalImageSource(_ source: String) {
        guard let url = URL(string: source), url.scheme != nil else { return }
        onOpenURL?(url)
    }

    private func displayName(forPath path: String) -> String {
        if let url = URL(string: path),
           let host = url.host,
           !host.isEmpty {
            return url.lastPathComponent.isEmpty ? host : url.lastPathComponent
        }
        return URL(fileURLWithPath: path).lastPathComponent
    }

    private func openKansoURL(kind: String, target: String) {
        guard let encoded = target.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed),
              let url = URL(string: "kanso://\(kind)/\(encoded)") else { return }
        onOpenURL?(url)
    }

    private func formatSize(_ bytes: Int64) -> String {
        let units = ["B", "KB", "MB", "GB"]
        var value = Double(max(bytes, 0))
        var unit = units[0]
        for next in units.dropFirst() {
            if value < 1024 { break }
            value /= 1024
            unit = next
        }
        return unit == "B" ? "\(Int(value)) \(unit)" : String(format: "%.1f %@", value, unit)
    }

    private func tableView(_ rows: [[String]]) -> some View {
        Grid(horizontalSpacing: 0, verticalSpacing: 0) {
            ForEach(Array(rows.enumerated()), id: \.offset) { rowIndex, row in
                GridRow {
                    ForEach(Array(row.enumerated()), id: \.offset) { _, cell in
                        InlineMarkdownText(cell)
                            .font(.system(size: 14, weight: rowIndex == 0 ? .semibold : .regular))
                            .foregroundStyle(Theme.textPrimary)
                            .padding(.horizontal, 9)
                            .padding(.vertical, 7)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(rowIndex == 0 ? Theme.elevated : Theme.editor)
                            .overlay(Rectangle().stroke(Theme.divider, lineWidth: 1))
                    }
                }
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: 7))
    }

    private func headingFont(_ level: Int) -> Font {
        switch level {
        case 1: .system(size: 28, weight: .semibold)
        case 2: .system(size: 22, weight: .semibold)
        case 3: .system(size: 18, weight: .semibold)
        default: .system(size: 16, weight: .semibold)
        }
    }

    private var fallbackPlainText: String {
        html
            .replacingOccurrences(of: "<br>", with: "\n")
            .replacingOccurrences(of: "<br/>", with: "\n")
            .replacingOccurrences(of: "<br />", with: "\n")
            .replacingOccurrences(of: "</p>", with: "\n\n")
            .replacingOccurrences(of: "</h1>", with: "\n\n")
            .replacingOccurrences(of: "</h2>", with: "\n\n")
            .replacingOccurrences(of: "</h3>", with: "\n\n")
            .replacingOccurrences(of: "<[^>]+>", with: "", options: .regularExpression)
            .trimmingCharacters(in: .whitespacesAndNewlines)
    }
}

private struct EngineMarkdownWebView: NSViewRepresentable {
    let html: String
    var onOpenURL: ((URL) -> Void)?

    func makeCoordinator() -> Coordinator {
        Coordinator(onOpenURL: onOpenURL)
    }

    func makeNSView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.defaultWebpagePreferences.allowsContentJavaScript = false

        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = context.coordinator
        webView.setValue(false, forKey: "drawsBackground")
        webView.allowsBackForwardNavigationGestures = false
        webView.configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        context.coordinator.onOpenURL = onOpenURL
        guard context.coordinator.loadedHTML != html else { return }
        context.coordinator.loadedHTML = html
        webView.loadHTMLString(html, baseURL: nil)
    }

    final class Coordinator: NSObject, WKNavigationDelegate {
        var loadedHTML: String?
        var onOpenURL: ((URL) -> Void)?

        init(onOpenURL: ((URL) -> Void)?) {
            self.onOpenURL = onOpenURL
        }

        func webView(
            _ webView: WKWebView,
            decidePolicyFor navigationAction: WKNavigationAction,
            decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
        ) {
            guard navigationAction.navigationType == .linkActivated,
                  let url = navigationAction.request.url else {
                decisionHandler(.allow)
                return
            }

            let scheme = url.scheme?.localizedLowercase
            if scheme == "about" || scheme == "data" {
                decisionHandler(.allow)
                return
            }

            onOpenURL?(url)
            decisionHandler(.cancel)
        }
    }
}

struct InlineMarkdownText: View {
    let text: String

    init(_ text: String) {
        self.text = text
    }

    var body: some View {
        Text(attributedText)
    }

    private var attributedText: AttributedString {
        Self.attributedString(for: text)
    }

    static func attributedString(for text: String) -> AttributedString {
        let rewritten = rewriteWikiLinks(in: text)
        return (try? AttributedString(markdown: rewritten)) ?? AttributedString(text)
    }

    static func rewriteWikiLinks(in text: String) -> String {
        var output = ""
        var remainder = text[...]

        while let open = remainder.range(of: "[["),
              let close = remainder[open.upperBound...].range(of: "]]") {
            output += remainder[..<open.lowerBound]
            let rawTarget = String(remainder[open.upperBound..<close.lowerBound])
            let parts = rawTarget.split(separator: "|", maxSplits: 1, omittingEmptySubsequences: false)
            let target = String(parts.first ?? "")
                .trimmingCharacters(in: .whitespacesAndNewlines)
            let label = String(parts.dropFirst().first ?? parts.first ?? "")
                .trimmingCharacters(in: .whitespacesAndNewlines)

            if let encoded = target.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed),
               !target.isEmpty {
                output += "[\(label.isEmpty ? target : label)](kanso://note/\(encoded))"
            } else {
                output += "[[\(rawTarget)]]"
            }

            remainder = remainder[close.upperBound...]
        }

        output += remainder
        return output
    }
}

struct PreviewBlock: Identifiable {
    enum EmbedKind {
        case attachment
        case sketch
        case unknown
    }

    enum Kind {
        case frontMatter([String])
        case heading(level: Int, text: String)
        case paragraph(String)
        case task(done: Bool, text: String)
        case bullet(String)
        case ordered(marker: String, text: String)
        case quote(String)
        case callout(type: String, title: String, body: String, folded: Bool?)
        case code(String)
        case table([[String]])
        case image(alt: String, source: String)
        case embed(kind: EmbedKind, target: String)
        case divider
    }

    let id = UUID()
    let kind: Kind

    static func parse(_ markdown: String) -> [PreviewBlock] {
        let lines = markdown.components(separatedBy: .newlines)
        var blocks: [PreviewBlock] = []
        var index = 0

        while index < lines.count {
            let line = lines[index]
            let trimmed = line.trimmingCharacters(in: .whitespaces)

            if trimmed.isEmpty {
                index += 1
                continue
            }

            if index == 0,
               trimmed == "---",
               let frontMatter = parseFrontMatter(lines, startingAt: index) {
                blocks.append(PreviewBlock(kind: .frontMatter(frontMatter.lines)))
                index = frontMatter.nextIndex
                continue
            }

            if trimmed.hasPrefix("```") {
                let start = index + 1
                index += 1
                while index < lines.count,
                      !lines[index].trimmingCharacters(in: .whitespaces).hasPrefix("```") {
                    index += 1
                }
                let code = lines[start..<min(index, lines.count)].joined(separator: "\n")
                blocks.append(PreviewBlock(kind: .code(code)))
                index += 1
                continue
            }

            if index + 1 < lines.count,
               isTableRow(line),
               isTableSeparator(lines[index + 1]) {
                var rows = [splitTableRow(line)]
                index += 2
                while index < lines.count, isTableRow(lines[index]) {
                    rows.append(splitTableRow(lines[index]))
                    index += 1
                }
                blocks.append(PreviewBlock(kind: .table(rows)))
                continue
            }

            if index + 1 < lines.count,
               !startsBlock(trimmed),
               let setextLevel = parseSetextUnderline(lines[index + 1].trimmingCharacters(in: .whitespaces)) {
                blocks.append(PreviewBlock(kind: .heading(level: setextLevel, text: trimmed)))
                index += 2
                continue
            }

            if trimmed == "---" || trimmed == "***" {
                blocks.append(PreviewBlock(kind: .divider))
                index += 1
                continue
            }

            if let heading = parseHeading(trimmed) {
                blocks.append(PreviewBlock(kind: .heading(level: heading.level, text: heading.text)))
                index += 1
                continue
            }

            if let embed = parseEmbed(trimmed) {
                blocks.append(PreviewBlock(kind: .embed(kind: embed.kind, target: embed.target)))
                index += 1
                continue
            }

            if let image = parseImage(trimmed) {
                blocks.append(PreviewBlock(kind: .image(alt: image.alt, source: image.source)))
                index += 1
                continue
            }

            if let task = parseTask(trimmed) {
                blocks.append(PreviewBlock(kind: .task(done: task.done, text: task.text)))
                index += 1
                continue
            }

            if trimmed.hasPrefix("- ") || trimmed.hasPrefix("* ") {
                blocks.append(PreviewBlock(kind: .bullet(String(trimmed.dropFirst(2)))))
                index += 1
                continue
            }

            if let ordered = parseOrderedList(trimmed) {
                blocks.append(PreviewBlock(kind: .ordered(marker: ordered.marker, text: ordered.text)))
                index += 1
                continue
            }

            if trimmed.hasPrefix(">") {
                let firstQuoteLine = unquote(trimmed)
                if let callout = parseCalloutHeader(firstQuoteLine) {
                    var bodyLines: [String] = []
                    index += 1
                    while index < lines.count {
                        let current = lines[index].trimmingCharacters(in: .whitespaces)
                        guard current.hasPrefix(">") else { break }
                        bodyLines.append(unquote(current))
                        index += 1
                    }
                    blocks.append(PreviewBlock(kind: .callout(
                        type: callout.type,
                        title: callout.title,
                        body: bodyLines.joined(separator: "\n"),
                        folded: callout.folded
                    )))
                    continue
                }

                var quoteLines: [String] = []
                while index < lines.count {
                    let current = lines[index].trimmingCharacters(in: .whitespaces)
                    guard current.hasPrefix(">") else { break }
                    quoteLines.append(unquote(current))
                    index += 1
                }
                blocks.append(PreviewBlock(kind: .quote(quoteLines.joined(separator: "\n"))))
                continue
            }

            var paragraphLines = [trimmed]
            index += 1
            while index < lines.count {
                let next = lines[index].trimmingCharacters(in: .whitespaces)
                if next.isEmpty || startsBlock(next) { break }
                paragraphLines.append(next)
                index += 1
            }
            blocks.append(PreviewBlock(kind: .paragraph(paragraphLines.joined(separator: " "))))
        }

        return blocks
    }

    private static func parseFrontMatter(
        _ lines: [String],
        startingAt startIndex: Int
    ) -> (lines: [String], nextIndex: Int)? {
        guard startIndex == 0,
              lines[startIndex].trimmingCharacters(in: .whitespaces) == "---" else {
            return nil
        }

        var index = startIndex + 1
        var properties: [String] = []
        while index < lines.count {
            let trimmed = lines[index].trimmingCharacters(in: .whitespaces)
            if trimmed == "---" {
                return (properties, index + 1)
            }
            properties.append(lines[index])
            index += 1
        }
        return nil
    }

    private static func parseSetextUnderline(_ line: String) -> Int? {
        guard line.count >= 3 else { return nil }
        if line.allSatisfy({ $0 == "=" }) { return 1 }
        if line.allSatisfy({ $0 == "-" }) { return 2 }
        return nil
    }

    private static func startsBlock(_ line: String) -> Bool {
        line.hasPrefix("#")
            || line.hasPrefix("```")
            || line.hasPrefix("![[")
            || parseImage(line) != nil
            || line.hasPrefix("- ")
            || line.hasPrefix("* ")
            || parseOrderedList(line) != nil
            || line.hasPrefix(">")
            || line == "---"
            || line == "***"
    }

    private static func unquote(_ line: String) -> String {
        String(line.dropFirst()).trimmingCharacters(in: .whitespaces)
    }

    private static func parseCalloutHeader(_ line: String) -> (type: String, title: String, folded: Bool?)? {
        guard line.hasPrefix("[!") else { return nil }
        guard let close = line.firstIndex(of: "]") else { return nil }

        let typeStart = line.index(line.startIndex, offsetBy: 2)
        let rawType = String(line[typeStart..<close])
            .trimmingCharacters(in: .whitespacesAndNewlines)
        guard !rawType.isEmpty else { return nil }

        var remainder = line[line.index(after: close)...]
        var folded: Bool?
        if let first = remainder.first, first == "+" || first == "-" {
            folded = first == "-"
            remainder = remainder.dropFirst()
        }

        let type = rawType.localizedLowercase
        let rawTitle = String(remainder)
            .trimmingCharacters(in: .whitespacesAndNewlines)
        return (type, rawTitle.isEmpty ? defaultCalloutTitle(for: type) : rawTitle, folded)
    }

    private static func defaultCalloutTitle(for type: String) -> String {
        type
            .split(separator: "-")
            .map { part in
                part.prefix(1).uppercased() + String(part.dropFirst())
            }
            .joined(separator: " ")
    }

    private static func parseHeading(_ line: String) -> (level: Int, text: String)? {
        let hashes = line.prefix { $0 == "#" }.count
        guard (1...6).contains(hashes),
              line.dropFirst(hashes).first == " " else { return nil }
        return (
            hashes,
            String(line.dropFirst(hashes + 1)).trimmingCharacters(in: .whitespaces)
        )
    }

    private static func parseEmbed(_ line: String) -> (kind: EmbedKind, target: String)? {
        guard line.hasPrefix("![["),
              line.hasSuffix("]]") else { return nil }
        let target = String(line.dropFirst(3).dropLast(2))
            .trimmingCharacters(in: .whitespacesAndNewlines)
        guard !target.isEmpty else { return nil }

        let lower = target.localizedLowercase
        if lower.hasPrefix("attachment:") {
            return (.attachment, target)
        }
        if lower.hasPrefix("sketch:") {
            return (.sketch, target)
        }
        return (.unknown, target)
    }

    private static func parseTask(_ line: String) -> (done: Bool, text: String)? {
        if line.hasPrefix("- [ ] ") {
            return (false, String(line.dropFirst(6)))
        }
        if line.hasPrefix("- [x] ") || line.hasPrefix("- [X] ") {
            return (true, String(line.dropFirst(6)))
        }
        return nil
    }

    private static func parseOrderedList(_ line: String) -> (marker: String, text: String)? {
        let digits = line.prefix { $0.isNumber }
        guard !digits.isEmpty else { return nil }
        let markerIndex = line.index(line.startIndex, offsetBy: digits.count)
        guard markerIndex < line.endIndex,
              line[markerIndex] == "." || line[markerIndex] == ")" else { return nil }
        let afterMarker = line.index(after: markerIndex)
        guard afterMarker < line.endIndex,
              line[afterMarker].isWhitespace else { return nil }
        let text = line[afterMarker...].trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return nil }
        return ("\(digits).", text)
    }

    private static func parseImage(_ line: String) -> (alt: String, source: String)? {
        guard line.hasPrefix("!["),
              let separator = line.range(of: "]("),
              line.hasSuffix(")") else { return nil }

        let altStart = line.index(line.startIndex, offsetBy: 2)
        let alt = String(line[altStart..<separator.lowerBound])
        var source = String(line[separator.upperBound..<line.index(before: line.endIndex)])
            .trimmingCharacters(in: .whitespacesAndNewlines)

        if source.hasPrefix("<"), source.hasSuffix(">") {
            source = String(source.dropFirst().dropLast())
        }

        source = source.trimmingCharacters(in: CharacterSet(charactersIn: "\"'"))
        guard !source.isEmpty else { return nil }
        return (alt, source)
    }

    private static func isTableRow(_ line: String) -> Bool {
        line.contains("|") && splitTableRow(line).count >= 2
    }

    private static func isTableSeparator(_ line: String) -> Bool {
        let cells = splitTableRow(line)
        guard cells.count >= 2 else { return false }
        return cells.allSatisfy { cell in
            let trimmed = cell.trimmingCharacters(in: .whitespaces)
            guard trimmed.count >= 3 else { return false }
            return trimmed.allSatisfy { $0 == "-" || $0 == ":" }
        }
    }

    private static func splitTableRow(_ line: String) -> [String] {
        var trimmed = line.trimmingCharacters(in: .whitespaces)
        if trimmed.first == "|" { trimmed.removeFirst() }
        if trimmed.last == "|" { trimmed.removeLast() }
        return trimmed
            .split(separator: "|", omittingEmptySubsequences: false)
            .map { $0.trimmingCharacters(in: .whitespaces) }
    }
}
