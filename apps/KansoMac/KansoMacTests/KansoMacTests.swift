import XCTest
@testable import KansoMac

final class KansoMacTests: XCTestCase {
    func testMarkdownPreviewParserRecognizesCoreNoteBlocks() {
        let blocks = PreviewBlock.parse(
            """
            # Heading

            This has [[Target|a link]].

            1. First ordered
            2. Second ordered

            - [ ] Open task
            - [x] Done task

            ![Diagram](attachments/hash/diagram.png)

            ![[attachment:file-id]]

            ![[sketch:sync-flow]]

            | Name | Value |
            | --- | --- |
            | Mode | Split |
            """
        )

        XCTAssertTrue(blocks.containsHeading(level: 1, text: "Heading"))
        XCTAssertTrue(blocks.containsOrdered(marker: "1.", text: "First ordered"))
        XCTAssertTrue(blocks.containsOrdered(marker: "2.", text: "Second ordered"))
        XCTAssertTrue(blocks.containsTask(done: false, text: "Open task"))
        XCTAssertTrue(blocks.containsTask(done: true, text: "Done task"))
        XCTAssertTrue(blocks.containsImage(alt: "Diagram", source: "attachments/hash/diagram.png"))
        XCTAssertTrue(blocks.containsAttachmentEmbed("attachment:file-id"))
        XCTAssertTrue(blocks.containsSketchEmbed("sketch:sync-flow"))
        XCTAssertTrue(blocks.containsTableCell("Split"))
    }

    func testMarkdownPreviewParserKeepsParagraphsSeparateFromFollowingBlocks() {
        let blocks = PreviewBlock.parse(
            """
            Paragraph one
            still paragraph one.
            1. Ordered after paragraph
            ![Image](/tmp/image.png)
            """
        )

        guard blocks.count == 3 else {
            XCTFail("Expected paragraph, ordered list item, and image; got \(blocks.count) blocks")
            return
        }

        if case .paragraph(let text) = blocks[0].kind {
            XCTAssertEqual(text, "Paragraph one still paragraph one.")
        } else {
            XCTFail("First block should be a paragraph")
        }
        XCTAssertTrue(blocks.containsOrdered(marker: "1.", text: "Ordered after paragraph"))
        XCTAssertTrue(blocks.containsImage(alt: "Image", source: "/tmp/image.png"))
    }

    func testMarkdownPreviewParserRecognizesObsidianCallouts() {
        let blocks = PreviewBlock.parse(
            """
            > [!warning]- Pay attention
            > Body has [[Target|a link]].
            >
            > - [ ] Follow up
            """
        )

        guard blocks.count == 1,
              case .callout(let type, let title, let body, let folded) = blocks[0].kind else {
            XCTFail("Expected a single callout block")
            return
        }

        XCTAssertEqual(type, "warning")
        XCTAssertEqual(title, "Pay attention")
        XCTAssertEqual(folded, true)
        XCTAssertTrue(body.contains("Body has [[Target|a link]]."))
        XCTAssertTrue(PreviewBlock.parse(body).containsTask(done: false, text: "Follow up"))
    }

    func testMarkdownPreviewParserUsesDefaultCalloutTitle() {
        let blocks = PreviewBlock.parse(
            """
            > [!tip]
            > Small useful detail.
            """
        )

        guard blocks.count == 1,
              case .callout(let type, let title, let body, let folded) = blocks[0].kind else {
            XCTFail("Expected a single callout block")
            return
        }

        XCTAssertEqual(type, "tip")
        XCTAssertEqual(title, "Tip")
        XCTAssertNil(folded)
        XCTAssertEqual(body, "Small useful detail.")
    }

    func testMarkdownPreviewParserRecognizesFrontMatterAndSetextHeadings() {
        let blocks = PreviewBlock.parse(
            """
            ---
            tags: [work, planning]
            status: active
            ---

            Product Direction
            =================

            Next Section
            ------------
            """
        )

        XCTAssertTrue(blocks.containsFrontMatterLine("tags: [work, planning]"))
        XCTAssertTrue(blocks.containsHeading(level: 1, text: "Product Direction"))
        XCTAssertTrue(blocks.containsHeading(level: 2, text: "Next Section"))
    }

    func testMarkdownPreviewWrapsEngineHtmlWithoutStrippingGfmBlocks() {
        let document = MarkdownPreviewView.wrapEngineHTML(
            """
            <h1>Heading</h1>
            <table><tbody><tr><td>Split</td></tr></tbody></table>
            <a class="kanso-block-link" href="kanso://attachment/file-id"><figure class="kanso-block">Attachment</figure></a>
            """
        )

        XCTAssertTrue(document.contains("<style>"))
        XCTAssertTrue(document.contains("<table>"))
        XCTAssertTrue(document.contains("kanso://attachment/file-id"))
        XCTAssertTrue(document.contains(".kanso-block-link"))
    }

    func testMarkdownPreviewSupportsSystemLightAndDarkAppearance() {
        let document = MarkdownPreviewView.wrapEngineHTML("<p>Body</p>")

        XCTAssertTrue(document.contains("color-scheme: light dark;"))
        XCTAssertTrue(document.contains("@media (prefers-color-scheme: dark)"))
        XCTAssertTrue(document.contains("--bg: #FBF9F4;"))
        XCTAssertTrue(document.contains("--bg: #1B1C1F;"))
    }

    func testInlineMarkdownRewritesEveryWikiLink() {
        let rewritten = InlineMarkdownText.rewriteWikiLinks(
            in: "Open [[First Note]] and [[Second Note|the second]], then keep [external](https://example.com)."
        )

        XCTAssertEqual(
            rewritten,
            "Open [First Note](kanso://note/First%20Note) and [the second](kanso://note/Second%20Note), then keep [external](https://example.com)."
        )
    }

    func testInlineMarkdownLeavesMalformedWikiTextReadable() {
        let rewritten = InlineMarkdownText.rewriteWikiLinks(
            in: "Keep [[ ]] visible and rewrite [[Valid]]."
        )

        XCTAssertEqual(
            rewritten,
            "Keep [[ ]] visible and rewrite [Valid](kanso://note/Valid)."
        )
    }

    func testExportFilenamesStayUniqueForDuplicateTitles() {
        var used = Set<String>()

        XCTAssertEqual(
            KansoStore.uniqueMarkdownFilename(for: "Meeting Notes", usedFilenames: &used),
            "Meeting Notes.md"
        )
        XCTAssertEqual(
            KansoStore.uniqueMarkdownFilename(for: "Meeting Notes", usedFilenames: &used),
            "Meeting Notes 2.md"
        )
        XCTAssertEqual(
            KansoStore.uniqueMarkdownFilename(for: "Meeting/Notes", usedFilenames: &used),
            "Meeting_Notes.md"
        )
    }

    func testSyncErrorsUsePlainRecoveryMessages() {
        let message = KansoStore.userFacingSyncError(
            KansoError.Engine(
                message: "transport error: error sending request for url (http://127.0.0.1:8787/v1/sync/pull): connection refused"
            ),
            baseURL: "http://127.0.0.1:8787"
        )

        XCTAssertEqual(
            message,
            "Can't reach sync server. Start Wrangler or check the server address."
        )
        XCTAssertFalse(message.contains("KansoMac.KansoError"))
        XCTAssertFalse(message.localizedCaseInsensitiveContains("transport error"))
    }

    func testSyncAuthErrorsAskUserToSignInAgain() {
        let message = KansoStore.userFacingSyncError(
            KansoError.Engine(message: "server returned 401 unauthorized"),
            baseURL: "https://sync.example.test"
        )

        XCTAssertEqual(message, "Sign in again to continue syncing.")
        XCTAssertFalse(message.contains("401"))
    }

    func testOperationErrorsStripGeneratedKansoWrapper() {
        let message = KansoStore.userFacingOperationError(
            KansoError.Engine(message: "note not found"),
            fallback: "Rename failed"
        )

        XCTAssertEqual(message, "Rename failed: note not found")
        XCTAssertFalse(message.contains("KansoMac.KansoError"))
        XCTAssertFalse(message.contains("Engine(message:"))
    }
}

private extension Array where Element == PreviewBlock {
    func containsHeading(level: Int, text: String) -> Bool {
        contains {
            if case .heading(let actualLevel, let actualText) = $0.kind {
                return actualLevel == level && actualText == text
            }
            return false
        }
    }

    func containsFrontMatterLine(_ expected: String) -> Bool {
        contains {
            if case .frontMatter(let lines) = $0.kind {
                return lines.contains(expected)
            }
            return false
        }
    }

    func containsOrdered(marker: String, text: String) -> Bool {
        contains {
            if case .ordered(let actualMarker, let actualText) = $0.kind {
                return actualMarker == marker && actualText == text
            }
            return false
        }
    }

    func containsTask(done: Bool, text: String) -> Bool {
        contains {
            if case .task(let actualDone, let actualText) = $0.kind {
                return actualDone == done && actualText == text
            }
            return false
        }
    }

    func containsImage(alt: String, source: String) -> Bool {
        contains {
            if case .image(let actualAlt, let actualSource) = $0.kind {
                return actualAlt == alt && actualSource == source
            }
            return false
        }
    }

    func containsAttachmentEmbed(_ target: String) -> Bool {
        contains {
            if case .embed(.attachment, let actualTarget) = $0.kind {
                return actualTarget == target
            }
            return false
        }
    }

    func containsSketchEmbed(_ target: String) -> Bool {
        contains {
            if case .embed(.sketch, let actualTarget) = $0.kind {
                return actualTarget == target
            }
            return false
        }
    }

    func containsTableCell(_ expected: String) -> Bool {
        contains {
            if case .table(let rows) = $0.kind {
                return rows.flatMap { $0 }.contains(expected)
            }
            return false
        }
    }
}
