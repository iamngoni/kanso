# Changelog

Every release of Kanso, newest first. Each entry links its commit, and each
heading compares the release with the one before.

## Unreleased

### New

- **desktop/mobile:** Choose transient or always-visible scrollbars for overflowing lists and panels ([#327](https://github.com/iamngoni/kanso/issues/327))
- **app:** `kanso update` checks for a newer Kanso and installs it from the command line, without opening the app or the library, so it works even when the app won't start. It uses the same signed feed and checks as Settings › Updates. `--check` only reports (exit code 10 when there's a newer version); `--yes` installs without asking.
- **app/mobile:** Overview brings Today and This week together as Kanso's home, with on-device system-calendar support ([#240](https://github.com/iamngoni/kanso/issues/240))

## [0.9.4](https://github.com/iamngoni/kanso/compare/v0.9.3...v0.9.4) - 2026-11-04

### New

- **plugins:** Warn three times before a plugin reads the clipboard history ([d551262](https://github.com/iamngoni/kanso/commit/d551262ec3e1676d2be44eaf3cd2861e19b68840))
- **store:** Resurface related notes from passage vectors, with persistent dismissals and On this day ([56e97b7](https://github.com/iamngoni/kanso/commit/56e97b7bc5dec31927a7ebdbc2a30dc6db3a8345))
- **app:** Show past notes about the paragraph in a margin beside the text ([b9616f3](https://github.com/iamngoni/kanso/commit/b9616f3377897cd4b7b0cca620f1a3acfed863e8))
- **mobile:** Resurface past notes and On this day through the bridge ([ae3d611](https://github.com/iamngoni/kanso/commit/ae3d61150d237ca7b5736d1f8470d7de02b61b4b))
- **mobile:** Show past notes in a pill and sheet, with settings, tests and docs ([17d1683](https://github.com/iamngoni/kanso/commit/17d168343a535da99deb66d147c7a3d9fda61af3))
- Suggest notebooks, tags, titles and links for Inbox notes, and file them with undo ([036a8c5](https://github.com/iamngoni/kanso/commit/036a8c55a1e8dae392a34ee2f94057cc0baac748))
- **desktop:** Inbox Triage view, bar, settings and AI switch ([9a41263](https://github.com/iamngoni/kanso/commit/9a41263652dcbef1bc91c4c0726719fe9d1f7664))
- **mobile:** Inbox triage calls in the Rust core ([8e483e6](https://github.com/iamngoni/kanso/commit/8e483e6f680771d56e9d3f02d13c55279a7e57f7))
- **mobile:** Bridge the Inbox triage calls ([dac4d52](https://github.com/iamngoni/kanso/commit/dac4d5264d884c1e883a28cb7d047f32c4f3a31a))
- **mobile:** Inbox Triage pager, bar and settings ([8ff19c4](https://github.com/iamngoni/kanso/commit/8ff19c4a09537657db964f1983b66dc0c05d37d2))
- **core:** Detect open loops in note text with phrase rules ([e38af76](https://github.com/iamngoni/kanso/commit/e38af764dbaa46ed20f300c5b722ac8f44cb623a))
- **store:** Add the open loops table, closing and finished history ([d69cd01](https://github.com/iamngoni/kanso/commit/d69cd019ea12456646fcdf990f160fc1a02d7940))
- **engine:** Compute the morning brief and the weekly review ([ce45938](https://github.com/iamngoni/kanso/commit/ce4593856d96a60b30702b6c25440e8c0a328443))
- **engine:** Find more open loops and summarise with AI, verified and approved ([d766e68](https://github.com/iamngoni/kanso/commit/d766e681006bbaed5d7426723b4d74d7692de1a2))
- **server:** List open loops over MCP and REST ([a1151d6](https://github.com/iamngoni/kanso/commit/a1151d6720b7605f4e95929ebd67f30fe9613ccb))
- **engine:** Schedule the brief and review notifications ([31cdbc1](https://github.com/iamngoni/kanso/commit/31cdbc10e44068abf13427492cc064d3af389764))
- **app:** Open loops, the morning brief and the weekly review on desktop ([70cce06](https://github.com/iamngoni/kanso/commit/70cce06577cda1caca16dced4e1a5625d467dd61))
- **mobile:** Open loops, the morning brief and the weekly review in the library ([6c9fe78](https://github.com/iamngoni/kanso/commit/6c9fe786a27ba8dc654cde226f3d3007b024d0f6))
- **mobile:** Bridge open loops, the brief and the review to Dart ([57fd7e2](https://github.com/iamngoni/kanso/commit/57fd7e2b0b330d20c59769b77cb004baf9a580ed))
- **mobile:** Open loops, the morning brief and the weekly review screens ([3141553](https://github.com/iamngoni/kanso/commit/3141553f920ae3e3488ae82e2201c335be62ddbc))
- **mobile:** AI loops and summaries, brief card, action-less digest notifications; review fixes ([6020554](https://github.com/iamngoni/kanso/commit/6020554c6e7087fcd9efbcf2a353864dca762823))
- **site:** Kanso Pocket on the App Store, mobile section on the download page ([93ca962](https://github.com/iamngoni/kanso/commit/93ca9626d5f3c7ea8c62e6957693da946f1c0a71))
- **store:** Apply and undo agent batches with moves ([94c2447](https://github.com/iamngoni/kanso/commit/94c2447ef601321c3b9fb6b8d8e40af5989b7219))
- **server:** Agent task change sets, task tools and in-process tool calls ([921607c](https://github.com/iamngoni/kanso/commit/921607c9d6bd5f81c4d99d602a2f36c3b91be363))
- **engine:** Agent task scope, API-key tool loop and review cards ([312f513](https://github.com/iamngoni/kanso/commit/312f513d48ab1e15388ee30918127b7499c5abc2))
- **app:** Agent tasks runner, Agent Tasks panel, new-task and review sheets ([0b0bbcb](https://github.com/iamngoni/kanso/commit/0b0bbcb2571694a7fd3464bc4fc033ade4a5555e))
- **app:** Agent task settings, selection menu, Undo Agent Task, history label ([b689c68](https://github.com/iamngoni/kanso/commit/b689c687753502a4ebbd0e106db21cd261470bc8))
- **mobile:** Agent tasks on the phone core ([8fe6b86](https://github.com/iamngoni/kanso/commit/8fe6b86fd02841ed3ff70433cae255c7daf44a3e))
- **mobile:** Agent tasks page and bridge; README agent tasks ([bef9082](https://github.com/iamngoni/kanso/commit/bef9082bcb3e78e8768b4d34e6160c022325a679))
- Agent review parity on desktop and phone (bloc, swipe review, settings, notification, keep originals) ([0274485](https://github.com/iamngoni/kanso/commit/027448514a7074564475faaf97f871908d34e08d))
- **mobile:** Ask happy users to rate Kanso Pocket ([a2cf7e7](https://github.com/iamngoni/kanso/commit/a2cf7e789f741f89e4596feb9775815eb4b9e388)) by Takudzwa Nyanhanga
- Flashcard syntax and FSRS scheduler in kanso-core ([e4c275b](https://github.com/iamngoni/kanso/commit/e4c275b4c269ed9b8d984a54be9a42786ccaf546))
- Flashcards in the store with a synced review log (review document kind) ([338f256](https://github.com/iamngoni/kanso/commit/338f2564c5b050018aa8125147b23d1e16e4c560))
- Drafting, citation check, suggested cards and Quiz Me prompts in kanso-ai ([8f9c576](https://github.com/iamngoni/kanso/commit/8f9c576de7e8c534a2411f1ce2ddc56fd5331ca1))
- Drafting and suggested-card services in kanso-engine ([8a8e0a3](https://github.com/iamngoni/kanso/commit/8a8e0a33efb0a20c840098475faa27f2b49ab1dd))
- **mobile:** Flashcards, suggested cards, Quiz Me and drafting in the phone facade ([81a58a0](https://github.com/iamngoni/kanso/commit/81a58a0d103ecbd538ff7ec7bf58a55cf078925b))
- **mobile:** Bridge flashcards and drafting to Dart ([5505f21](https://github.com/iamngoni/kanso/commit/5505f214b823a26c9469d0d0298a35b8b9573565))
- **mobile:** Review, stats, Make Flashcards, Quiz Me and Draft from notes screens ([ef7cdb3](https://github.com/iamngoni/kanso/commit/ef7cdb379c1824113f08ca83668e03bd6eef1a7f))
- Review flashcards overlay and Flashcards settings on the desktop ([b03c9b4](https://github.com/iamngoni/kanso/commit/b03c9b49a1a88e594bbf886592c62ea93492d26f))
- Draft, Make Flashcards and Quiz Me pages, Review row, reminder and entry points on the desktop ([637ba65](https://github.com/iamngoni/kanso/commit/637ba658df53744a83080f7a0e5672c1a6b241fa))
- Editor card marks, flashcard reminder, sync and budget tests for flashcards ([9b2054d](https://github.com/iamngoni/kanso/commit/9b2054d91a969d632ebfc9559fcb92900e9aafed))
- **mobile:** Flashcard strings in ARB, friendly AI errors, daily reminder, editor card marks and notebook, tag and multi-select draft entries ([e219d8b](https://github.com/iamngoni/kanso/commit/e219d8bd98253dc3959669a6f476a8f6d6e1344d))
- Voice capture pipeline and grounded tidy for voice notes ([7e4bcda](https://github.com/iamngoni/kanso/commit/7e4bcdaafde0957e2e57c611d94bb50b7ee06d35))
- Voice capture and Tidy on the desktop ([6c53ae4](https://github.com/iamngoni/kanso/commit/6c53ae48269c1dc66d69bc52b79e74a35774a438))
- **mobile:** Voice capture facade, voice drops and tidy in the bridge ([896c5f2](https://github.com/iamngoni/kanso/commit/896c5f270af2699d38380b3660cbc255a1709374))
- **ios:** Voice capture from the Action Button, Control Center and Siri, with a Live Activity ([a4c610f](https://github.com/iamngoni/kanso/commit/a4c610fe4e528294c5c17ff94e12d26070783e66))
- **android:** Voice capture from the tile and the widget, and file transcription ([75ef7e8](https://github.com/iamngoni/kanso/commit/75ef7e8d0a55cf316765acddd0577927facd6c92))
- Audio-only voice drops, and clippy cleanups ([29ebe3e](https://github.com/iamngoni/kanso/commit/29ebe3e57a700d4de0d7bbb3a41470c6d272f6b7))
- **mobile:** Voice capture in the app, voice settings, tidy strip, and bridge tests ([5bfd2f9](https://github.com/iamngoni/kanso/commit/5bfd2f963cb501fdffd168932dbc97d15c3419bf))
- Passages, time in questions, line links and passage vectors for Ask ([195b909](https://github.com/iamngoni/kanso/commit/195b909f0cef40f5467619e4f1fb8eac94f2ef6f))
- **ai:** Numbered excerpts, citing instructions, budgets and prices ([395640a](https://github.com/iamngoni/kanso/commit/395640a316e0770c754e6e054e3673f8819175c3))
- **engine:** Receipts, scope, time, passage retrieval and the question driver for Ask ([5938808](https://github.com/iamngoni/kanso/commit/5938808eb0a7febc4630009a727f60988a0ae7e7))
- **engine:** Save as note, passage vectors in the indexer, and the requests line ([9e8ae3c](https://github.com/iamngoni/kanso/commit/9e8ae3c9a0d75537cd5318f729cf6b801862efbb))
- **server:** Ask tools that number what they show, for agents ([e65640c](https://github.com/iamngoni/kanso/commit/e65640c270a78563631c6869418031052c2c22cb))
- **app:** Ask with receipts, scope, time and sources on the desktop, and agent numbering ([7fded7c](https://github.com/iamngoni/kanso/commit/7fded7c689944c99aea2b00213b5e4eeb00966ce))
- **mobile:** Receipts, scope, time and save-as-note in the mobile core ([9747b6d](https://github.com/iamngoni/kanso/commit/9747b6d9201da713e174adb1f9d9466645311ec1))
- **mobile:** Regenerate the bridge for Ask with receipts ([19220df](https://github.com/iamngoni/kanso/commit/19220df4cd69d7f3d11a0d79e7f3b05c9da0091a))
- **mobile:** Forget the newest exchange in the mobile core ([8932d27](https://github.com/iamngoni/kanso/commit/8932d2708fe61c04d8a26fcd4f8418bf6162d8f2))
- **mobile:** Ask with receipts, scope, time, sources and Ask settings on the phone ([3458130](https://github.com/iamngoni/kanso/commit/3458130d60d39ce88b55befe5c8c5293d354c426))
- **mobile:** Regenerate the bridge after the rebase ([104cc05](https://github.com/iamngoni/kanso/commit/104cc050900f7c367d3f603a00b2ffaf253cc629))
- Mark sentences with no source, hold cut ranges to the gap rule ([b4c42c3](https://github.com/iamngoni/kanso/commit/b4c42c36f253688a042fa32282ac05c430a551ea))
- End a conversation whose used note is kept out; relocate receipts and count unsourced sentences on the phone ([ac0399a](https://github.com/iamngoni/kanso/commit/ac0399a7131791c6e559a18aaa282b990de890ed))
- **app:** Draw failed and unsourced sentences apart before any click ([4e2ef11](https://github.com/iamngoni/kanso/commit/4e2ef110e58de16eb7139a100d22f5f72e1cc68b))
- **mobile:** Relocate receipts after edits, count unsourced sentences, end a conversation whose note is kept out ([f645876](https://github.com/iamngoni/kanso/commit/f645876c48495d9cee515e18a41e2167010dd3b6))
- Entity pages, mention index and page cache in the core and store ([38ef269](https://github.com/iamngoni/kanso/commit/38ef26911dcf4c00d8bd355d0f32d8beeff8d9da))
- **engine:** Generated-page core, people and project pages, detection and suggestions ([27a60fb](https://github.com/iamngoni/kanso/commit/27a60fb2ba66f0971bf93bb03f138a257b6909c6))
- **server:** Kanso-list-mentions for agents ([84147ea](https://github.com/iamngoni/kanso/commit/84147eae284a4a12a3a60af390144070a2adbed8))
- **mobile:** People and projects in the mobile core ([5fa87e0](https://github.com/iamngoni/kanso/commit/5fa87e02a9b02a01347c416ed58ab5aa97de64a6))
- **mobile:** Regenerate the bridge for people and projects ([fc0d364](https://github.com/iamngoni/kanso/commit/fc0d364f578333deaead62a442d90473cf77aad1))
- **mobile:** People and projects pages, suggestions and settings ([1c6a3c1](https://github.com/iamngoni/kanso/commit/1c6a3c19ece73fc10968f432c83b259129742905))
- **app:** People and projects page, panel, settings, sidebar and mentions ([05deb53](https://github.com/iamngoni/kanso/commit/05deb538f37d139b9a30c51433894b8e85154d1c))
- **mobile:** Choose the notebook people and project pages are kept in ([cda245c](https://github.com/iamngoni/kanso/commit/cda245c2dcaf54862ece805943616882699a4eb3))
- Undo for trashing a single note, with Cmd/Ctrl+Z while its notice is up ([f570a18](https://github.com/iamngoni/kanso/commit/f570a189829198ce124223c2e63028cfc177182e))
- **store:** Notebook subtree, homepage reads, notebook page cache and Home note mark ([f5e81fa](https://github.com/iamngoni/kanso/commit/f5e81fa8cd49aed8c0516261b1171f0dfbc8f7e7))
- **engine:** Notebook homepage source, Home note actions, map and locked-notebook view ([deb991e](https://github.com/iamngoni/kanso/commit/deb991ec5779c60455b4fea3b5239efee70cc675))
- **mobile:** Notebook homepage facade in the mobile core ([41b82df](https://github.com/iamngoni/kanso/commit/41b82df09da4aeced8f2720bcd8d7f2e3405005a))
- **mobile:** Regenerate the bridge for notebook homepages ([f03b049](https://github.com/iamngoni/kanso/commit/f03b049f21fb5d1657700c46a2bda491224bdea0))
- **app:** Notebook homepage in the editor pane, Home row, Go menu entry and settings ([8257730](https://github.com/iamngoni/kanso/commit/8257730f4764f336c75697bb3e45950d885aea7e))
- **mobile:** Notebook homepage screens, parent-only home and move toast; faster homepage queries and benchmark ([529bfa8](https://github.com/iamngoni/kanso/commit/529bfa8ad765c43596ad80d182291a770fabb70d))
- Import Apple Notes exported as files (HTML, RTF, text, Markdown, PDF) under an Apple Notes notebook ([1117c5b](https://github.com/iamngoni/kanso/commit/1117c5be8dd9f3aa46ef68a275ab5d17137d0a4e))
- Turn a note shared from Apple Notes into a Markdown note with tasks ([f8eb62c](https://github.com/iamngoni/kanso/commit/f8eb62c3a842bde9858dc10faa57e403c63c100a))
- Add the Apple Notes guide to Import on iPhone and iPad ([bf7dd95](https://github.com/iamngoni/kanso/commit/bf7dd957c4a62d001dfe2a22bdab9d7fed5e31b2))
- Timestamped transcripts between markers for recordings (#38 step 1) ([d9c45b0](https://github.com/iamngoni/kanso/commit/d9c45b0dd979bac76ae42240246c5115340e1cab))

### Improved

- **app:** Read the open-loop count off the UI thread ([1e8e01e](https://github.com/iamngoni/kanso/commit/1e8e01e8a95f2c13756dba4e62f35f3482714939))
- Clear the clippy warnings in the Ask code ([ca76ddb](https://github.com/iamngoni/kanso/commit/ca76ddbe88f2bac18cdf97fd5be8545271d698cc))
- Clear the clippy warnings from the review fixes ([46c4cda](https://github.com/iamngoni/kanso/commit/46c4cda8a4d2c329a43a375cf57263a93edf8c42))
- **engine:** Read a notebook's subtree once per homepage and measure the 10,000-note budget ([54113e2](https://github.com/iamngoni/kanso/commit/54113e27d2b452649f1c49c48b30f872af3b33f0))

### Fixed

- **resurface:** Show the mobile pill in reading mode and for On this day, and address review findings ([a432f12](https://github.com/iamngoni/kanso/commit/a432f125bcb50428b84c8ae4d51d7403c61e5268))
- Keep the Inbox settings below Data and clear two clippy warnings ([0af24da](https://github.com/iamngoni/kanso/commit/0af24da5051c47d6679329bcd5a5867ed663646b))
- Inbox Triage review findings (undo, chips, perf, pager, migration 13) ([4077e66](https://github.com/iamngoni/kanso/commit/4077e669a815311013124781ec685114d43ab2e7))
- **app:** Bundle the new icons and scroll to Backups in the settings test ([51c812d](https://github.com/iamngoni/kanso/commit/51c812df004eff3d520057de5ec13b2efbdc617b))
- **loops:** Keep a loop's key when it is marked done; union-merge loops and acks on sync ([34bb93f](https://github.com/iamngoni/kanso/commit/34bb93f06512c35dfa6a403100f952fcaa4c1fd0))
- **server:** Filter #ai-exclude loops in SQL for the MCP tool and REST, push the limit down ([783e922](https://github.com/iamngoni/kanso/commit/783e922130f27200a6fd4b0dcd18bfd3520f8ac8))
- **site:** Title the download page for every device ([8333b78](https://github.com/iamngoni/kanso/commit/8333b78d37bebcd770f0d07d7ccac7b7b22abb64))
- **app:** Brief, review, open loops and triage as full pages with fitted headers, real empty states and friendly AI errors ([7f7022d](https://github.com/iamngoni/kanso/commit/7f7022dbbf6ac7c4ae9bbb0036d52074b5241252))
- **app:** Pages give way to any opened note, Triage closes on a place, Escape returns, fitted header labels, error details ([62515a8](https://github.com/iamngoni/kanso/commit/62515a811841ca12a694afccf049ad13197a7ded))
- **site:** Serve the last good release when GitHub fails ([5b4c2c9](https://github.com/iamngoni/kanso/commit/5b4c2c97d76cbde54a06b6eececb0f867efd692b))
- **mobile:** Google sign-in can no longer hang the sign-in screen ([5e7adab](https://github.com/iamngoni/kanso/commit/5e7adab3dd6f151913c7ddec82319be04bd5c8be))
- **mobile:** Read the chosen notes, not notebooks; desktop review keeps originals, renders Markdown, folds Ask suggestions ([6d22651](https://github.com/iamngoni/kanso/commit/6d226515097d75feaf3dfebf43b4fa4318f3180c))
- Agent tasks write only where the person allowed, with limits, undo to Trash and phone parity ([2e5470a](https://github.com/iamngoni/kanso/commit/2e5470a2bccb36189ce3c639abf7514cb1a203c1))
- Put the task last in the agent prompt, after where new notes go ([00fdf8d](https://github.com/iamngoni/kanso/commit/00fdf8df33fe491730ccd212cb6d9885ebd46926))
- Clippy findings, and keep the Backups test in view on a longer General page ([47a6ef0](https://github.com/iamngoni/kanso/commit/47a6ef03f54e2bfce0f23acd8f2eef5e0a3567ac))
- Use bundled icons for the learning pages and refresh the tablet Assist goldens ([a01395d](https://github.com/iamngoni/kanso/commit/a01395dc0f673572aa044306c82700dde1aaa24e))
- Review prompt counts only deliberate leaves, waits out modals, keeps times in UTC ([0ea7364](https://github.com/iamngoni/kanso/commit/0ea7364a4f852d8f1428ecd311e2b69e71ca2dd1))
- Never lose a capture, keep recordings kill-safe on both phones, label AI wording, and undo a tidy ([3dc882c](https://github.com/iamngoni/kanso/commit/3dc882c3868ddd55c971383aef908852f88bc7f6))
- Quarantine instead of deleting, surface problems, undo a tidy on the phone, plain AI errors ([3351111](https://github.com/iamngoni/kanso/commit/3351111d87f9d323f2eef9309f27334cdb9eb0b0))
- Keep resets and deletions through sync, check that cited notes support a draft, and read the Review badge off the main thread ([05c63ae](https://github.com/iamngoni/kanso/commit/05c63ae9b4321df5a6e5f52438bfb5860fdf997a))
- Keep the drafts and quizzes sentence in the requests line ([8e5198f](https://github.com/iamngoni/kanso/commit/8e5198f74691a814b668f6fe785c301243aaf4f1))
- Count every round, the repair and the rewrite in the most a question can cost; current model prices ([c0cabee](https://github.com/iamngoni/kanso/commit/c0cabee55529e1e88660177da6e300f00e6a3214))
- The repair prompt carries the lines the tools read, and a repair that checks out less never replaces the answer ([3606265](https://github.com/iamngoni/kanso/commit/3606265df85d60187a8429c5e3930eed6a9113fb))
- Rolling past week and year, 29 February, ambiguous may and march, and each end's own offset across daylight saving ([e0b7124](https://github.com/iamngoni/kanso/commit/e0b71243309cb0bf80094aaa0e5e0fd507d66a30))
- Reset only notes a card may be shown from, and give the cards tests fixtures their cited notes support ([2744916](https://github.com/iamngoni/kanso/commit/2744916de03d93a68416ffc0dc0b8c0c119f47a1))
- **app:** Draw the people views with icons the themes cover and open without settings ([dfa8ffe](https://github.com/iamngoni/kanso/commit/dfa8ffe279e87523abb0b385207b05207e344dfe))
- **people:** Link @mentions to their pages, keep un-keeping across devices and close the parity gaps ([b20d0f9](https://github.com/iamngoni/kanso/commit/b20d0f90b85305edea8fed4f0f3d561795fcb56d))
- Let the main window shrink to half a small Windows screen and fold the sidebar to its rail there ([f5776b6](https://github.com/iamngoni/kanso/commit/f5776b6a1f133e3d23b98cb569503e6d6c081db1))
- Keep the sidebar toggle, inspector and note toolbar working in a half-screen window ([b7d1168](https://github.com/iamngoni/kanso/commit/b7d11688484e506e141f82495fbb14ecbd745c82))
- Tie Cmd/Ctrl+Z to the Undo notice, keep text and task-list Undo, palette Undo tests ([6418ddf](https://github.com/iamngoni/kanso/commit/6418ddfdb3f07226ccc0c8e54a68fd8cd1419650))
- **mobile:** Cmd/Ctrl+Z shortcut is safe without an Undo notice bloc above it ([6e971da](https://github.com/iamngoni/kanso/commit/6e971dae11ea2c4ebda7573c4e3555fb5329b6b7))
- **app:** Pass the keep time to page pins after the rebase and regenerate the bridge ([a959010](https://github.com/iamngoni/kanso/commit/a9590106f73fd2a88f819bab5a3605e0265010f3))
- **mobile:** Leave Go Notebook Home without a phone key ([295cccb](https://github.com/iamngoni/kanso/commit/295cccb8b0d54501544805a3a524c7c3973cc414))
- **app:** Desktop homepage gaps: move rule and notice, return from a new Home note, drag handles, lock state ([def31b5](https://github.com/iamngoni/kanso/commit/def31b5ea00d927b86e49974f59fcd771f0f2298))
- **mobile:** Home notes read as preview, phone Ask is the sparkles alone ([f64b158](https://github.com/iamngoni/kanso/commit/f64b15890a1ca02c99cbb3ccb889fac05e91f738))
- Make two flaky tests deterministic (random note ids, f32 tolerance) ([eb45cf3](https://github.com/iamngoni/kanso/commit/eb45cf349d1bd1bcefd7db15c047647dab9fec20))
- Point the Apple Notes help at a real page, keep drawing labels off Android photo shares, test zip and locked-notebook safety ([76e9798](https://github.com/iamngoni/kanso/commit/76e979855691d553bd88be9aa338d330b3810db8))
- Replace only this recording's untouched transcript block; hide HTML comments on mobile ([432fbd4](https://github.com/iamngoni/kanso/commit/432fbd456bfd6e49700ecb7adc05e933eda2d3c0))
- Gate notebook_home map tests on the discovery feature; verify downloaded release artifacts ([8ea34d6](https://github.com/iamngoni/kanso/commit/8ea34d65c8b1a7dbb2d058ee9c5104b688715dc4))

### Under the hood

- **mobile:** Regenerate the bridge after rebasing on resurfacing ([982fbc3](https://github.com/iamngoni/kanso/commit/982fbc38a53ee1dc63df13b6de5bcbfbafbeb24f))
- Rebase onto master (migration 14), regenerate the bridge and the settings goldens ([e6c780b](https://github.com/iamngoni/kanso/commit/e6c780b9a23146f092eace3e2dcca1cc6ab09d93))
- **mobile:** Refresh settings goldens for the On this day row ([c53fab8](https://github.com/iamngoni/kanso/commit/c53fab891f4505307c9d874215b75f5898114996))
- Move the agent task runner and model into the engine ([7da3b24](https://github.com/iamngoni/kanso/commit/7da3b24993937aab0c0852abb533a9ed5dc35f75))
- Clippy fixes for agent tasks ([aadb66f](https://github.com/iamngoni/kanso/commit/aadb66f5d99689eeff4b3b200b60e206ba4c1995))
- Reuse the AI trouble text for agent task failures ([560a1ef](https://github.com/iamngoni/kanso/commit/560a1ef8f4cf61f06aabd1db8fc13bc7501a4042))
- Derived version 4 and a rebuild test for cards ([3058cc9](https://github.com/iamngoni/kanso/commit/3058cc92c6b7274fc002761eb256c1b7e7621933))
- **mobile:** Freezed output for the flashcard and drafting bridge types ([ace646d](https://github.com/iamngoni/kanso/commit/ace646dfe0df8c1de99355296f2b7047c83a0503))
- **mobile:** Bridge suite for flashcards and drafting, and regenerate the bridge after the rebase ([542fd15](https://github.com/iamngoni/kanso/commit/542fd158bf1dfddba3755dcaefb80cbd246f6a64))
- Learning pages, bundled icons and the bridge-aware test harness ([1489678](https://github.com/iamngoni/kanso/commit/14896788a80650d21b5df1c936cf28140aca7393))
- Regenerate the bridge and the lock file after the rebase ([254c53a](https://github.com/iamngoni/kanso/commit/254c53aa9f590209d5820083d9025de4fc401677))
- Regenerate the bridge after rebasing onto master ([1cb564b](https://github.com/iamngoni/kanso/commit/1cb564b93c6d9ccec1362975f27af935c8c0cbe5))
- Merge master into the learning branch and regenerate the bridge ([04b8d22](https://github.com/iamngoni/kanso/commit/04b8d22af0cfa5d717f58467ff6418b1fa3e4e25))
- Restore master's proptest regression files and drop the ones a test run left ([9cb4ac1](https://github.com/iamngoni/kanso/commit/9cb4ac1a3920b79225654e47693756f1ae58d243))
- **engine:** Evaluation library, questions and checker answers for Ask ([fcd2bb9](https://github.com/iamngoni/kanso/commit/fcd2bb995998d5ec87e726961a5fb35d4a933de7))
- **store:** Date bounds on search ([d9d7030](https://github.com/iamngoni/kanso/commit/d9d703002d97e1d3a398c901da8cf26b969550ac))
- **server:** Tool count with the Ask tools ([70d3382](https://github.com/iamngoni/kanso/commit/70d33821b191dcba8b50649c1975fcf9193555dd))
- **server:** Listed tools with the Ask tools ([87b99e4](https://github.com/iamngoni/kanso/commit/87b99e465e948683f17be017fac2a693fbcda4ab))
- **server:** Listed tools for a task scope ([ceeef95](https://github.com/iamngoni/kanso/commit/ceeef95f0378c8b0ccb7bcd97f9338f4473fe7d3))
- **app:** Snapshots for the Ask panel's receipts, never-wrote and banner states ([13470b2](https://github.com/iamngoni/kanso/commit/13470b2a5a3f4c96132774d83b2277eafe82cece))
- **mobile:** Receipts, sentinel, save and settings ([57fdac6](https://github.com/iamngoni/kanso/commit/57fdac6456c06fce7a85ef2ef2663050150431fd))
- **mobile:** Receipts, scope, save and Ask settings on the phone ([7700597](https://github.com/iamngoni/kanso/commit/77005977501dec50fb841f15fd698e95ea75ed41))
- **mobile:** The model's own answer handed back for receipts is not a leak ([d728082](https://github.com/iamngoni/kanso/commit/d728082cc51bdcb6e11c6647a3398681d2e052ab))
- Give the ai-exclude conversation test a notebook ([6381844](https://github.com/iamngoni/kanso/commit/6381844271dec0f3122ea2bc2408731e696ec2ef))
- **mobile:** Cover the people bridge on devices, hide the empty drawer row and update the settings goldens ([c56ca6c](https://github.com/iamngoni/kanso/commit/c56ca6c6624e89d73f697921538d91119b1f0e49))
- **store:** Expect the derived index at version 5 ([bae97f9](https://github.com/iamngoni/kanso/commit/bae97f903b84449d7db6b36f53e0ffb94ec1b71c))
- **app:** Notebook homepage rows, menu and window tests, catalog entry for Go Notebook Home ([65e60ef](https://github.com/iamngoni/kanso/commit/65e60ef71f821f2a39ad7af7e1023502221e48b8))
- Clippy type aliases, phone Ask goldens and semantics finder, analyzer line length ([b571f6c](https://github.com/iamngoni/kanso/commit/b571f6cfe802cbd6921890e654f15de74aff69e3))

## [0.9.3](https://github.com/iamngoni/kanso/compare/v0.9.2...v0.9.3) - 2026-11-02

### New

- **app:** Present a note full screen as slides ([b640a0c](https://github.com/iamngoni/kanso/commit/b640a0c9eccba54554d39f6b37bf3e4c2a678b09)) by @gtchakama
- Present on phones and tablets, and fix the review's findings ([37dc2bd](https://github.com/iamngoni/kanso/commit/37dc2bdf1329178634386f9d48b5d17300b94725)) by @gtchakama

### Fixed

- Rebuild a damaged search index when the store opens ([629d30f](https://github.com/iamngoni/kanso/commit/629d30f61906e80a86487afeefdf448d7efd13d6))
- Close the pre-push review's findings on presenting ([ad7ad79](https://github.com/iamngoni/kanso/commit/ad7ad7990118bb7709d0aef93bfc4cd9dc4741cb)) by @gtchakama

### Under the hood

- Provide the blocs above the app so the pushed slides see them ([71a8d8e](https://github.com/iamngoni/kanso/commit/71a8d8ee067a1ecd77018417ca1971cb30b6cf05))

## [0.9.2](https://github.com/iamngoni/kanso/compare/v0.9.1...v0.9.2) - 2026-11-01

### New

- Select all notebooks in the plugin install sheet ([cf56130](https://github.com/iamngoni/kanso/commit/cf56130aae4abe065a8ceeb531ff0a819e88c574))

## [0.9.1](https://github.com/iamngoni/kanso/compare/v0.9.0...v0.9.1) - 2026-11-01

### New

- Refresh the plugin directory from Settings › Plugins ([e500f42](https://github.com/iamngoni/kanso/commit/e500f424d157e0bdf7566ed9967482c5da5537aa))
- **metrics:** Hourly batches with a ledger, new events, ranges and closed lists ([8890996](https://github.com/iamngoni/kanso/commit/8890996291314656c7ab9879598ab06d8940d547))
- **cloud:** Validate the new metrics events, stamp receipt time, and send server_stats ([3087cd9](https://github.com/iamngoni/kanso/commit/3087cd96af837ef6324c77cf77210d40b20e0ced))
- **app:** Send hourly and on quit, show the notice again, count 24 more features ([eb7d3a4](https://github.com/iamngoni/kanso/commit/eb7d3a402a8fed3f1494546296e1bb902d38d608))
- **mobile:** Hourly and background batches, notice version and new counts ([3510596](https://github.com/iamngoni/kanso/commit/35105966caa97643a75903c0bcec9c7444d32d39))

### Fixed

- Preserve WorkManager database constructor and service type ([bf608bf](https://github.com/iamngoni/kanso/commit/bf608bfbb788f76adc8227586e6a375c78fce856))
- **sticky:** Tint stickies from the active theme, and fit toasts to narrow windows ([9f22350](https://github.com/iamngoni/kanso/commit/9f223509657e7a661e14ea67f1c53da1626297eb))
- Close a sticky from its close button directly ([bcdbc26](https://github.com/iamngoni/kanso/commit/bcdbc263033ae9517e66dffbec349136c770b050))
- Keep the .deb version's tilde free of a backslash on older bash ([dbbe52b](https://github.com/iamngoni/kanso/commit/dbbe52b423ca5254aa0c4fc71d06bad588e076a7))
- **metrics:** Check the PostHog key and log each forwarded batch ([f509f3f](https://github.com/iamngoni/kanso/commit/f509f3f669c94e16aa53266c980ad5752d6982c3))
- **markdown:** A rule after a setext underline no longer crashes markdown-rs, and Live falls back to one block per line ([a7adeb9](https://github.com/iamngoni/kanso/commit/a7adeb9a3899633a75368238e2b1d015c497a33a))
- Drop a stray proptest regression for the markdown crate ([cb2d38c](https://github.com/iamngoni/kanso/commit/cb2d38cdce0556d5d71316db4145b3fcbe2ebc1f))
- **cloud:** Catch up missed server_stats windows, count devices_seen hourly, route once ([3a7dc03](https://github.com/iamngoni/kanso/commit/3a7dc03533ce5dc3f2b03ce4e2193a7fbc011952))
- **app:** Buffer counts, note the launch at start, and don't resend on quit ([d90663f](https://github.com/iamngoni/kanso/commit/d90663f10c37dd9e3d17f60e900a6a48cab02fe7))
- **mobile:** Buffer ledger writes, count batches per note, search on open, no note_deleted on delete forever ([d9c48d9](https://github.com/iamngoni/kanso/commit/d9c48d92b242950a623abe80c0ff262d82447eba))
- **ci:** Pin the MCP contract to 1.6.0 and fix Windows and Android lints ([898697e](https://github.com/iamngoni/kanso/commit/898697e6ee773784f933096e784db9fde2f82b4b))

### Under the hood

- Update App Store name and review notes ([0c5028f](https://github.com/iamngoni/kanso/commit/0c5028fed48d3ada639d67b63c4e79fa591fa74f))
- **changelog:** Credit the sticky notes feature to @Zawala ([55709a0](https://github.com/iamngoni/kanso/commit/55709a0f553c803a40abcdacf1347624072c5c0e))
- **changelog:** Give Kelvin his full name in the sticky notes credit ([e371169](https://github.com/iamngoni/kanso/commit/e371169e37cca8055e4d6f33b71a5dd0c0003e4c))
- Leave package-managed installs to apt ([31ab542](https://github.com/iamngoni/kanso/commit/31ab542b87a6a7ec1efc05ba0d30c35f9c2a277d)) by @Zawala
- Publish the .debs and a signed apt index ([1b25076](https://github.com/iamngoni/kanso/commit/1b25076187b6be6fbbbf549221b5fae61a343408)) by @Zawala
- Apt redirects at /apt ([629d427](https://github.com/iamngoni/kanso/commit/629d427443291c58a12277607ca33923841e8d63)) by @Zawala
- Apt-setup for the maintainer's one-time key setup ([8aa6fff](https://github.com/iamngoni/kanso/commit/8aa6fffa6be6294a3eacf0587179db947f9fe316)) by @Zawala
- Apt install, the repository's design and its setup ([d68302d](https://github.com/iamngoni/kanso/commit/d68302d0bfb9ef05720974a6f45686d68e7a772a)) by @Zawala
- Use the setext shape markdown still panics on after the retry ([99f27b6](https://github.com/iamngoni/kanso/commit/99f27b62c36e53baa882a3cbbbc3b6580d92b7e5))
- Define the expanded metrics in METRICS.md, the notice, the site and store disclosures ([f817f2a](https://github.com/iamngoni/kanso/commit/f817f2a641888317bd662f4d0fda9c2417b8a1a3))
- Allow Kanso's template bookId in the MCP contract test ([3ce30f8](https://github.com/iamngoni/kanso/commit/3ce30f81c1b756f97da36d9f9a51e38da65c7b6d))
- Expect the platform's path separator in the agent report ([0abfa8f](https://github.com/iamngoni/kanso/commit/0abfa8fb5d5572af60fa3a2829ee7f3f2bb6ac16))

## [0.9.0](https://github.com/iamngoni/kanso/compare/v0.8.4...v0.9.0) - 2026-10-31

### #44

- Seal locked-note attachments end to end ([#143](https://github.com/iamngoni/kanso/pull/143)) ([83764f9](https://github.com/iamngoni/kanso/commit/83764f94fdd7088f1f29d4dade23736cb0bcc0a5))

### New

- **app:** A note can open as a sticky note that stays above other windows, with a kanso sticky command, shortcuts and a Linux XWayland option ([963174e](https://github.com/iamngoni/kanso/commit/963174e146c69c04acfc2d67df6f77748477b855)) by @Zawala (Kelvin Zawala)
- **sidebar:** A notebook can be washed in a color, chosen from swatches, a picker or typed hex, so it's easy to find ([bae7c9d](https://github.com/iamngoni/kanso/commit/bae7c9d953160ac2b68246f8edf78000d8f34fb7)) by @gtchakama
- **preview:** Mermaid diagrams are drawn by Pele, with Kanso's own renderer a choice in Settings › Editor › Preview ([5c2f6af](https://github.com/iamngoni/kanso/commit/5c2f6af8b9709a44e99b533f9ae0f3ce4f5ecc73))
- **preview:** Math in Mermaid labels is typeset, and MathJax and Pele share one QuickJS engine ([df428f9](https://github.com/iamngoni/kanso/commit/df428f9670a92fc49d40c02fe22e960ae4d70bef))
- **export:** Pages use the preview's typography and code colors ([9ed041e](https://github.com/iamngoni/kanso/commit/9ed041ed0fd39c4c1033d103e3ae73bb7794ae2c))
- **updates:** Look for updates every half hour, sooner after a failure and on waking ([e5d6eb2](https://github.com/iamngoni/kanso/commit/e5d6eb21859d0b900abc0d3f1e1a9209ed7b2193))
- **core:** Notes carry properties and query views ([7d78013](https://github.com/iamngoni/kanso/commit/7d780134ede01c7eb8d546cc5b6749a7097e29ec))
- **store:** Read notes for query views, leaving locked text sealed ([aad1511](https://github.com/iamngoni/kanso/commit/aad151185afdeb22914c99bef18116e4cce0e218))
- **app:** Properties in the inspector and query tables in the preview ([fd2e723](https://github.com/iamngoni/kanso/commit/fd2e723dce18fd83b2389373bca94be58bf713f7))
- **gallery:** Add the signed theme catalog and SVG sanitizer ([dba6419](https://github.com/iamngoni/kanso/commit/dba6419323e0eb2c453814670db85d95ff304117))
- **themes:** Compile every theme in the kanso-themes index in ([a65aa68](https://github.com/iamngoni/kanso/commit/a65aa68a1cbbaf55f471be54f0bfd2ada91750ff))
- **tabs:** Keep several notes open as tabs in the main window ([7f9ddc3](https://github.com/iamngoni/kanso/commit/7f9ddc379c0137fc61abda8de8fdce5f6b8080fb))
- **ai:** Add a provider crate that streams answers from local and cloud models ([fa0570d](https://github.com/iamngoni/kanso/commit/fa0570d2f5feb7307387b7821c248198e9e7f7e9))
- **ai:** Ask your notes answers from your own notes, with links to them ([c7d2928](https://github.com/iamngoni/kanso/commit/c7d29280d4d93fe81551861f12a684afd14c66c1))
- **ai:** Score retrieval and let the model search the notes ([07f300d](https://github.com/iamngoni/kanso/commit/07f300dd802195aec8d0aac6a78d50f458467723))
- **ai:** Keep the Ask conversation and mark cut-off answers ([416084c](https://github.com/iamngoni/kanso/commit/416084c8fa68747b9f57cc02aa6c71c7588b40c7))
- **ai:** Finish AI settings and hide Ask while AI is off ([3bdb20f](https://github.com/iamngoni/kanso/commit/3bdb20f9c2fd2b185e20477f937c1badea27812f))
- **gallery:** Install gallery themes and browse them in Settings ([560849b](https://github.com/iamngoni/kanso/commit/560849b3a2542a1593d4848d6b1a0b3133f114ae))
- **gallery:** Update, revoke and fall back on the update timer ([f0736db](https://github.com/iamngoni/kanso/commit/f0736db445f0054aacf12c139344410551b0876c))
- Add passkey and provider sign-in flows ([f855789](https://github.com/iamngoni/kanso/commit/f8557890a4d34035ac6309e6437a79eea62a61a2))
- **sign-in:** Complete provider flows, tickets and linking ([71cca1b](https://github.com/iamngoni/kanso/commit/71cca1b3bdf1f0793d9c8ce797e2c8ed85a2964a))
- **sign-in:** Add passkeys and the PRF wrap ([49f1fca](https://github.com/iamngoni/kanso/commit/49f1fcaebb149c269531250b5a29e737fec4337d))
- Add passkey authentication and account management contracts ([02bd7e4](https://github.com/iamngoni/kanso/commit/02bd7e43d83f89eebc0f06b4783c1fdc82169386))
- **sign-in:** Approve a new device and hear account events live ([c00e3b4](https://github.com/iamngoni/kanso/commit/c00e3b4a138c31cec8b6fb7c3747cbf62123006c))
- Improve device approval flow and queue requests ([e4a9cce](https://github.com/iamngoni/kanso/commit/e4a9cce2a614e509dc20603fcf93e9c3099c0e24))
- Add end-to-end device approval flow ([66d1ae8](https://github.com/iamngoni/kanso/commit/66d1ae89c47ee78de21905cbe3ef8fd37decf04a))
- Add resumable device approval and locked-account unlocks ([34e9063](https://github.com/iamngoni/kanso/commit/34e9063b69490e969412c595e176ab12bcdc8b48))
- **app:** Run native passkey ceremonies behind a narrow account adapter ([59f5738](https://github.com/iamngoni/kanso/commit/59f57382b0f345e9ca3e93efd8eff527cdfca695))
- **sign-in:** Sign in with Apple through the system browser on Windows and Linux ([a20624a](https://github.com/iamngoni/kanso/commit/a20624ac0d7123a44002f7424979d773874941b6))
- **sync:** Add passkey proofs and the passkey sign-up, sign-in and method flows ([6c2a1a2](https://github.com/iamngoni/kanso/commit/6c2a1a2bee90632b3cf1157bde925582b5c8a5fb))
- **sync:** Make MemoryServer a verifying relying party with the proof and passkey routes ([9fa70c6](https://github.com/iamngoni/kanso/commit/9fa70c64a65994c0ef3e2175afacdda1abff0f8c))
- **sync:** Verify a Linux security key's local PRF assertion before its wrap ([2339bb4](https://github.com/iamngoni/kanso/commit/2339bb4930604e2832b21438f18e6849da001f8a))
- **sign-in:** Run Linux security-key ceremonies through kanso-sync's checked registration ([e4044e9](https://github.com/iamngoni/kanso/commit/e4044e91e87a78f51380d79a63e7e38e4348f54a))
- **app:** Offer passkeys and security keys on the signed-out Account page ([077c5eb](https://github.com/iamngoni/kanso/commit/077c5eb1c389209f3b026fbdb1d043035223d30f))
- **sync:** Give 409 no_password its own AccountError, and read a pending link poll ([ebbf927](https://github.com/iamngoni/kanso/commit/ebbf9272a2976cceed9fbae3bf8a8d7486226537))
- **account:** Change sign-in methods in Settings behind a fresh proof ([3e4826b](https://github.com/iamngoni/kanso/commit/3e4826b9beba2297dc028e5176a4fc7e83421f6f))
- **sign-in:** Accept passkey proofs through a shared proofs module and add proof/start ([52ac9d7](https://github.com/iamngoni/kanso/commit/52ac9d7f2d878f502646c2af55a13b1c1f1afde6))
- **sign-in:** Add passkeys to a signed-in account ([ca61987](https://github.com/iamngoni/kanso/commit/ca61987d7d13872ca19b0253e04fe1a4403afc73))
- **sign-in:** Remove a passkey with a fresh proof ([94e2f58](https://github.com/iamngoni/kanso/commit/94e2f580d61d1ac6eaba3b237dc30c812e4a3193))
- **sign-in:** Set the first password with a fresh proof ([c580b36](https://github.com/iamngoni/kanso/commit/c580b368b675c9b12efb41a24778fba3d241dd44))
- **sign-in:** Replace the recovery key and delete the account with a fresh proof ([60d5d30](https://github.com/iamngoni/kanso/commit/60d5d3017c93cc6369a7762f7db766ced5515b59))
- **sign-in:** Serve the apple-app-site-association file from APP_ASSOC_APPS ([a5e6be7](https://github.com/iamngoni/kanso/commit/a5e6be70e753ae7076155c350b4e2415837d5023))
- **sign-in:** Close event sockets on eviction, deletion, admin disable or delete, and expiry ([f7d3491](https://github.com/iamngoni/kanso/commit/f7d34916dee8ccfeb72f46cfee8c102e023dbfdd))
- **account:** GitHub's sign-in code is copied, ready to paste on its page ([dbfac16](https://github.com/iamngoni/kanso/commit/dbfac1608b9ed205be1e3a10fcf7d19383e6041b))
- **account:** The sign-in and create-account forms are one designed column ([bbf770a](https://github.com/iamngoni/kanso/commit/bbf770ab625dee72d4d05ab6508a378e4fd851db))
- **ai:** Answer with an agent on this computer over ACP ([da66ccc](https://github.com/iamngoni/kanso/commit/da66ccc444c447b972a3a8c03b75a2be2e4b5c72))
- **server:** Agents read notes over /mcp with a scoped, expiring token ([8dcc85e](https://github.com/iamngoni/kanso/commit/8dcc85ed42d1af766ab48fb6200670686b2f1041))
- **ai:** An agent searches and reads the notes it may, and follow-ups keep its session ([abb8de2](https://github.com/iamngoni/kanso/commit/abb8de20c779535e453c0bdd197c358d16f9e166))
- **ai:** Settings chooses an agent's notebooks and installs agents from the ACP registry ([9b05d34](https://github.com/iamngoni/kanso/commit/9b05d347fbc18d5f51f7f450723fab04508496a0))
- **ai:** History records an agent's edit that the person applied ([a06b2d7](https://github.com/iamngoni/kanso/commit/a06b2d7e5a6478bc792810f3014027702485dcc5))
- **ai:** Agents' suggested edits wait in a review queue and write nothing ([baef162](https://github.com/iamngoni/kanso/commit/baef162acaec3d0391999b417fe9189bdbadf799))
- **ai:** Review suggested edits in Kanso and apply them once ([b9108dc](https://github.com/iamngoni/kanso/commit/b9108dcf03d471eca5ae7b82b31ce1de05f8044f))
- **ai:** Prompts for the inline assistant and a note summary that reads long notes in parts ([7e6a23a](https://github.com/iamngoni/kanso/commit/7e6a23a922650f9b4543de26afda0d797d9237f9))
- **ai:** Features ask the chosen backend with no tools and no notes access ([7704dd2](https://github.com/iamngoni/kanso/commit/7704dd2b4c7b3a007e6f170c2665cadb9347eb4f))
- **ai:** The Ask panel summarizes the open note ([14b2e41](https://github.com/iamngoni/kanso/commit/14b2e41dff5cff8ed8142c175cb84ce21895b679))
- **ai:** An inline assistant in the editor suggests a rewrite, shorter text, a fix, a continuation or a summary ([80683b0](https://github.com/iamngoni/kanso/commit/80683b025faebffe0bab278a20c6d5590cfb8bf9))
- **acp:** A permission request waits for a decision without blocking the reader ([a958a68](https://github.com/iamngoni/kanso/commit/a958a685dc6c5af633ed0cfbdb08e607ee33d74a))
- **ai:** Agents ask the app's permission decider, and reject when there is none ([ad20641](https://github.com/iamngoni/kanso/commit/ad20641095b7ddf67e62bac6b8b9390ea1f55c3f))
- **app:** A sheet asks before an agent uses one of its own tools ([3bef4b9](https://github.com/iamngoni/kanso/commit/3bef4b96be8fe9946414d5d8183ea41abbd1e3fa))
- **ai:** A diagnostics report for the agent that is safe to paste ([07a580a](https://github.com/iamngoni/kanso/commit/07a580aafd001f157ef0fa9c21877e1efa968035))
- **ai:** Copy diagnostics on the agent's Test row ([c897030](https://github.com/iamngoni/kanso/commit/c89703031cf297930393db4a6587869f4ad17896))
- **ai:** A missing custom agent points to Settings, and the not-installed lines are tested ([6ef8471](https://github.com/iamngoni/kanso/commit/6ef84712789d3e14b3f815615483a5dab59c5d00))
- **plugins:** The plugin host, manifest, grants, scoped reads, staged writes, supervisor, network proxy and registry (#37 steps 1-2, 5, 7-9) ([ce6f74c](https://github.com/iamngoni/kanso/commit/ce6f74c8a83c2e01479dfc3f3722af78d0c8a4e9))
- **plugins:** The app runs plugin commands from the palette, draws their fences and applies their batches; Settings has a Plugins page ([bf3b01c](https://github.com/iamngoni/kanso/commit/bf3b01c5c644265e963b42b35c66b191540d78f2))
- **plugins:** Settings › Plugins, the install, update, developer and review sheets, and the inspector panel, with light and dark snapshots ([9706bff](https://github.com/iamngoni/kanso/commit/9706bff58493cb4c93d795a2d921123904f05902))
- **mobile:** Flutter app scaffold with Kanso's design tokens, fonts, icons and CI ([6c27d60](https://github.com/iamngoni/kanso/commit/6c27d6049da70102134611cc872aa2f69fa535c2))
- **mobile:** The proposed mobile design as mock screens with goldens ([dfafb03](https://github.com/iamngoni/kanso/commit/dfafb03509e79b1baf447a33ab1237c5709ef361))
- **mobile:** Font choices and a polish pass over the design ([8308b2e](https://github.com/iamngoni/kanso/commit/8308b2ee300452e33692173952cf98118a872117))
- **engine:** A Markdown and code highlighter of the engine's own, checked against the desktop editor's ([24d9d4f](https://github.com/iamngoni/kanso/commit/24d9d4f8e95b70350cf18ae4e7520a77eece0091))
- **plugins:** The directory is the kanso-plugins repository, signed by its CI; checked for withdrawals and updates; the Windows and Linux updaters replace the plugin host too ([#37](https://github.com/iamngoni/kanso/pull/37)) ([6b1cacf](https://github.com/iamngoni/kanso/commit/6b1cacfdf11d261292ed39ee79a2873f984b0068))
- **render:** Draw maths and diagrams on iOS and Android, with the phones' own fonts for drawings ([fe54a7d](https://github.com/iamngoni/kanso/commit/fe54a7d318654fd1b86797621eccbfc025991480))
- **tls:** IOS and Android check certificates with the phone, through one crate that sync, shared notes and AI share ([059b37a](https://github.com/iamngoni/kanso/commit/059b37ab6be730f35a95d1e2f9e8dcc69f0c5523))
- **mobile:** The kanso-mobile facade for the phones' app ([dfd1895](https://github.com/iamngoni/kanso/commit/dfd189574e542757c97c31e3d8f108bc37b28c57))
- **mobile:** The library the app loads, and the background sync entry ([f376da1](https://github.com/iamngoni/kanso/commit/f376da1ae6ce71d83a26971c1f9abaa343b38bf4))
- **mobile:** The bridge to the library, generated, and the hook that builds it ([ecd02f2](https://github.com/iamngoni/kanso/commit/ecd02f2435680389259d772f25e739b76fec1840))
- **mobile:** Android's side of the library ([045a2c1](https://github.com/iamngoni/kanso/commit/045a2c16b4d642c463b1c2335ff1268bbcc8fd11))
- **mobile:** The app opens its library, and a core check page shows it ([29326e6](https://github.com/iamngoni/kanso/commit/29326e688ab09b3a17b593a08e3241c92c60814c))
- **mobile:** Size of the real app and its library, per platform ([d2f433f](https://github.com/iamngoni/kanso/commit/d2f433fc59d5c54969647c857809948939c670eb))
- **mobile:** Lists by place with the desktop's sort, filters and grouping, and the library's counts ([051740d](https://github.com/iamngoni/kanso/commit/051740db189e88b0b1b16f9550277b1a2eedaf41))
- **mobile:** Pin, move and trash notes, one or several at a time ([f896180](https://github.com/iamngoni/kanso/commit/f8961808ba8866df00e508c31320e8ba89b7d030))
- **mobile:** Search within titles, tags or a notebook ([d1b214f](https://github.com/iamngoni/kanso/commit/d1b214f83fdf920eebed40c21385cd0b94c155e3))
- **mobile:** Notes made on the phone get their inline tags, and notebooks can be made ([22f9161](https://github.com/iamngoni/kanso/commit/22f91613632e5aa5b0b9d998efa21bad6f1b01cb))
- **mobile:** The built-in themes, and Markdown styled in a theme ([30c145d](https://github.com/iamngoni/kanso/commit/30c145d1fa2ac10c4244b297c0be19ee7804e98b))
- **mobile:** The library is kept out of iCloud and computer backups ([98abb16](https://github.com/iamngoni/kanso/commit/98abb16a18179d290e3685b852c1e42582bb5cbb))
- **mobile:** Bridge the new library calls ([60d1848](https://github.com/iamngoni/kanso/commit/60d18481e65c639dac30d53b80a1de9ba4818c31))
- **mobile:** Colours and syntax styles come from the theme files' tokens ([2344ddb](https://github.com/iamngoni/kanso/commit/2344ddb4bcd3cc366b9a609a908b6e7f45c75d89))
- **mobile:** Appearance and list choices are saved ([5e5320c](https://github.com/iamngoni/kanso/commit/5e5320c6ab112e281788a5f9a272a9ea927b16e2))
- **mobile:** Routes for places, notes, search and settings, and kanso:// links ([9f56a43](https://github.com/iamngoni/kanso/commit/9f56a43ec75472f2b9c87f8ddaaf537c25ef092b))
- **mobile:** One, two or three panes as the window allows ([478cbb3](https://github.com/iamngoni/kanso/commit/478cbb3a96a2488e5609f92b5f8e6ad6c1c87b72))
- **mobile:** Notebooks, tags, counts, sync status and note history shared across screens ([9d3c1a4](https://github.com/iamngoni/kanso/commit/9d3c1a4cdcd091c334c4158d381672d5a39982a3))
- **mobile:** Android keeps the library out of backups and device transfers ([216ed67](https://github.com/iamngoni/kanso/commit/216ed67757fb106f7e619c89a6a28f6e499f875b))
- **mobile:** The library, from the real notebooks and tags ([3f2a933](https://github.com/iamngoni/kanso/commit/3f2a93394c02a0569ee52d57102c5cbea3c15e83))
- **mobile:** The note list grouped by date, sorted and filtered as on the desktop ([b782fe0](https://github.com/iamngoni/kanso/commit/b782fe03b4a035db10329a260b326665cd326c3c))
- **mobile:** Swipe to pin, move or trash; long-press to choose several ([ab35160](https://github.com/iamngoni/kanso/commit/ab35160a0f62517558c8151e18bda9d83cb26efe))
- **mobile:** Pull to sync while signed in ([2f685e1](https://github.com/iamngoni/kanso/commit/2f685e10ee194769544835759fa06dc634fd941e))
- **mobile:** A note's title, facts and highlighted text; locked notes show their title alone ([59ab876](https://github.com/iamngoni/kanso/commit/59ab87678434e6847f76e8ce3a77d337eb6a91de))
- **mobile:** Search with highlighted matches ([94af45b](https://github.com/iamngoni/kanso/commit/94af45b2f8fae79a7c3336921ef2182456bd764c))
- **mobile:** Settings: theme, fonts, text size, sync status, version, website and licences ([050d655](https://github.com/iamngoni/kanso/commit/050d655c8293e0c07f10d17da92f74d4a2e2c661))
- **mobile:** The theme picker, with the built-in themes and following the system ([fc85150](https://github.com/iamngoni/kanso/commit/fc851500761e8abf98a6e4fca070eed8cbd01187))
- **mobile:** Notes on this phone aren't in device backups, said once ([ff8e26f](https://github.com/iamngoni/kanso/commit/ff8e26faed3773102fa595b84c8692ab13a07996))
- **mobile:** Keyboard shortcuts for moving around on a tablet ([12b3bd9](https://github.com/iamngoni/kanso/commit/12b3bd98aa89f5f126239c134fd7bcdf0206135c))
- **site:** Colour fenced code in the demo's preview and editor with highlight.js ([44524cd](https://github.com/iamngoni/kanso/commit/44524cdd8d577848eb61a0bf790efefd874c3248))
- **sync:** Wake other devices with content-free pushes (#48 step 8b) ([7ff64bb](https://github.com/iamngoni/kanso/commit/7ff64bb7bf80773cf045a6127f2eee63f207ab92))
- **mobile:** Export compliance answer, privacy manifest and export options for iOS ([bbbff59](https://github.com/iamngoni/kanso/commit/bbbff59ea5af8340e80611a23b67cb57f05832bc))
- **mobile:** Register the phone's push token with the account's server ([c324162](https://github.com/iamngoni/kanso/commit/c324162c32f20a0fb8c8b4883dc63ca47f92c59f))
- **mobile:** Wake the phone by push to sync, with a setting to turn it off ([5a48dcd](https://github.com/iamngoni/kanso/commit/5a48dcda32ee85457858aa951d4b5ba455614c13))
- **engine:** Code block format and properties for the formatting round trips ([68c2515](https://github.com/iamngoni/kanso/commit/68c25154c299d9fa59ac2291608ceda7afe5581f))
- **editor:** Markdown formatting keyboard shortcuts in the note's text ([4118210](https://github.com/iamngoni/kanso/commit/41182108ff1a18c1de7f79ba9935d92eb813bd01))
- **cloud:** Accept crash reports from the mobile apps, keep them 90 days with no IP or account (#48 step 13) ([893d644](https://github.com/iamngoni/kanso/commit/893d64443b34610c7e173ab092db249d6ac91861))
- **site:** Release notes and Help pages, crash reports on the privacy page (#48 step 13) ([ae9ae55](https://github.com/iamngoni/kanso/commit/ae9ae5557d23942598d9dfdb9b5deb97b7c8cdaf))
- **mobile:** Render the store pictures for every required size from the golden harness (#48 step 13) ([82a6be6](https://github.com/iamngoni/kanso/commit/82a6be6096a7463519e8dd97fc38fbfa215e5667))
- **mobile:** Crash reports without an SDK or note content, release notes and help links, the Shorebird patch number (#48 step 13) ([cacba0c](https://github.com/iamngoni/kanso/commit/cacba0c86524ed53332424fd9614f2ec22d06010))
- **mobile-core:** The handles step 8 fills in, and the account's plain data ([78f2658](https://github.com/iamngoni/kanso/commit/78f2658c479c5feee1a6bc98c584025f71369ded))
- **mobile-core:** The seam a passkey's answer comes back through ([afdd3f0](https://github.com/iamngoni/kanso/commit/afdd3f0b45a3e2af084170d04a13e8a519f5584a))
- **mobile:** The phone's account flows: sign-in, providers, passkeys, locked sessions and recovery ([54290ac](https://github.com/iamngoni/kanso/commit/54290aceb5d4a68e877cca34006eca9158d4e7b7))
- **mobile:** Shared notes and the foreground sync schedule ([85bc65c](https://github.com/iamngoni/kanso/commit/85bc65c2afe15c922fba2642480f681b7c130e95))
- **mobile:** A command-line peer device and a script that runs the phone's sync flows against a local Worker ([1779418](https://github.com/iamngoni/kanso/commit/17794189443f5a8ad8b364fc1fc938d19ec85dec))
- **mobile:** The theme gallery lists, verifies and installs themes on the phone ([07ff371](https://github.com/iamngoni/kanso/commit/07ff371f30c244deb613b3d8fdd2fa35753bf736))
- **mobile:** The native side of sync sign-in is in: background sync, the system sign-in page and the passkey sheet ([30fed96](https://github.com/iamngoni/kanso/commit/30fed96b8c2414b5096ebc97257e619d9f4577a0))
- **mobile:** The account page's tasks and approving other devices from the phone ([061047e](https://github.com/iamngoni/kanso/commit/061047e2c8078cacd992a7ea88ee876d91ef897b))
- **mobile:** The account, approvals, shared notes and gallery reach Dart ([1e4bf99](https://github.com/iamngoni/kanso/commit/1e4bf9904133748220fc2b56bb72ee5a4040eb30))
- **mobile:** Shared notes and the theme gallery have their screens ([1fd25e4](https://github.com/iamngoni/kanso/commit/1fd25e49132040d6dfe7c40a578305dcaafbc764))
- **mobile:** The account screens, device approval and foreground sync reach the phone app ([20f9c8d](https://github.com/iamngoni/kanso/commit/20f9c8d2c045339df49f6b19ef6b825737e01010))
- **mobile-core:** Ticking a task, and a ticket that takes a change back ([dcbbdb7](https://github.com/iamngoni/kanso/commit/dcbbdb795a1658ee1154a404c47064f2ff12c592))
- **mobile-core:** The Tasks place lists open tasks across notes, grouped as on the desktop, with locked notes left out ([257409c](https://github.com/iamngoni/kanso/commit/257409ce5ac65120682d6ba15accb288fc8ebadd))
- **mobile-core:** A change to several notes says what it did and keeps what it needs for Undo ([8aba6eb](https://github.com/iamngoni/kanso/commit/8aba6ebe46bbcca83a68a3c74d7d26faf9fbce69))
- **mobile-core:** The bridge offers Undo and ticking a task ([141185c](https://github.com/iamngoni/kanso/commit/141185c8a5d22b4f9af3a698050280275f57d8db))
- **mobile:** A notice with Undo after a swipe or a change to several notes ([ffba51e](https://github.com/iamngoni/kanso/commit/ffba51e93b8a648b3dcc6a944d6022b3fffa1441))
- **engine:** Read icon themes in kanso-engine, as the desktop does ([9832de1](https://github.com/iamngoni/kanso/commit/9832de1c50aa62cfacf2098012cdda3a58ff1d16))
- **mobile-core:** The bridge lists icon themes and gives the icons of one ([48cfcc6](https://github.com/iamngoni/kanso/commit/48cfcc6978ccba6f5bad0f438cd6c4f4caaad05a))
- **mobile:** Choose an icon theme under Settings, Appearance, Icons ([ba5de78](https://github.com/iamngoni/kanso/commit/ba5de7892d54779da8e61c6ec55cf9d9295885fd))
- **engine:** A UTF-16 index and an undo history for editors ([ed1143a](https://github.com/iamngoni/kanso/commit/ed1143a44b15c766dbe134a9762e51fdf2529bf4))
- **engine:** Cycling headings, wiki-link brackets and the link at the caret ([8ce2f7b](https://github.com/iamngoni/kanso/commit/8ce2f7b91a54a2ecc2245fdabc8d3bd7ea8fe83f))
- **mobile-core:** An editing session for each open note ([5bc263f](https://github.com/iamngoni/kanso/commit/5bc263f3a8047e6721e4f6831d1500c2b331c064))
- **mobile:** The editing session through the bridge ([d4d7242](https://github.com/iamngoni/kanso/commit/d4d7242a110d88f19eb95f9b438746d387e832c5))
- **mobile:** Markdown typed with the Rust core's highlighting ([7d348df](https://github.com/iamngoni/kanso/commit/7d348dfa9b5fbb634f602eaf2ddab38d257553f7))
- **mobile:** Edit and Preview on a note, with autosave and the desktop's conflict choices ([220fcba](https://github.com/iamngoni/kanso/commit/220fcba860108a6afdcea16332a4b168f12558cf))
- **mobile:** The keyboard accessory bar and keyboard shortcuts in the editor ([8b817a1](https://github.com/iamngoni/kanso/commit/8b817a14e1734021fd4bde0b2f459b1db63abb5e))
- **mobile:** Completion for links to notes and tags, and opening a link from the editor ([d89dd5a](https://github.com/iamngoni/kanso/commit/d89dd5a6eaa7cc57ab0c328d24f3a90027ae102b))
- **mobile:** A note's properties above its text, its front matter out of the way ([d86df3d](https://github.com/iamngoni/kanso/commit/d86df3d7f1b7607069d691dd0aee8c981c141e3b))
- **mobile:** Long notes are edited a window at a time ([cc19ba8](https://github.com/iamngoni/kanso/commit/cc19ba86465db04a40044235b4d45da8a285c2dd))
- **mobile-core:** New notes take their place and the preferred notebook ([59947e4](https://github.com/iamngoni/kanso/commit/59947e40f76319fd9729d7ffe5872edaf85a01f1))
- **mobile:** New notes from the list's + button and ⌘N, opened ready to write ([426593a](https://github.com/iamngoni/kanso/commit/426593ab2f1d0c8b15e88b48e3460b46be3de33f))
- **mobile:** New notebook from the library ([d6d323c](https://github.com/iamngoni/kanso/commit/d6d323c92f3cfc35aeaf0b35ba95451294a86fe3))
- **mobile:** Settings › General › Default notebook ([8eb3852](https://github.com/iamngoni/kanso/commit/8eb3852d5cdc3484567c7f08655b67e565999030))
- **mobile:** A note's preview is its page, and a tap in it does what the desktop does ([9f7de64](https://github.com/iamngoni/kanso/commit/9f7de649dc38ba83654f08773f3fa2c3c10ad562))
- **mobile:** Accounts, sync, shared notes and comments on the phones (#48 step 8) ([0389fed](https://github.com/iamngoni/kanso/commit/0389fed8ccd4058193da1912b401a326e66e9390))
- **mobile:** A more polished, less stock look ([dc3e052](https://github.com/iamngoni/kanso/commit/dc3e0529516a655131a802198b87dba566324e11))
- **core:** PDFs can be attachments ([e7250ac](https://github.com/iamngoni/kanso/commit/e7250ac0f900f0ac530ad7e3f3911da89c79299e))
- **mobile-core:** Shared items are saved from the inbox's drops (#48 step 10) ([0d0cfee](https://github.com/iamngoni/kanso/commit/0d0cfee5697364727edb60e52531b336e954e159))
- **mobile:** Share to Kanso, a widget and Shortcuts on iOS (#48 step 10) ([d5573d2](https://github.com/iamngoni/kanso/commit/d5573d25d0e5192597c2026ebdcf3e4e029e0355))
- **mobile:** Share to Kanso, a widget and shortcuts on Android (#48 step 10) ([3ecf1be](https://github.com/iamngoni/kanso/commit/3ecf1be75106993d93ac6c73c94fcf7fc372cc73))
- **mobile:** The share extension's privacy manifest ([452dcfc](https://github.com/iamngoni/kanso/commit/452dcfcd985d6313ea4f91d946965ef33f057709))
- **mobile-core:** Today's note and in-app captures through the facade ([54f18bf](https://github.com/iamngoni/kanso/commit/54f18bf4ae537ebf989aa173836665a31a191c3a))
- **mobile:** The library in the App Group, shared items saved on resume, today's note (#48 step 10) ([163ded5](https://github.com/iamngoni/kanso/commit/163ded5fae084960f1465b4998938d1510ebe457))
- **mobile:** Attach a photo or a PDF from the editor (#48 step 10) ([2bc77fe](https://github.com/iamngoni/kanso/commit/2bc77fe54999061f734d3ab8b7b49d1393491110))
- **mobile:** Capture a photo, photos or a PDF from the add button and the editor (#48 step 10) ([7fe69fe](https://github.com/iamngoni/kanso/commit/7fe69fe0af756ac8d17f9cbb9fe8b0f10aa5b5bc))
- **mobile:** Lock notes and notebooks the desktop's ways, and unlock with biometrics ([f7b9d3c](https://github.com/iamngoni/kanso/commit/f7b9d3c93a24c07dbb8a3d42ccb7a816bdc7a17c))
- **mobile:** A lock plugin for biometric unlock and the privacy cover ([5c5abbd](https://github.com/iamngoni/kanso/commit/5c5abbdbe78a3a089dfcfa5366e18fcb362d9950))
- **mobile:** Wire the lock plugin into the app, and use a FragmentActivity for BiometricPrompt ([87ce69f](https://github.com/iamngoni/kanso/commit/87ce69f7afa24dcfb40805a8f1c767c902b61041))
- **mobile:** Bridge the lock calls and the biometric answer entry ([9f868e5](https://github.com/iamngoni/kanso/commit/9f868e5c18aee80d36b380bf89cda8aed033d7fd))
- **mobile:** The lock models, repository and BLoCs for locked notes ([727c8d0](https://github.com/iamngoni/kanso/commit/727c8d06d8fe8bb09955d74e687e2d32555a8674))
- **mobile:** The lock sheets, the note menu and the notebook lock actions ([4e1a23f](https://github.com/iamngoni/kanso/commit/4e1a23fca7497a22ceecab22bff42325487b24e8))
- **mobile:** Unlock the locked note, lock from the menus, Settings section, lifecycle and cover ([202e813](https://github.com/iamngoni/kanso/commit/202e813fea325969b182eccba6aa2b5592e1001c))
- **mobile:** Import and open Markdown notes ([8201e8a](https://github.com/iamngoni/kanso/commit/8201e8ae11c94f7ace4ae64718843318aef1d551))
- **editor:** Tick a task in the preview, and offer the gpui-base hook upstream ([466283c](https://github.com/iamngoni/kanso/commit/466283cf3866da26bffa8ee3c03c568958066fb9))
- Add MCP tools for templates and native note workflows ([ae4ce32](https://github.com/iamngoni/kanso/commit/ae4ce3295ce581b60de6f180fa2ff8984937c752))
- **engine:** Version history diff, Replace words and the backup schedule move into the engine (#48 parity D, H) ([a296489](https://github.com/iamngoni/kanso/commit/a2964895f2c03307299c65f7b4b91b06c5e9350d))
- **mobile:** History, Replace and backups in the bridge, and Undo says what it put back (#48 parity D, H) ([4bd0dff](https://github.com/iamngoni/kanso/commit/4bd0dffa563ef533a192c24c5ac08d7646d7ab31))
- **mobile:** Add history, replace, and backup foundations ([ce89623](https://github.com/iamngoni/kanso/commit/ce89623160a7c3910c8161350345d52e155af4fc))
- **mobile:** Merge locked notes and capture parity ([8a20546](https://github.com/iamngoni/kanso/commit/8a20546302e49f6b1e1ff9d691d41cd4167b881f))
- **mobile:** Add history, Replace and daily backups ([10e2627](https://github.com/iamngoni/kanso/commit/10e262704197dc9267b526f3b04dd3332f94420e))
- **mobile:** The shared note drag payload from package Q ([#48](https://github.com/iamngoni/kanso/pull/48)) ([2fa7312](https://github.com/iamngoni/kanso/commit/2fa731234caaf7d57b7f21f131328a87362b45e0))
- **mobile:** Bundle the Lucide icons a notebook can show ([#48](https://github.com/iamngoni/kanso/pull/48)) ([8f93f7d](https://github.com/iamngoni/kanso/commit/8f93f7d85a88b37860345e2fb3b6ec2a5841fb54))
- **mobile:** Manage notebooks and tags, and the notes' extra actions ([#48](https://github.com/iamngoni/kanso/pull/48)) ([fee0ee3](https://github.com/iamngoni/kanso/commit/fee0ee3ef0f72d311aa9f566962ed487233a25b8))
- **mobile:** A plugin that prints a page to a paged PDF (#48 package I) ([ed7a2e5](https://github.com/iamngoni/kanso/commit/ed7a2e54a670569920c581dc9b6ec712db044fc3))
- **engine:** Export, publish and the page's code colors move to the engine (#48 package I) ([6b6e757](https://github.com/iamngoni/kanso/commit/6b6e757db60e7ac8674d5b3bb3706f9d7934b89d))
- **mobile:** Write a note as a PDF, in the theme it is read in ([750ac73](https://github.com/iamngoni/kanso/commit/750ac734b2b7764613b25cc9d58845130e5e8959))
- Expand mobile app features and shared engine support ([9ebb260](https://github.com/iamngoni/kanso/commit/9ebb260415ae8f4ef9bb20718d7f5b8f20c34265))
- **mobile:** The drag payload and the note window seam that package Q's parts share (#48 package Q) ([e5dd16c](https://github.com/iamngoni/kanso/commit/e5dd16c5bf0f4352172a6f4f66d78c917336b6e9))
- Add cross-platform hardware keyboard shortcuts and catalog ([c612fef](https://github.com/iamngoni/kanso/commit/c612fef37712b68eff6f5b502ad813df07a58ec4))
- Add tablet note tabs and navigation controls ([d465e3d](https://github.com/iamngoni/kanso/commit/d465e3de5a737841588958cd982aef4487aba229))
- Add multi-window note editing across platforms ([a664c5f](https://github.com/iamngoni/kanso/commit/a664c5fedaa6b6884f64818c8eddac00f7eb1e8f))
- Add note drag-and-drop to notebooks, tags, and tabs ([a058dad](https://github.com/iamngoni/kanso/commit/a058dad97e8eab4c4ed8a15ead643ae08e0d7083))
- **discovery:** Read notes by meaning and links on the phones too ([f7979ee](https://github.com/iamngoni/kanso/commit/f7979eed248d6c5432837b8fa0fee7482a13ebe1))
- Add local discovery, note suggestions, and graph settings ([23eff8a](https://github.com/iamngoni/kanso/commit/23eff8a00318eeb38c6a2f9a9a1f1d838a41a005))
- Add note discovery insights and actions to info sheet ([544c23e](https://github.com/iamngoni/kanso/commit/544c23e1b21df4801a58d4a15fd9b3a35dcd3591))
- Add interactive note graph with filters and local views ([fa5065f](https://github.com/iamngoni/kanso/commit/fa5065f0fb3a6085fb482b0f8fc11c0d7e4eaaf1))
- Add semantic matches to search results ([d738ca3](https://github.com/iamngoni/kanso/commit/d738ca3b558a73c156bbbe29feff058b4127818b))
- **mobile:** Reminders as local notifications on iOS and Android (#48 F+G) ([4dc2eae](https://github.com/iamngoni/kanso/commit/4dc2eaef8dbba0024768d1d57fae9d12e8bf82ee))
- Add comprehensive mobile app features and engine support ([7b92291](https://github.com/iamngoni/kanso/commit/7b92291bf46e1e6bd26585626a5ce920e069f979))
- Add mobile library management and PDF export, extract shared engine features ([c23fefb](https://github.com/iamngoni/kanso/commit/c23fefb01f8107bfa94566e2e53e74fe320d1b70))
- Add journal, board, templates, and task reminders ([c0591b8](https://github.com/iamngoni/kanso/commit/c0591b89833431de8afbdcd1f657a7878a36b036))
- Add task boards, journal actions, and reminders ([352ae3f](https://github.com/iamngoni/kanso/commit/352ae3ff4858146e28d1409c64f6f31f69697d4e))
- **mobile:** Word board due chips in the bridge, and finish parity F+G tests ([b49ac6b](https://github.com/iamngoni/kanso/commit/b49ac6b72bec1d2622ddb19475aa4b0d08834725))
- **mobile:** Drive keyboard shortcuts from the desktop catalog ([5069e3e](https://github.com/iamngoni/kanso/commit/5069e3eb2fb2e1897c0060294e897e7b30e8326c))
- **mobile:** Command palette, outline and backlinks ([#133](https://github.com/iamngoni/kanso/pull/133)) ([21e425d](https://github.com/iamngoni/kanso/commit/21e425d613d5400b25e8c72a5d622f1a2f9aca3a))
- **mobile:** Focus mode ([#135](https://github.com/iamngoni/kanso/pull/135)) ([ae33e5f](https://github.com/iamngoni/kanso/commit/ae33e5f56b3c8ae5917f063abbefb4d9464ab3d8))
- **mobile:** Palette acts on the open note, and six more rows ([#138](https://github.com/iamngoni/kanso/pull/138)) ([0c511ff](https://github.com/iamngoni/kanso/commit/0c511ff5ee815735362797ce6a79c7134d3c81e2))
- **mobile:** Add Ask, writing assistance, and dictation ([#137](https://github.com/iamngoni/kanso/pull/137)) ([61e4273](https://github.com/iamngoni/kanso/commit/61e427370dc2f6345cbaa7e58f00dec3eafb7e9c))
- **mobile:** Add palette note mode commands ([#140](https://github.com/iamngoni/kanso/pull/140)) ([09a8450](https://github.com/iamngoni/kanso/commit/09a84508577710a68c260baf37294ab8bd4f22de))
- Add direct assistant actions to the command palette, editor, and notes ([#141](https://github.com/iamngoni/kanso/pull/141)) ([d3eeacd](https://github.com/iamngoni/kanso/commit/d3eeacd4b8a1852edc6d03c45cb76716288a03a7))
- Complete mobile word-goal and appearance parity ([#142](https://github.com/iamngoni/kanso/pull/142)) ([66a2ac8](https://github.com/iamngoni/kanso/commit/66a2ac86964a99dd56a4016e3f8d3470b4dca7c4))
- Seal notebook attachments with note locks, preserve shared ownership, and rekey file keys safely ([#146](https://github.com/iamngoni/kanso/pull/146)) ([0a894af](https://github.com/iamngoni/kanso/commit/0a894afdd5b89bd3af09f628f5e0a8216fcd41e9))
- Seal attachment names and types (#44 step 6) ([#150](https://github.com/iamngoni/kanso/pull/150)) ([ceb1f7d](https://github.com/iamngoni/kanso/commit/ceb1f7d356ce7602b37531a167b60155ecb9bf23))
- Allow any non-empty password for sign-in and unlock, expose new-password checks ([f5a2201](https://github.com/iamngoni/kanso/commit/f5a220189a1f5af618b24df2be1acc610b7c203a))
- Add downloadable Google Fonts and preview styling ([df10128](https://github.com/iamngoni/kanso/commit/df1012854b0d3e1a84be0a10702f5ba9278bf8f3))
- Add pannable, zoomable graph viewport with fit control ([9779a88](https://github.com/iamngoni/kanso/commit/9779a88eafc9070b37506813203341d87e200718))
- Add custom dialogs and improve bottom sheets ([275c537](https://github.com/iamngoni/kanso/commit/275c53724c62280c7d6dbe3de138bf10e60026b0))
- Add app-wide feedback and scaffold-free pages ([6926bb0](https://github.com/iamngoni/kanso/commit/6926bb0ca99ad0cdaee1eda0e1f207e582e3e532))
- Support continuous dictation with live transcript revisions ([4f10407](https://github.com/iamngoni/kanso/commit/4f1040772771ff3cff6bd72ec995bfe6dbc69629))
- Add platform-aware page transitions and swipe back support ([deda86f](https://github.com/iamngoni/kanso/commit/deda86f2b462d54d1d33dac622065347112c22c3))
- Add continuous note text size controls ([59fe726](https://github.com/iamngoni/kanso/commit/59fe7264b3309a045764888bfa32ef29546a452d))
- Add iOS 26 streaming dictation and refresh mobile theming/UI ([5f64e65](https://github.com/iamngoni/kanso/commit/5f64e65a6d9d5b280e5be368f7d6b41306ea6b73))
- **render:** Typed preview blocks for a native note preview ([d42cca3](https://github.com/iamngoni/kanso/commit/d42cca3b8c8d3a274936baf0ff4f42c7048f9cd6)) by Claude
- **mobile:** Draw the note preview natively ([1f251a9](https://github.com/iamngoni/kanso/commit/1f251a9006a50358db756acbac3a641c0361bf07)) by Claude
- **engine:** Work out the marks Live mode hides and the headings it scales ([3953b8e](https://github.com/iamngoni/kanso/commit/3953b8ea3605c630f5406893d6c0bb530bd63b79)) by Claude
- **editor:** Let the editor hide ranges and draw lines taller ([f9862ee](https://github.com/iamngoni/kanso/commit/f9862ee8b2cbe2515d5b2a87174e39ae82b00990)) by Claude
- **mobile:** Add Live mode beside Edit and Preview ([36a040a](https://github.com/iamngoni/kanso/commit/36a040af31ade5c8300658ff6bb03e6151495d61)) by Claude
- **desktop:** Add Live mode to the editor ([dc7861c](https://github.com/iamngoni/kanso/commit/dc7861c367c2dc0f2b4b2369c0700869a5bc0ecd)) by Claude
- Offer Live Preview in focus mode ([de56348](https://github.com/iamngoni/kanso/commit/de563482e55bef8659f11628490a50a133a0f09c)) by Claude
- **editor:** Draw replacements, line spacing, ornaments and block widgets ([2fa393f](https://github.com/iamngoni/kanso/commit/2fa393ff2ecd180502836a4b4a93d4754191d75e)) by Claude
- **engine:** Lay out Live as Typora does ([6a6da7a](https://github.com/iamngoni/kanso/commit/6a6da7a4b7045c659f8c090857fe0a3f8b4d5249)) by Claude
- **desktop:** Draw Live like the preview ([d6d3a0a](https://github.com/iamngoni/kanso/commit/d6d3a0a15b42d5ff8330a425070e67933a53a2b8)) by Claude
- **mobile:** Give the editor Live's whole-note layout and the preview's blocks ([e0455ee](https://github.com/iamngoni/kanso/commit/e0455ee2d92b6ce7d17a5f03486b2397c15e142d)) by Claude
- **mobile:** Draw Live as the preview draws the note ([0749eb5](https://github.com/iamngoni/kanso/commit/0749eb5437d0669b0612ff1b5fe0062d12e2e731)) by Claude
- **metrics:** Accept anonymous usage counts and pass them to PostHog ([8db146e](https://github.com/iamngoni/kanso/commit/8db146e46f4e67a13d905ed79b41ad5a2e50ab7d)) by Claude
- **metrics:** Count daily use on the desktop ([6417cc4](https://github.com/iamngoni/kanso/commit/6417cc4c8ea5a07559dd96f436bf8946dda92a98)) by Claude
- Add anonymous daily mobile usage metrics ([1120d7f](https://github.com/iamngoni/kanso/commit/1120d7f76f9ec3082556ea52b896b84e88a80193))

### Improved

- **engine:** A syntax error's parse is made for the range asked for, not to the end of the note ([7ce051b](https://github.com/iamngoni/kanso/commit/7ce051be1c6e24c77acdfd3ccdccb4fed50436a3))
- **mobile:** Colours that come back for a text a few keys old are shown, moved along the keys ([f5386ff](https://github.com/iamngoni/kanso/commit/f5386ff2b5987d5826563511e1673b90de11c630))
- **engine:** A range that is a small part of an error's parse reads only what touches it ([7aa3d04](https://github.com/iamngoni/kanso/commit/7aa3d0440f51e235e1df30fb1d357f54787c6484))
- **mobile:** The bridge's modules and type imports as rustfmt orders them ([9225594](https://github.com/iamngoni/kanso/commit/9225594beb896c82a65e826307cfb4a0bd85e608))
- **mobile:** The merged modules in rustfmt's order ([b80ad44](https://github.com/iamngoni/kanso/commit/b80ad4426ff16bf9ed4864e1ff79a8121427e428))
- **mobile:** The merged modules in rustfmt's order ([3af77a3](https://github.com/iamngoni/kanso/commit/3af77a3eb2ec6855ff0f81fdc9cb96bbe077a2dd))

### Fixed

- **locked:** Lock a note from its right-click and More menus ([23892fa](https://github.com/iamngoni/kanso/commit/23892fac61e56b5f4d6fa4a19912728b15c0ad10))
- **locked:** Show when locked notes are unlocked, lock them in one click, and auto-lock 10 minutes after unlocking ([9bda4ca](https://github.com/iamngoni/kanso/commit/9bda4ca393a81cd6c6550d38efe6e7453aefef34))
- **properties:** Keep comments, blank lines and unreadable values when editing ([8ba865e](https://github.com/iamngoni/kanso/commit/8ba865ebe2af3a132f918bccce523dd346dc924a))
- **query:** Sort built-in dates by instant, tie-break by ID, and read less ([cd4f393](https://github.com/iamngoni/kanso/commit/cd4f39321717fc2e23cd9f1bf241b6dc8bd52523))
- **app:** Keep query tables on screen while they refresh ([448b945](https://github.com/iamngoni/kanso/commit/448b9459339ad4a6032fe195aa8c2569329a8f3c))
- **gallery:** Name color files where the repository keeps them ([70a42d5](https://github.com/iamngoni/kanso/commit/70a42d55eb44ecd29b797093ed5b2529c260a0bc))
- **settings:** Keep the light/dark switch above the theme grid ([30ee915](https://github.com/iamngoni/kanso/commit/30ee91582306098a6438270fa76c2fbc7a3cce89))
- **core:** Keep property comments and stop prose reading as front matter ([3182a9d](https://github.com/iamngoni/kanso/commit/3182a9d038a0d0e27bd3fe7ec11820a224e20f0d))
- **inspector:** Edit the property row that was clicked ([7a80986](https://github.com/iamngoni/kanso/commit/7a80986269133c2a12b78e867ff9174e925f3e10))
- **export:** Keep both values when user front matter mirrors Kanso's ([251869c](https://github.com/iamngoni/kanso/commit/251869c2a67c8663a3e61d10da1c7df98b1df501))
- **query:** Keep front matter out of query fences, and skip needless parses ([b2930a3](https://github.com/iamngoni/kanso/commit/b2930a3107a17255c67b2965add57005943980f5))
- **query:** Refresh tables that are already on screen ([4e83f21](https://github.com/iamngoni/kanso/commit/4e83f210ff2624d81b8901d060866d0d8950344c))
- **query:** Keep query view state per store ([fff9e2b](https://github.com/iamngoni/kanso/commit/fff9e2b55ed6731c9629e93b3b78bfba02bca7a1))
- **import:** Keep the user's own title, status and pinned beside Kanso's ([616128b](https://github.com/iamngoni/kanso/commit/616128bacd86ff39308d4fd24db85a705a2fb73e))
- **notes:** Command-click again chooses several rows; middle click opens a tab ([e03f385](https://github.com/iamngoni/kanso/commit/e03f38540355a63575161ec4894f6b663f57939b))
- **tabs:** Save the open note before Command-W closes the last tab ([9e5b707](https://github.com/iamngoni/kanso/commit/9e5b707fc2e8dfa293de1312dece6af0ee26ef9d))
- **tabs:** Open a note before a menu command acts on it ([2c07b2a](https://github.com/iamngoni/kanso/commit/2c07b2a6f163dbcab976c9757cd88db19bb29d58))
- **tabs:** Go back and forward in the current tab, and keep the bar honest ([d0b2e1e](https://github.com/iamngoni/kanso/commit/d0b2e1eced0f9add93d1d0b042b6d8c024e95d13))
- **tabs:** A notebook pick with two tabs open gets its own tab ([3b50d9c](https://github.com/iamngoni/kanso/commit/3b50d9c37e97c5aa32f7c82fa142f30614d6649e))
- **tabs:** Scroll the tab row and offer every tab when it runs past ([4e8d43c](https://github.com/iamngoni/kanso/commit/4e8d43c013c5a3a4c75110fc6f0fef07d8b41327))
- **tabs:** Re-check a kept tab once the editor's unsaved text settles ([756c588](https://github.com/iamngoni/kanso/commit/756c58835c056b693d04617f6062d6e7df2c4b41))
- **tabs:** Leave the tabs alone when a notebook click picks a note ([db81825](https://github.com/iamngoni/kanso/commit/db818257def5450b1bcd676eddc4f3a24192a912))
- **gallery:** Check every redirect with the gallery's own fetch policy ([b1ed2fc](https://github.com/iamngoni/kanso/commit/b1ed2fc5792d4d29905538ff6e37f5cf2fdd5c37))
- **gallery:** Ship the themes public key and prove the key lists are disjoint ([f626674](https://github.com/iamngoni/kanso/commit/f62667484fd3fa2f7457cc5775b00c1018ffde32))
- **gallery:** Never let a stale index lower the stored serial ([81e4ef5](https://github.com/iamngoni/kanso/commit/81e4ef5b655ebcd73b9b5e026082b38f4271ead7))
- **gallery:** Refuse a use target that contains another use ([c6722c2](https://github.com/iamngoni/kanso/commit/c6722c2a316abc7c559fa228a00feb2dff46a0f3))
- **gallery:** Cap icon bundles while unpacking them ([fdfcf7a](https://github.com/iamngoni/kanso/commit/fdfcf7a64840838caa7cceba0412f6150e073d33))
- **ask:** Harden answers, retrieval and conversation history ([2e5663e](https://github.com/iamngoni/kanso/commit/2e5663e0d73c07ba58a48b1eb62ace5095bab672))
- **ask:** Neutralise only the HTML and images the panel would render, and tie each turn to every note it read ([405a6e0](https://github.com/iamngoni/kanso/commit/405a6e0534466568814687317936becef8484f33))
- **gallery:** Refuse href with xlink:href, and keep an icon update when its backup won't delete ([20f3a15](https://github.com/iamngoni/kanso/commit/20f3a150b026165a02efada676b244240537018b))
- **gallery:** Name color files where the repository keeps them ([83c959e](https://github.com/iamngoni/kanso/commit/83c959efab8cbb45592970cd2c9987eb1a95f4bc))
- **gallery:** Name the cached signature index.json.sig ([b6c7c06](https://github.com/iamngoni/kanso/commit/b6c7c064f71e56eee2f2417647791f909e269757))
- **gallery:** Refuse to overwrite edits and validate an icon theme before swapping it ([1ba03f1](https://github.com/iamngoni/kanso/commit/1ba03f13a0097f2dde1df37cfc3bf19e52bb3036))
- **gallery:** Check every fetched file, apply revocations with auto-update off, and recover from a 304 ([d3875da](https://github.com/iamngoni/kanso/commit/d3875da0b7f969e56e795f2941cf32a042ee3432))
- **gallery:** Serialize installs so a pair keeps both state records ([be87714](https://github.com/iamngoni/kanso/commit/be877144975ba75a4e1a60af9c3be9ac5041fabb))
- **gallery:** Announce a disabled theme once, not at every check ([a9c4623](https://github.com/iamngoni/kanso/commit/a9c4623ed3e10c6f3bd46a7652b59226c61f7ad7))
- **gallery:** Name the cached signature index.json.sig ([6eb3c33](https://github.com/iamngoni/kanso/commit/6eb3c3369e36c2fa47bc262c013bf49abc77c416))
- Encrypt GitHub device codes and reject expired flows ([06acbde](https://github.com/iamngoni/kanso/commit/06acbde28524c3d317c32f39ad72429ee4e87766))
- **sync:** Close the gaps the review of #56 found in sign-in ([f183286](https://github.com/iamngoni/kanso/commit/f183286ba09f03dd199ef3279fe2fbe7092abcf2))
- Secure provider flow completion and retry handling ([19a98dd](https://github.com/iamngoni/kanso/commit/19a98dd948f954d232ff555e40c54d300f2004b4))
- Add reauthentication reason to API errors ([219464a](https://github.com/iamngoni/kanso/commit/219464adfb8eb272214ad864592c07a43767ce9d))
- Enable hmac-secret during credential creation ([e22d49d](https://github.com/iamngoni/kanso/commit/e22d49d0cc6b8e9d54502648c71aa7c8b7ceca0b))
- Fix passkey authentication and clear credentials on recovery ([16e5621](https://github.com/iamngoni/kanso/commit/16e562148505ce8a4c1030dbab3837e5910a2f3b))
- Allow apt downloads without root privileges ([71531fb](https://github.com/iamngoni/kanso/commit/71531fbef10eeb564a077e1b4d114f6dff53b2ce))
- Return 200 for passkey sign-in and validate attestation statements ([229dee4](https://github.com/iamngoni/kanso/commit/229dee40aa50dd54f1e30d6bdb6507402a6e6307))
- Align memory passkey errors with Worker behavior ([2557da0](https://github.com/iamngoni/kanso/commit/2557da09162d780d2a8840ce0b07dcb7d54c59cf))
- Add user-friendly sign-in error messages ([7740d46](https://github.com/iamngoni/kanso/commit/7740d464a568c24ceb98cba746a15a6eff9dd3f7))
- Encode public keys as base64url in approval transcript ([866f7d7](https://github.com/iamngoni/kanso/commit/866f7d7aba97af134661212feb20bdc6508ec68a))
- **sign-in:** Pin the approval commitment and let the approver approve ([379e7cd](https://github.com/iamngoni/kanso/commit/379e7cd960f3983e6c77224543141806836177f1))
- **sign-in:** Keep a revoked session revoked when a provider flow is polled again ([1ef89b7](https://github.com/iamngoni/kanso/commit/1ef89b72de4fe9812c414e9ca76fec70af9c65cc))
- **sign-in:** Bind Sign in with Apple to the app that finished it ([93e1377](https://github.com/iamngoni/kanso/commit/93e1377c13a0bcd4e96878bd61844900bc80c43d))
- **sign-in:** Let only an approval's two devices cancel it, and count answers to real requests ([ed84644](https://github.com/iamngoni/kanso/commit/ed84644877b28e045f55115372575bb15fdead7d))
- **sign-in:** Close account-event sockets when their session ends ([791330c](https://github.com/iamngoni/kanso/commit/791330ce80ab1bed038a8a4081893968775b0d4a))
- **sign-in:** Keep a signed-out passkey sign-up from coming back on replay ([b3110cf](https://github.com/iamngoni/kanso/commit/b3110cf646612fb13bbd8e0a1a214ca590a68149))
- **passkeys:** Store only known transport hints so smart-card can't fail sign-up ([2876c4d](https://github.com/iamngoni/kanso/commit/2876c4d87526acb5e21558360fa7811dd3c668ab))
- **webauthn:** Keep the Windows PRF salt in a Box for the call and read pHmacSecret by version ([30f1c66](https://github.com/iamngoni/kanso/commit/30f1c6679033600ca50b74a5aefb90940e7abbe5))
- **webauthn:** Use the real macOS PRF classes and selectors, and stop leaking ceremony objects ([6b73b92](https://github.com/iamngoni/kanso/commit/6b73b92a77a7f1bf5edc634aecddbd9fbead681a))
- **sign-in:** Use the entitled relying party only from the owner's server ([8f249c9](https://github.com/iamngoni/kanso/commit/8f249c95fd79bc0fdfc20398b806d2246ed60a52))
- **sync:** Run a passkey ceremony only for a relying party the server owns ([9b598f7](https://github.com/iamngoni/kanso/commit/9b598f73bf6a4b7f4252d0c32446ceb67bb6b5fc))
- **sync:** Refuse a provider start whose page isn't the provider's or the server's ([5ddef81](https://github.com/iamngoni/kanso/commit/5ddef81c209d1c0fa352f31347aa09d99df87ed4))
- **sign-in:** Hold one relying-party rule, kanso-sync's, and check the origin before a ceremony ([570f6e4](https://github.com/iamngoni/kanso/commit/570f6e4f487c490367a7b303314085761c5d74e3))
- **app:** Import the relying-party helpers only where the tests use them ([3a6a1b8](https://github.com/iamngoni/kanso/commit/3a6a1b854920e89631ebe381d0778b487979e2b7))
- **sign-in:** Open only provider pages, checked twice, in the system browser ([2778781](https://github.com/iamngoni/kanso/commit/2778781b21e0ea0791d7a08c68f6712643a7b752))
- **passkeys:** Start the macOS sheet and return, and run Windows' on the account thread ([6c1333a](https://github.com/iamngoni/kanso/commit/6c1333ad833f503ce05c3fb420f5f6179dac5f3f))
- **sync:** Refuse a passkey sign-in answered for another account than the passkey's ([aa16055](https://github.com/iamngoni/kanso/commit/aa16055100179ded02f59fd45a903241117261ac))
- **sync:** Make MemoryServer's passkey checks the Worker's ([60a1780](https://github.com/iamngoni/kanso/commit/60a1780d51ef324fe3452ce9a9b3b2630e5cefe4))
- **sync:** Wipe a security key's hmac-secret output when it goes ([bd8bcfe](https://github.com/iamngoni/kanso/commit/bd8bcfe5df3185be201010d7afdc79a99dbc8a6b))
- **sync:** Keep why a security key's registration got no PRF ([a7e95ae](https://github.com/iamngoni/kanso/commit/a7e95aee5229bfa7d34b698f953847455c5493bd))
- **sync:** Count a passkey removal whose answer was lost as done ([1ba1ebf](https://github.com/iamngoni/kanso/commit/1ba1ebfd8429c6acc4cc31e5ec9ffacae628c890))
- **sync:** Refuse a provider start whose page isn't the provider's or the server's ([34aa100](https://github.com/iamngoni/kanso/commit/34aa100d520571dbc4a09f2dfa83df93eeab378a))
- **fido:** Keep the PIN out of the requests' Debug ([e6ce952](https://github.com/iamngoni/kanso/commit/e6ce9525ec677d82d30a71e77eec0b45292f10e6))
- **app:** Serve each callback forward on its own thread with three seconds in all, and draw its tokens from the system ([cceb413](https://github.com/iamngoni/kanso/commit/cceb413f80085f5b28038bf8fa8e2f86cccb73d3))
- **app:** Take a kanso:// link only when it is the whole command line ([09d5d89](https://github.com/iamngoni/kanso/commit/09d5d892d72f4fd2c2ef35d7ab604ddc5dfb0adc))
- **sync:** Count MemoryServer's proof and password limits per device, then per account ([4045de9](https://github.com/iamngoni/kanso/commit/4045de9335eb1aca6c4c8b26bdb04fcfcf547eb3))
- **sync:** Refuse a locked passkey sign-in that nothing ties to the passkey ([4bfaee9](https://github.com/iamngoni/kanso/commit/4bfaee9298825cfe6396b914e9b11f11341de6a9))
- **sync:** Check the page a link or provider-proof start names ([ec84ec5](https://github.com/iamngoni/kanso/commit/ec84ec5adec56991413853f728066fd4d5e7acab))
- **sync:** Open a provider page only as it was parsed ([5a119f7](https://github.com/iamngoni/kanso/commit/5a119f791abf2ff82b675f1ee798d6402fc7816a))
- **app:** Check the Apple sheet's URL with the system opener's web-page rule ([a5102a9](https://github.com/iamngoni/kanso/commit/a5102a9cc9fb1c22a1238d981890cc303a55e5cb))
- **app:** Refuse a passkey relying party that is a public suffix ([b1d9cd1](https://github.com/iamngoni/kanso/commit/b1d9cd1fde4727edb75b4433ddc1debeaa37e543))
- **fido:** Tell a wrong PIN, with the tries left, from a missing one, and say when the PIN is blocked ([0555cb6](https://github.com/iamngoni/kanso/commit/0555cb6b590321ed8a9a881500e543fa6a219e1a))
- **sync:** Complete a MemoryServer provider link only while its session lives ([1d2d6a0](https://github.com/iamngoni/kanso/commit/1d2d6a0659b953f402fc82ffe02c531916dfd2f2))
- **sync:** Bind a MemoryServer provider proof to the device that started it ([3605771](https://github.com/iamngoni/kanso/commit/36057713e6dc3b4fc212e8159c582c475d1c40f5))
- **sync:** Count only offered ways in for MemoryServer's last-method rule ([63fe416](https://github.com/iamngoni/kanso/commit/63fe416a9a2d3db3f28a2dbea7f4d882b4ca69f4))
- **sync:** End a MemoryServer session's flows with it, as migration 0010 does ([50c58cc](https://github.com/iamngoni/kanso/commit/50c58ccc660a5ac22cefe516435b2680d0363e67))
- **sign-in:** Answer a password change on a passwordless account with 409 no_password ([fe24492](https://github.com/iamngoni/kanso/commit/fe244926fd2e0bf48c05117523c9cc4337714f67))
- **sign-in:** Mark a recovered account as having a password ([50908b7](https://github.com/iamngoni/kanso/commit/50908b748bbb5f2e0bdaa3c6e91d7f59ee187b72))
- **sign-in:** End a provider link with the session that started it ([dda1e94](https://github.com/iamngoni/kanso/commit/dda1e940ac0f77deb2515060f15b8316cf3df9fb))
- **sign-in:** Bind a provider proof to the session's device and its proof ([de16eae](https://github.com/iamngoni/kanso/commit/de16eae515cfc410df6bb2d9b4d03ae676f247a7))
- **sign-in:** Count proof and password limits per device before the account ([311ef64](https://github.com/iamngoni/kanso/commit/311ef644ba125601cdb13fc225fe44caffdb02b9))
- **sign-in:** Refuse a sign count that drops to 0 and use up its challenge ([b2c702a](https://github.com/iamngoni/kanso/commit/b2c702a3a54da975bdca5f0659647cdf8f4b2d0f))
- **sign-in:** Count only the ways in this server offers for the last-method rule ([10d309e](https://github.com/iamngoni/kanso/commit/10d309e2addd86d8a27e81532d48b1ecceb438b7))
- **sync:** Keep the device a passkey or provider sign-in bound the session to ([35a79f0](https://github.com/iamngoni/kanso/commit/35a79f06d791dadc83f66f52c695c304f96c6464))
- **account:** Count ways in as the Worker does, ask again on last_method, and blame the right secret ([0c01d73](https://github.com/iamngoni/kanso/commit/0c01d73b95696d9d8389f26412d24bec2a0e8616))
- **account:** Lay out the sign-in methods as one column and offer the username form as a method ([1010c2c](https://github.com/iamngoni/kanso/commit/1010c2c5d291287792f7971856e8c9a9ffff76bc))
- **passkeys:** Name a passkey's manager, don't offer a second iCloud Keychain passkey on a Mac ([6e9d43e](https://github.com/iamngoni/kanso/commit/6e9d43e8537c78f3237d8e800ca697705942d890))
- **cloud:** Apple, Google and GitHub sign-in reach their providers ([8fab966](https://github.com/iamngoni/kanso/commit/8fab9664d2ef084d6cef0e6debeaa432f21c2cdc))
- **ai:** An agent allowed to suggest edits is told how, and that they wait for review ([cd688d0](https://github.com/iamngoni/kanso/commit/cd688d03a418723135d08655469dac6599bbadcf))
- **ai:** A rewrite that stopped partway can't replace the text ([de2010b](https://github.com/iamngoni/kanso/commit/de2010b11f19772e2dcbc44b5a543a83bc6f403c))
- **ai:** A permission sheet that ended under another dialog leaves that dialog open ([326054b](https://github.com/iamngoni/kanso/commit/326054b5b2ad8b8d83dc2c095b1b8b69f057ca24))
- **ai:** Say that the permission sheet asks and that suggested edits wait for review ([8afeec4](https://github.com/iamngoni/kanso/commit/8afeec438696aa42ed7f85c83ae78f4830c745f1))
- **app:** Declare Markdown and text documents in the macOS bundle ([e0d198e](https://github.com/iamngoni/kanso/commit/e0d198ea1dbb5e478800b298f57dcc19679b9f46))
- **packaging:** List Kanso in Open With for Markdown files on Linux and Windows ([70ce20c](https://github.com/iamngoni/kanso/commit/70ce20c3929389c6976e899fc442a852727fb607))
- **app:** Open the Markdown and text files the system hands to Kanso ([4043083](https://github.com/iamngoni/kanso/commit/4043083cb44377ed64934d66bfb4f15e81bc0cca))
- **engine:** Read a device approval's saved state from the store, not the caller's session ([8088af2](https://github.com/iamngoni/kanso/commit/8088af2b1d89328fb2d64a1018cc7f32c772fc9a))
- **store:** Clippy's thread-local lint is wrong about Android ([ddfc343](https://github.com/iamngoni/kanso/commit/ddfc3439900549fa8a660f7f0932faf0f6861a35))
- **mobile:** A closed locked note's tags and status stay hidden when it is opened ([394cc5c](https://github.com/iamngoni/kanso/commit/394cc5c337491fd1c5595af44274ea339dbf7ed7))
- **ci:** Name the mobile scripts' temporary files the way Linux's mktemp accepts ([18d416e](https://github.com/iamngoni/kanso/commit/18d416e6f9247e89363ddb73d1267da27025949e))
- **site:** Keep the code block's language label above its first line ([6682688](https://github.com/iamngoni/kanso/commit/6682688c51e941f607cdada167508bbc5939cc9b))
- **export:** Keep code readable on a light page when the theme is dark ([2d53d8e](https://github.com/iamngoni/kanso/commit/2d53d8e52ec2954899b062b741243c2e748fb8f8))
- **themes:** Colour code in Kanso Dark and Kanso Light so functions, types and properties read apart ([cce1d70](https://github.com/iamngoni/kanso/commit/cce1d70d742ee898ba07caf0acc065eac7a2b0d6))
- **mobile:** Kanso Light and Kanso Dark's code colours follow the theme files again ([4fc7e84](https://github.com/iamngoni/kanso/commit/4fc7e8482a83296ce5b0de5c9b79d702c9a7e3fe))
- **mobile:** Release builds on Android may reach the network ([c0f7a62](https://github.com/iamngoni/kanso/commit/c0f7a62224a5cdd5b44624ae2c549096745b215d))
- **mobile:** Answer a silent push from the app delegate, so it works before Flutter starts ([05e2e7f](https://github.com/iamngoni/kanso/commit/05e2e7f030fb27502086bc3b1f1551e6f8b29bb4))
- **mobile:** Kanso's app icon on iOS and Android instead of Flutter's placeholder ([2e5f015](https://github.com/iamngoni/kanso/commit/2e5f01558eec72a8619aebc50a656757c5572d0e))
- **mobile-core:** Signing in publishes the account's sharing identity, so others can share with a phone-only account at once ([16bdc13](https://github.com/iamngoni/kanso/commit/16bdc1371a0dfed03abc402b995f2795f4f0c5b5))
- **mobile:** Flutter run builds again ([7c92d28](https://github.com/iamngoni/kanso/commit/7c92d2862f7cc410c9737d39bcdd51402cd68ba3))
- **mobile:** Today's note opens inside the browsing screens ([6344da6](https://github.com/iamngoni/kanso/commit/6344da6f46d7eb8cd11e4eed9f3f41a127e454d1))
- **mobile:** Keep biometric tickets below 2^53 and cover them with tests ([ab5a87d](https://github.com/iamngoni/kanso/commit/ab5a87d2836cbb01c54984f695a966a91aafcc83))
- **mobile:** A forgotten biometric secret is written at once, and a late check opens nothing ([0a5be8b](https://github.com/iamngoni/kanso/commit/0a5be8b289c73d894956ba2d7b6a61810bc91404))
- **mobile:** Lock sheets drawn as the polished sheets, with their hairlines past the icons ([6ca90d3](https://github.com/iamngoni/kanso/commit/6ca90d3dd24d55a96197bdcf0e9593dc2d275708))
- **engine:** Align merged editor helper exports ([b93e279](https://github.com/iamngoni/kanso/commit/b93e279c6bc9c41732bf03bff32e6ef4e183b075))
- **engine:** Align merged exports and document Vim keys ([6af8adf](https://github.com/iamngoni/kanso/commit/6af8adfbcad24fb75169addc807011768acc6ccd))
- **mobile:** Read back a biometric item with nothing sealed yet ([a29c68b](https://github.com/iamngoni/kanso/commit/a29c68b5c19e40c09a0551ba806a91617b1ba25e))
- **store:** Replace and its undo never write into a locked note (#48 parity D, H) ([69cd1d9](https://github.com/iamngoni/kanso/commit/69cd1d9f400ce37ec60db482c4a4f2d37ffde757))
- **mobile:** Read back a biometric item with nothing sealed yet ([67f203e](https://github.com/iamngoni/kanso/commit/67f203edca98aebe1dd31f53bf75d62b96a61898))
- **app:** Find the publish words in the engine, and open the code colours to tests ([fd317cb](https://github.com/iamngoni/kanso/commit/fd317cbf9153fc7406df78b1dc0f3c2861f78d6b))
- **engine:** Colour a page's code as the preview does, locals and all ([bf69a4e](https://github.com/iamngoni/kanso/commit/bf69a4e676cca45048db7291748df81d64941e95))
- Expand tabbed note to fill pane and add tablet goldens ([0d5b98e](https://github.com/iamngoni/kanso/commit/0d5b98ebcbb9fd64337276f762bc84c47491b262))
- **mobile:** Ignore closed editor sessions for word goals ([#145](https://github.com/iamngoni/kanso/pull/145)) ([dcfb2d1](https://github.com/iamngoni/kanso/commit/dcfb2d1e1d676ab815350980d4d1ac20d5c1792c))
- Preserve shared files when locking notes ([#147](https://github.com/iamngoni/kanso/pull/147)) ([2747d8f](https://github.com/iamngoni/kanso/commit/2747d8f039c26c283d349cc9fb69090f2a31242f))
- Seal attachments on legacy locked notes when opened ([#148](https://github.com/iamngoni/kanso/pull/148)) ([c692fcc](https://github.com/iamngoni/kanso/commit/c692fcc2b7dc0cf6b7b8c8a08d224dc806b832b7))
- **ios:** Find rustup in Xcode Cloud's build phase ([1bb5e70](https://github.com/iamngoni/kanso/commit/1bb5e700b198898e80717e81c8a7a552aa0181ab))
- **ios:** Unblock the TestFlight upload (ITMS-90208, ITMS-90592) ([41eb142](https://github.com/iamngoni/kanso/commit/41eb14202540288253425ccf9708b02f12b8081e))
- **ios:** Declare exempt encryption in Info.plist, stop the ASC questions ([a865fe2](https://github.com/iamngoni/kanso/commit/a865fe231fbcc6685fa9aefa0da9ae4b4d37a382))
- Reject web-only Google Font files ([1376cb5](https://github.com/iamngoni/kanso/commit/1376cb54291128bee8ebd058eff2c0836c69b5b2))
- **mobile:** Interface font, downloaded fonts in every list, one-line download row ([55bfca3](https://github.com/iamngoni/kanso/commit/55bfca3d0ef2cda3137bf5747bd7f42d6b989f55))
- **mobile:** Set the note preview in the chosen fonts ([78f5d4e](https://github.com/iamngoni/kanso/commit/78f5d4e5866dadc9af4aa83af604afffb75b87ce)) by Claude
- **render:** Even table rows, capped nesting and the page's urls in preview blocks ([604fbd7](https://github.com/iamngoni/kanso/commit/604fbd744bfa21c08d477a081ec7cefa954203ea)) by Claude
- **mobile:** Native preview review fixes ([88bca8d](https://github.com/iamngoni/kanso/commit/88bca8d4a9f1a035af6dc4217df3309ea8499074)) by Claude
- **mobile:** Draw exports in the chosen preview theme ([002747d](https://github.com/iamngoni/kanso/commit/002747df4eef508411d8d5e80b50441fd4ef5b34)) by Claude
- **mobile:** Draw only what's on screen in a note's preview ([ae01ccf](https://github.com/iamngoni/kanso/commit/ae01ccf6aca1a30845638012148487195dc85d05)) by Claude
- **mobile:** Far jumps, long code blocks, selection colours, a top bar that gives way ([9854a89](https://github.com/iamngoni/kanso/commit/9854a89ed8bf41b51e5d43f475bcaf5dd2cfda08)) by Claude
- **desktop:** Line Live up with the preview to the pixel ([762678c](https://github.com/iamngoni/kanso/commit/762678c58f2694ccb1499efa75d0517bd01d73f1)) by Claude
- **sticky:** Keep a closed sticky's place and colour, and the note's text whole ([03990ac](https://github.com/iamngoni/kanso/commit/03990ac7b96210a09dd386c0292cf091c9fb0df4))
- **ci:** Turn master green again ([d42276b](https://github.com/iamngoni/kanso/commit/d42276b75b403b942982fa996af82a161e9f31b1))

### Under the hood

- **mermaid:** Draw Mermaid with Pele in QuickJS behind KANSO_MERMAID=pele, with a side-by-side comparison ([0b5c419](https://github.com/iamngoni/kanso/commit/0b5c419345d5cc5b5b8d83fe0ee5c594f1c41983))
- **export:** Keep the query table test to the notes it needs ([67fabdc](https://github.com/iamngoni/kanso/commit/67fabdc87a6a3fad9017e6cf4ec46f1678c51173))
- **store:** Notes in locked and nested locked notebooks stay out of query views ([849c439](https://github.com/iamngoni/kanso/commit/849c43940fea524d6fa12a1d7f0919f9ea6ec885))
- **tabs:** Cover tabs in a window and snapshots of the bar ([d1df3b4](https://github.com/iamngoni/kanso/commit/d1df3b4ff154b9ea1ade6c54a2b543c2f098c49f))
- **tabs:** Cover middle click, the menu, Command-Return, drags and tab keys ([6e84dab](https://github.com/iamngoni/kanso/commit/6e84dab115634454894870fe64ee8cc85597c0cc))
- **locked:** A locked notebook's background tab never shows its text ([35a342a](https://github.com/iamngoni/kanso/commit/35a342a7e1655ad3ec4785527b25f91ccbdb7254))
- **themes:** Take the merged themes, with visible states for fourteen of them ([07c981c](https://github.com/iamngoni/kanso/commit/07c981cceeb274f18dbdc447f5cd4f7fb789b8cd))
- **gallery:** Parse the builder's real index and sanitize its SVGs ([7560853](https://github.com/iamngoni/kanso/commit/75608531ee0fd4880d71404ead915bee05480d78))
- **ai:** Replace the hand-written providers with agent-runtime ([d8dc876](https://github.com/iamngoni/kanso/commit/d8dc8766ac673da31db63abee8ad5aec82de69d5))
- **ai:** Pin agent-runtime to its merged master ([5ffccff](https://github.com/iamngoni/kanso/commit/5ffccff225d4419e2d0784101b20eb552babecee))
- **gallery:** Prove edited copies survive and revocations clear every slot ([4283d70](https://github.com/iamngoni/kanso/commit/4283d70677f68bcf54ce5b743ea398eea03279d6))
- **gallery:** Say what a crash between the two catalog writes means ([9b69e16](https://github.com/iamngoni/kanso/commit/9b69e1676b8e1c29c661ace96ffaeadce40f6538))
- **themes:** Document the gallery and test it end to end ([4ab73f8](https://github.com/iamngoni/kanso/commit/4ab73f83418dfd3f42cbee501f178c2f8867e50b))
- **releasing:** Describe the themes key that already exists ([3358a86](https://github.com/iamngoni/kanso/commit/3358a86b96ef3e36dc855098387b4f4fae6caedb))
- **accounts:** Design passkeys and third-party sign-in ([#32](https://github.com/iamngoni/kanso/pull/32)) ([9d2fa41](https://github.com/iamngoni/kanso/commit/9d2fa41c813cf48f0e28814e360c865a2d44df26))
- **accounts:** State the provider-proof limits in the decisions row ([c8a1fce](https://github.com/iamngoni/kanso/commit/c8a1fce25d805349b41700ef75a4cb7fec4cde09))
- **accounts:** Note the recovery key in the passwordless delete question ([ca2ea64](https://github.com/iamngoni/kanso/commit/ca2ea647c6ef790e4b966ffd74d2e11c5b7c54ba))
- **accounts:** Note the password step for linking from a provider-only account ([80ac5e8](https://github.com/iamngoni/kanso/commit/80ac5e8efbeab755dc576745a955358bf6735095))
- **accounts:** Commit to the new device's key, keep the password wrap behind a password check ([b900511](https://github.com/iamngoni/kanso/commit/b900511c47a4f5c9374b7be9c2224f426c41c5da))
- Update account spec for security keys and live approvals ([21641aa](https://github.com/iamngoni/kanso/commit/21641aa12cd6dec341d4a15bbde70a88af57863b))
- **sign-in:** Provider flows and linking, step 4 (unfinished, picked up by the next builder) ([1407766](https://github.com/iamngoni/kanso/commit/140776623913746776a8dc99b05aad70c340fb3c))
- Sign-in providers ([cca892e](https://github.com/iamngoni/kanso/commit/cca892e276868ebbdd22ea45e010c4c58fd96e16))
- **sign-in:** Passkeys and the PRF wrap, step 2 (unfinished, picked up by the next builder) ([3770983](https://github.com/iamngoni/kanso/commit/3770983089140495a1d6ce475e3557c269ef6f11))
- Simplify optional PIN conversion ([20cbd5a](https://github.com/iamngoni/kanso/commit/20cbd5a19ea429df14b1c953e5a055e2f3dc666e))
- Add libfido2-dev to Linux dependencies ([15012c6](https://github.com/iamngoni/kanso/commit/15012c6f39136c5383d95db3672bd579da50031c))
- Verify sign-in providers and passkeys together ([38e270a](https://github.com/iamngoni/kanso/commit/38e270a430b81c3fc80567313e3a0543a91d2381))
- Include PR 60 app error messages ([e5be921](https://github.com/iamngoni/kanso/commit/e5be921e1949ca952d91cedf817166fcd52b91ce))
- **sign-in:** Device approval and live events, step 3 (unfinished, picked up by the next builder) ([cda2169](https://github.com/iamngoni/kanso/commit/cda2169c756edb7c4ca8a2df8cf698d49a42f418))
- **sign-in:** Say the Apple callback route is POST only ([c686dc8](https://github.com/iamngoni/kanso/commit/c686dc8d19458a385b283c916e4bb24cb5eafeb2))
- **macos:** Add the passkey Associated Domains entitlement and embed the provisioning profile ([db39b17](https://github.com/iamngoni/kanso/commit/db39b174057df8d625b2832cfa8a130f6c39c4c5))
- **app:** Keep the platform crates' PRF input, CTAP salt and client data equal to kanso-sync's ([369991b](https://github.com/iamngoni/kanso/commit/369991b350abf65583621240f6a1a0b9c3a42ea1))
- **snapshots:** Draw the signed-out passkey methods, the passkey username step and the security-key PIN form ([ef3d095](https://github.com/iamngoni/kanso/commit/ef3d095bd75be0e91f4134b9c61604ca201533a4))
- **sync:** Run sign_in.json over MemoryServer and check its shapes, skipping until it lands ([5d55b6c](https://github.com/iamngoni/kanso/commit/5d55b6ce08a18c8be8694bdfb03200e04c4488f2))
- **e2e:** Approve a new Mac live over the event socket and reconnect after a drop ([d1ee72f](https://github.com/iamngoni/kanso/commit/d1ee72f622ceaeaabb960134cb729f156eece226))
- **sync:** Run PR A's sign_in.json over MemoryServer and match the Worker's sign-in rules ([d18a21a](https://github.com/iamngoni/kanso/commit/d18a21aec896ba8188bf1b50b1f6276787478180))
- **sync:** Give MemoryServer's provider starts the providers' own pages ([1dcd3b5](https://github.com/iamngoni/kanso/commit/1dcd3b5822e7c535e8161d080a898e70385d62f6))
- **sync:** Give the stub's Google start a real query, which the start check accepts ([d45b328](https://github.com/iamngoni/kanso/commit/d45b328a2beb7bde1feb86df77353d6ac0f6df79))
- **macos:** Check the provisioning profile's expiry and certificate, and launch the signed app once in the release ([9df8416](https://github.com/iamngoni/kanso/commit/9df841632fd8caf2c9a1ae45a5740dc92e1dac41))
- **sync:** Give MemoryServer's provider starts the providers' own pages ([c74c1d4](https://github.com/iamngoni/kanso/commit/c74c1d4e3714823c8bb7d9396f5621c6a563e13b))
- **sync:** Give the stub's Google start a real query, which the start check accepts ([f258116](https://github.com/iamngoni/kanso/commit/f2581165fd118f246ce7fb4e3274a9e2c06dbd5e))
- **app:** Build security-key ceremonies through SecurityKeyPasskeys::new after merging B ([c74f530](https://github.com/iamngoni/kanso/commit/c74f530b88e2ab72fed3b4c8a1f9b7ca4db08fd6))
- **account:** Cover the sign-in method changes, their proofs and error paths ([a5dfea5](https://github.com/iamngoni/kanso/commit/a5dfea5500d316c272b2a15ba691471d8933b0a1))
- **sign-in:** Add the shared sign_in.json conformance fixture and its Worker runner ([b6911ed](https://github.com/iamngoni/kanso/commit/b6911eda454d389854536fd9e932903c437ac03c))
- **sign-in:** Document the finished Worker, where the code wins, and the owner's decisions ([c2e0121](https://github.com/iamngoni/kanso/commit/c2e0121d88487fffa7f33daefa7b26dae4354a22))
- **cloud:** Set the Google and GitHub sign-in client IDs ([c41a4c2](https://github.com/iamngoni/kanso/commit/c41a4c2345a8c48bba87d876a0280f0338cef828))
- **cloud:** Turn on passkeys, the AASA file and Sign in with Apple ([dca30fe](https://github.com/iamngoni/kanso/commit/dca30fe43278a26b6a0ff00a7599539652ae6958))
- **cloud:** Record the sign-in deployment ([7b4f433](https://github.com/iamngoni/kanso/commit/7b4f4331b99df249b684c076c3b039fc0b9cdc33))
- **app:** Snapshot the sign-in method buttons, the proof sheet and a link's wait ([9574257](https://github.com/iamngoni/kanso/commit/95742572b2addcaff123ff38f6ce8ab05ddff080))
- **cloud:** Let Kanso Dev use passkeys on sync.kansoapp.app ([c7bb77e](https://github.com/iamngoni/kanso/commit/c7bb77ed11b66d7cc6b7ed3d05e35ac87aa8de48))
- **themes:** The clear-a-theme property never picks the fallback as the theme being cleared ([5e755d9](https://github.com/iamngoni/kanso/commit/5e755d9aa8c10a8eab80118b0d69e7bc9fe305f3))
- Increase timeouts for property-based tests ([58f1b22](https://github.com/iamngoni/kanso/commit/58f1b2241b59eb02dca4c03a89c7ec48b6b930ab))
- **ai:** The request carries the larger answer room ([227ffa1](https://github.com/iamngoni/kanso/commit/227ffa10a8472c5fcc51b52de1949ae05bf7debe))
- **ai:** Only a whole rewrite can be accepted ([094ede4](https://github.com/iamngoni/kanso/commit/094ede4414aa809af15c13a38c5e505cdc211136))
- **changelog:** The permission sheet for agents ([4ab2ebc](https://github.com/iamngoni/kanso/commit/4ab2ebc59271005508ab95071aa54a8379737e12))
- Document AI in the README and the new Copy diagnostics ([000ed8c](https://github.com/iamngoni/kanso/commit/000ed8c2a2aeee950f1def9ecb55991beced2c58))
- **themes:** Every icon theme draws all of Kanso, with ten more icon themes and twenty color themes ([58f8bad](https://github.com/iamngoni/kanso/commit/58f8bad2be6664925a25f72600e8b679d4570089))
- **ai:** A manual matrix against the agents installed on this computer ([c0c0d4c](https://github.com/iamngoni/kanso/commit/c0c0d4c06417f72902a3fc3dbec566b4ccbf917d))
- Say Kanso opens Markdown files from the file manager ([242709a](https://github.com/iamngoni/kanso/commit/242709ac80752dade24919fd5f642bb803e5a639))
- **plugins:** Generated libraries and calls never reach locked, trashed, #ai-exclude or ungranted notes; the real host under the supervisor ([9cfb8af](https://github.com/iamngoni/kanso/commit/9cfb8af8eb8b73294e190cb5be8838d7257e14d2))
- **plugins:** The plugin guide and API reference, two sample plugins run by the tests, and the host packaged beside Kanso on every platform ([573f546](https://github.com/iamngoni/kanso/commit/573f5462c73af4c5e27cd907bbb0bd8ee8867475))
- **mobile:** Ignore anti-aliasing when comparing goldens with their files ([0abd817](https://github.com/iamngoni/kanso/commit/0abd8177fa5d1c9924d429cf94d68cb6920c0553))
- **mobile:** Allow 0.3% changed pixels against a golden file ([604aac3](https://github.com/iamngoni/kanso/commit/604aac3bb0336a15d7caaf4af64de6e012c3807a))
- **engine:** The kanso-engine crate, with what accounts say in plain words ([6e695c4](https://github.com/iamngoni/kanso/commit/6e695c4250ff020b50b9e5fe776dcf3e57e6403f))
- **engine:** Lock sessions and locked notebooks move into the engine ([e251c1a](https://github.com/iamngoni/kanso/commit/e251c1a6fee18b61e02c11c010afc9c1bca7b32b))
- **engine:** Sync scheduling, outcomes and ended-session cleanup move into the engine ([2e53be2](https://github.com/iamngoni/kanso/commit/2e53be2ed94235956e09a063ba189aae1bd882db))
- **engine:** Device approval's waiting state and the approver's checks move into the engine ([4c85d69](https://github.com/iamngoni/kanso/commit/4c85d69f95558627450e2e786d06a71b9928220e))
- **engine:** Account flows, provider sign-in, device approval and the launch check move into the engine ([d0c5822](https://github.com/iamngoni/kanso/commit/d0c582291a62f4ba1da17566b4d07d04b2682260))
- **sync:** The desktop Keychain behind a system-keychain feature, so the engine builds without it ([1406a91](https://github.com/iamngoni/kanso/commit/1406a916bb0ac82032472be85682ef80566d2df8))
- **engine:** Build the engine for iOS and Android on GitHub's macOS runner ([e19eeb8](https://github.com/iamngoni/kanso/commit/e19eeb80090b6a6497d0deaa5ad3e2a0620d5793))
- **themes:** Pin how every built-in theme resolves, fits and reaches the component library ([f467aa2](https://github.com/iamngoni/kanso/commit/f467aa26cd8e760c9f08d10cd44f2beb70267499))
- **engine:** Fuzzy matching, wiki-link and tag autocomplete, link copying and the rename offer move into kanso-engine ([bffd226](https://github.com/iamngoni/kanso/commit/bffd2266a9c75176f14fb8365c55eebf8f88fc10))
- **engine:** Daily notes, templates, due dates and reminder rules move into kanso-engine ([c4e9d8b](https://github.com/iamngoni/kanso/commit/c4e9d8bcbf91c23a32229e2d77b4657e1054b24d))
- **engine:** The Markdown edits, auto-pairing, list and table keys and UTF-16 offsets move into kanso-engine ([4f8b4dd](https://github.com/iamngoni/kanso/commit/4f8b4dde54c649b4bba37c2d843ee8075d5781ac))
- **engine:** Palettes, the theme file format and theme fitting move into kanso-engine ([2c31b23](https://github.com/iamngoni/kanso/commit/2c31b234fb3f8aeafba69de88ca051339a6e380d))
- **engine:** Check for one copy of tree-sitter and a window-free mobile tree, and document the highlighter ([0aa7cdc](https://github.com/iamngoni/kanso/commit/0aa7cdc1ef85a9beb46983a47909e054afde1efe))
- **plugins:** An ignored check against the live directory: install a sample, take its update, find it withdrawn ([142a253](https://github.com/iamngoni/kanso/commit/142a2534aa8d8118c5068633a6cfbb310c6878b7))
- **mobile:** A release profile that is small and unwinds ([152fc01](https://github.com/iamngoni/kanso/commit/152fc01d53495d77fe357bebe5c63b37f66a6e68))
- **ai:** Agents on this computer sit behind an acp feature, which the phones leave off ([bd1335f](https://github.com/iamngoni/kanso/commit/bd1335ff6352469e36bab90e2cdf0e915319f485))
- **mobile:** Fixtures are found at run time, and kanso-sync's tests build without the desktop Keychain ([6574128](https://github.com/iamngoni/kanso/commit/6574128318afb04738674ac016f4142f3fdc6f2d))
- **mobile:** Scripts that check the core for the five targets and run its tests in the iOS Simulator and on Android ([d95cd61](https://github.com/iamngoni/kanso/commit/d95cd61ba40351aad8d0c81836e33a72a6343f6f))
- **mobile:** Check the core for five targets, test it in the iOS Simulator, and in an Android emulator when the crates change ([1e3bef2](https://github.com/iamngoni/kanso/commit/1e3bef26ecef06626078e905c5f0260882455352))
- **mobile:** A stand-in library to measure the core's size on phones, and a script that builds the table ([8e9da31](https://github.com/iamngoni/kanso/commit/8e9da316ea8229786428c66c9d65182f57cb19df))
- **mobile:** Mobile targets in DEVELOPING.md, and the changelog line for the rest of the core ([3914b11](https://github.com/iamngoni/kanso/commit/3914b11caf4b9222a18dc30ef16b936744e36e0b))
- **mobile:** The checks cover kanso-tls and the size stand-in ([c927dc5](https://github.com/iamngoni/kanso/commit/c927dc5217858d5c359e567839d1d7854955e10e))
- **mobile:** Every call of the bridge is tested, on a computer and on a device ([c7c51e6](https://github.com/iamngoni/kanso/commit/c7c51e683b469ef1f28815bd7216699fbeaebe84))
- **mobile:** IOS builds with Swift packages, for Apple silicon simulators ([097cd17](https://github.com/iamngoni/kanso/commit/097cd173f44b891aed32af80bb07396699205696))
- **mobile:** Release builds, integration tests and checks of the app and its library ([30b2fa0](https://github.com/iamngoni/kanso/commit/30b2fa014416fb949d05259939fa8e1207d657e9))
- **engine:** The note list's places, sorting and date grouping are decided in kanso-engine ([4fd7369](https://github.com/iamngoni/kanso/commit/4fd7369dce9680b5e7de3e0fe10fe7c523351a30))
- **engine:** Changes to several notes at once are planned in kanso-engine ([3f3b50b](https://github.com/iamngoni/kanso/commit/3f3b50b3b73b8f58541cd8a4240cb7665bdd2510))
- **engine:** The built-in themes are compiled into kanso-engine ([ed802cc](https://github.com/iamngoni/kanso/commit/ed802cc01b02ddfdcce6d53b250d8bc39fe3e0de))
- **mobile:** Binding tests load the computer's library, never a simulator's ([59a9e04](https://github.com/iamngoni/kanso/commit/59a9e046dcba5ef9ffed2c381e8cfb6113debdc7))
- **mobile:** Goldens of the main screens, phone and tablet, light and dark ([4a55926](https://github.com/iamngoni/kanso/commit/4a55926805d654b066170df0f514c3aeb62c1bae))
- **mobile:** Browse, search and open flows on a simulator and an emulator ([9ea32b7](https://github.com/iamngoni/kanso/commit/9ea32b7e803966031432e9c75f164c417ae2b216))
- **mobile:** Frame timings of scrolling the note list ([063b5f7](https://github.com/iamngoni/kanso/commit/063b5f72a53307c8f0f3033f455bf80ecd0fb1a9))
- **mobile:** Retire the core check and the built screens' mocks ([b7e525b](https://github.com/iamngoni/kanso/commit/b7e525b02f2f278e722cb4985722e411ae4751fc))
- Run CI on master and for pull requests from forks only ([81d4a65](https://github.com/iamngoni/kanso/commit/81d4a6588417778d8dbd9d5cf1a623c57ff54843))
- **cloud:** Record the push deployment ([6ea1c5b](https://github.com/iamngoni/kanso/commit/6ea1c5ba9c07232208641a29ddb2bd41795f7b5c))
- **mobile:** One version for the apps, from Cargo.toml and the run number ([8d5f9d4](https://github.com/iamngoni/kanso/commit/8d5f9d408bd5c17e42b0f9d28ac90d12219f4d5f))
- **mobile:** Sign the Android release with an upload key when there is one ([9a5df23](https://github.com/iamngoni/kanso/commit/9a5df23c9c54b2146d248e29b24ec68a1d83b3fc))
- **mobile:** TestFlight, Play internal testing and Shorebird release workflow ([181c6ba](https://github.com/iamngoni/kanso/commit/181c6bac2e1416e47aca180e3b5fc3cf2d4688d2))
- **mobile:** Release guide, store listing text and a screenshot script (#48 step 13) ([8980308](https://github.com/iamngoni/kanso/commit/898030890906f95efb3718bbaf259e76c163a875))
- **mobile:** The release manifest has the network permission now ([82aa385](https://github.com/iamngoni/kanso/commit/82aa385cbb95a706b7e9c774279d755e4c6f0723))
- **mobile:** Goldens of every screen in an iPad's and an Android tablet's windows ([16d6608](https://github.com/iamngoni/kanso/commit/16d6608196b343ba7fc78aa51e48c3000539c614))
- **mobile:** Resize the window through Split View, Slide Over and a rotation ([735ac51](https://github.com/iamngoni/kanso/commit/735ac51f0791f1ad020e9c12bc3176094809b036))
- **mobile:** Tidy imports ([385bf29](https://github.com/iamngoni/kanso/commit/385bf298215bf6fae43ba77dd1581742942e3558))
- **mobile:** Flows that work on every layout class, and a way to turn the device ([be7ccae](https://github.com/iamngoni/kanso/commit/be7ccaef2e1782083f2967ae1832a463cd45a5be))
- **mobile:** Run the flows on an iPad simulator and a Pixel Tablet emulator, in both orientations ([492cdc4](https://github.com/iamngoni/kanso/commit/492cdc4c2c80351b642b15b754d4cd66fe3521b4))
- What a mobile crash report may hold (#48 step 13) ([e49290f](https://github.com/iamngoni/kanso/commit/e49290f37971a35735fac1b6df6489172648ece7))
- **mobile:** Install the NDK with retries, keep the iOS debug symbols; export compliance and privacy answers (#48 step 13) ([3ec4e6c](https://github.com/iamngoni/kanso/commit/3ec4e6c534b071b32ab8bf9c1c7a9772e619dac0))
- **mobile:** Render and keep the store pictures; docs for crash reports, export compliance and the owner's steps (#48 step 13) ([4ff8340](https://github.com/iamngoni/kanso/commit/4ff8340b3d5f384c955d80ea0063c35d2c39fd35))
- **mobile:** Render the store pictures on macOS, where the goldens are made (#48 step 13) ([185917e](https://github.com/iamngoni/kanso/commit/185917eee69bb698ff6d3e66483c4bfa5d5bf8e8))
- **mobile:** Linking and navigation core, package C ([#48](https://github.com/iamngoni/kanso/pull/48)) ([c8da236](https://github.com/iamngoni/kanso/commit/c8da2368e90e42e12c770bbad1e06cb475a19e84))
- **mobile:** Cover the linking bridge on this computer and on a device ([d894d6d](https://github.com/iamngoni/kanso/commit/d894d6d56f5347a5fda8b4a53eeeadc765dc4d5e))
- **mobile:** Import layer in Rust, not finished (#48 package J) ([93712bd](https://github.com/iamngoni/kanso/commit/93712bd81c78ab27eb20f7c3f15b2f582d4ed9f8))
- **mobile:** The facade joins by approval and recovery key, syncs in the background and shares notes against the real Worker ([a01ba83](https://github.com/iamngoni/kanso/commit/a01ba83b19c89832f8aeefa2bf278f9d4d3aa885))
- **mobile:** Pictures and tests for sharing and the theme gallery ([795ecbd](https://github.com/iamngoni/kanso/commit/795ecbd3e693d32118b4a4a129535bb173d10e41))
- **mobile:** F2-sharing-gallery-review-fixes, stopped mid-task for the handoff to the main session ([70d4aa8](https://github.com/iamngoni/kanso/commit/70d4aa86191f565701a0e4729e471057611beb02))
- **mobile:** Sync words in the engine, sync settings, comments on shared notes ([6b41bd1](https://github.com/iamngoni/kanso/commit/6b41bd117cbf06ed60b9ff3190aef374176de0e3))
- **mobile:** Comments count the end of wide text, bridge for sync and comments ([407799a](https://github.com/iamngoni/kanso/commit/407799a58e8633e46b014232bcd6923db713b331))
- **mobile:** R5-comments-sync-settings, stopped mid-task for the handoff to the main session ([f664d3b](https://github.com/iamngoni/kanso/commit/f664d3bb247b412d6d9872d582f6c6e408716288))
- **mobile:** Step 8 handoff to the main session ([c1c6ea4](https://github.com/iamngoni/kanso/commit/c1c6ea48add9c1a5a50d38910ddd59c8eab64360))
- **mobile:** Tidy the account tests for the analyzer ([feb5758](https://github.com/iamngoni/kanso/commit/feb575873aa57c4744713d68797987b80b14659e))
- **mobile:** Goldens and widget tests for the account screens ([17e8052](https://github.com/iamngoni/kanso/commit/17e80521ecb0050290e0e3bdeedd72e52190c263))
- **mobile:** The app reports being in front to sync and to device approval ([ba70966](https://github.com/iamngoni/kanso/commit/ba7096671cd32ee8feb132e13ba2e9edce137689))
- **mobile:** The router and shell tests provide the account blocs, and the settings pictures show Sync as it is now ([b42d2c2](https://github.com/iamngoni/kanso/commit/b42d2c23e63de2d8fbe68055ccee9f70b686a416))
- **mobile:** A flow that swipes a note to the trash and takes it back ([08a4ab0](https://github.com/iamngoni/kanso/commit/08a4ab01b9e96fb7bb00260438a45188309b258c))
- **engine:** The autosave rule and the conflict checks are decided in kanso-engine ([cd8f711](https://github.com/iamngoni/kanso/commit/cd8f711224814ccfae7d3c6869e51ca5ff91f2dd))
- **mobile:** Goldens of the editor, and edit and conflict flows on a simulator and an emulator ([12cd4cb](https://github.com/iamngoni/kanso/commit/12cd4cb1ae0567bc295b621fd22b5d45dae5c2b5))
- **mobile:** Retire the editor's mocks ([3ed791f](https://github.com/iamngoni/kanso/commit/3ed791f63552baf252e7812cc0a7049b1c61d992))
- **mobile:** Measure typing in a long note ([acdef44](https://github.com/iamngoni/kanso/commit/acdef4471cde09b611270aed9477f90f39f2b22f))
- **mobile:** Composition, dictation, autocorrect and spell check through the editor ([89d8d00](https://github.com/iamngoni/kanso/commit/89d8d007a15fd2deb637d0621fbc75c7f7cc9ee4))
- **mobile:** The editor's device checks and the performance gate ([6054e19](https://github.com/iamngoni/kanso/commit/6054e198d2e28406a460c1d9ad2c4ceccdfdb019))
- **mobile:** Where a fresh highlight's time went, and the host and simulator numbers after the fix ([a4d9148](https://github.com/iamngoni/kanso/commit/a4d91487cc9fa1def142ec23634cf33e953217ea))
- **engine:** Where a new note goes is decided in kanso-engine ([f314beb](https://github.com/iamngoni/kanso/commit/f314beb64e6a7ed5e0d7434441b227d6e0544281))
- **mobile:** Goldens of the editor on an iPad and an Android tablet, in both orientations ([67ce69a](https://github.com/iamngoni/kanso/commit/67ce69a0302b1a6baab5aee40ea7dd0454bfa278))
- **mobile:** Goldens and flows for new notes and notebooks ([a7cbee3](https://github.com/iamngoni/kanso/commit/a7cbee31f16ba66be4bdd2b085931d45c5095f26))
- **mobile:** The new note flow on every device, and a note just made opens at its title ([843f4ea](https://github.com/iamngoni/kanso/commit/843f4eaf32a57266e9a4bb95f2f07619a5ffdaff))
- **mobile:** The privacy settings test gives Settings the notebooks it now reads ([05e0916](https://github.com/iamngoni/kanso/commit/05e091695c57222808416a2e896f76c8cc07a80e))
- **mobile:** The Tasks place's screens (unfinished, full test run not seen) ([574d493](https://github.com/iamngoni/kanso/commit/574d4935290b688511139e9dad1607d2e101153b))
- **render:** The preview page in the app (unfinished, facade unchecked) ([a58f8e9](https://github.com/iamngoni/kanso/commit/a58f8e92a3b8a2c162bd1448a09e54e2e1a3398f))
- **mobile-core:** Preview facade (unfinished) ([525f685](https://github.com/iamngoni/kanso/commit/525f685f9eb5932214f78ecf5e2f099b9d46b7a0))
- **mobile-core:** The preview page and its facade (unfinished, not fully checked) ([af1f44a](https://github.com/iamngoni/kanso/commit/af1f44adb41052730322fb9f948f6f6a624c4de8))
- **changelog:** The preview on the phones (#48 step 7) ([4eb2dc4](https://github.com/iamngoni/kanso/commit/4eb2dc479590af9c6da0158cc3ad15663e4e1c7a))
- **mobile:** Run the mobile jobs on the self-hosted runners ([f829e3c](https://github.com/iamngoni/kanso/commit/f829e3cb2ca1f5a1ed3380e13fc50309a199b6a4))
- **mobile:** Goldens for the polished look, and tests of its button sizes ([6e57934](https://github.com/iamngoni/kanso/commit/6e57934b6c3e5e180cf12e9990f4a55f4687e28b))
- **mobile:** Fix the import fixtures: a Roam page needs a uid, and a copied single file needs its folder (#48 package J) ([c726b3f](https://github.com/iamngoni/kanso/commit/c726b3f5443e2087fd3c933564a7e3ef405ed533))
- **mobile:** Regenerate the bridge for the import API (#48 package J) ([1c1b118](https://github.com/iamngoni/kanso/commit/1c1b118bf080b7b243977b146806f6722f38fe1e))
- **engine:** Attachments and quick capture are decided in kanso-engine ([13a78c4](https://github.com/iamngoni/kanso/commit/13a78c450f272127482ac9151b792ab83b29d2b8))
- **mobile:** Capture acceptance harnesses (#48 step 10), unfinished ([04af4f0](https://github.com/iamngoni/kanso/commit/04af4f04cd112120ff5c6091fdc82330538f9083))
- **mobile:** The locked-notes contract for step 9 ([587c3fe](https://github.com/iamngoni/kanso/commit/587c3fe639f37ef1678d4a3fe164fd3750d86044))
- **mobile:** Call every new lock function through the bridge, with a biometric answer helper ([8c236b9](https://github.com/iamngoni/kanso/commit/8c236b9a8d35dc1ef8152231b6ee8068460090bc))
- **mobile:** Properties for the lock logic and tests of every lock BLoC ([3f9196d](https://github.com/iamngoni/kanso/commit/3f9196dfe7ed881f7ca597e7c3cfb96a288aefbd))
- **mobile:** Step 9 handoff, locked notes UI and security fixes part done ([09d7149](https://github.com/iamngoni/kanso/commit/09d7149bf35b94fdc4ec1f703fa7cc29dca0d741))
- **changelog:** Locked notes on the phones (#48 step 9) ([6ad9b86](https://github.com/iamngoni/kanso/commit/6ad9b86fbd348427b5f6e38c035205f4a8be4fdf))
- **mobile:** Lock goldens, the locked flows, and the goldens the polish merge left behind (#48 step 9) ([d4cc2aa](https://github.com/iamngoni/kanso/commit/d4cc2aae9bca95013e930a77e8b1efe397a18135))
- **mobile:** The capture sheet's flows and its goldens across phone and tablets (#48 step 10) ([5842066](https://github.com/iamngoni/kanso/commit/5842066e2c3d0edd75d2d285ea2d81933af9b656))
- **mobile:** Draw the Icons page goldens as the app draws it ([238b34b](https://github.com/iamngoni/kanso/commit/238b34bafcb2b8a4a5c4d12f2998b756106f45c0))
- **engine:** Smart paste's text rules, image sizes and drawings move into kanso-engine ([f140c83](https://github.com/iamngoni/kanso/commit/f140c83eb4083f992622d8c6688a1064614b7a31))
- **engine:** Word counts, word goals and the editor settings move into kanso-engine ([aa3c478](https://github.com/iamngoni/kanso/commit/aa3c478c803504a4c875c5b19050e5cdcba4b3aa))
- **engine:** Vim keys move into kanso-engine ([8f063ab](https://github.com/iamngoni/kanso/commit/8f063ab524ff78814cb3a6c5d90f099649d46c14))
- **mobile:** History, Replace and backups facade and bridge, unfinished (#48 parity D, H) ([5c34cf4](https://github.com/iamngoni/kanso/commit/5c34cf46f6210ca0fec4b1b77f5bcb0b1d1f51df))
- **mobile:** Cover history, Replace and backups ([38937f2](https://github.com/iamngoni/kanso/commit/38937f25cafc10aa99cbfd391c35a64ba14c5feb))
- **mobile:** Draw the Icons page goldens as the app draws it ([0b81f43](https://github.com/iamngoni/kanso/commit/0b81f435b7e90b079506d0f71dbf7a759ee773e0))
- **mobile:** Draw the Settings goldens with both new rows ([14a7b51](https://github.com/iamngoni/kanso/commit/14a7b51a37632736dcf893c4d800eb3eb3501662))
- **engine:** Move the notebook icon and color catalogue into kanso-engine ([#48](https://github.com/iamngoni/kanso/pull/48)) ([31ae50d](https://github.com/iamngoni/kanso/commit/31ae50dce309a7369e047799e16a2ef59adc2032))
- **engine:** Move the batch notice words and the copy title into kanso-engine ([#48](https://github.com/iamngoni/kanso/pull/48)) ([d06a1c0](https://github.com/iamngoni/kanso/commit/d06a1c0b0e137c15e410835bffe1edbf3f8c3e44))
- **engine:** Move the sidebar's folding, tag and status rules into kanso-engine ([#48](https://github.com/iamngoni/kanso/pull/48)) ([5088612](https://github.com/iamngoni/kanso/commit/50886120abfb49198cc1543e922ed0f71b857268))
- **mobile:** Parity A+B: note actions and library management, unverified ([#48](https://github.com/iamngoni/kanso/pull/48)) ([e350022](https://github.com/iamngoni/kanso/commit/e3500221ee4daf108e3b162d9008eac121a3088b))
- **mobile:** Give the linking fixtures a notebook and read context as words ([5d4e356](https://github.com/iamngoni/kanso/commit/5d4e3561c93a5294c4865a0c5b44f91b4dc4c077))
- Handle null selection semantics in library pane test ([478fe31](https://github.com/iamngoni/kanso/commit/478fe31f890153d50e93d6c8cf6b6b9332ed4720))
- Simplify backlink results and accept palette slices ([76630af](https://github.com/iamngoni/kanso/commit/76630afebbc1d24a4708005c6497c1db7b40782a))
- Remove unused sidebar test imports ([8b88b90](https://github.com/iamngoni/kanso/commit/8b88b903fafdb5425ace0c9676727693e156a530))
- **engine:** Move export and publish logic out of kanso-app (#48 package I) ([e5db103](https://github.com/iamngoni/kanso/commit/e5db1034f310e9ae29a359db1c5b4637a46e741d))
- **engine:** Keep the fuzzy tail from forming a run of its own ([2214de6](https://github.com/iamngoni/kanso/commit/2214de6a30224bcdcb9422fca8b9b68adc256f13))
- **mobile:** Give the linking fixtures a notebook and read context as words ([35d0a7c](https://github.com/iamngoni/kanso/commit/35d0a7c76c221a324104fc562dc01ebf9d06b6ca))
- **engine:** Take a user's tags as the parser reads them ([3f9034d](https://github.com/iamngoni/kanso/commit/3f9034d4f07cbc772bd2a39effd05dabf7e9c8cb))
- Simplify active theme code colors ([d0b487f](https://github.com/iamngoni/kanso/commit/d0b487f814207079b750cbca58c4feee71f2216c))
- Restrict set_timing to test builds ([7b9192b](https://github.com/iamngoni/kanso/commit/7b9192b66501e276ae1ea92658402c250d67eedd))
- Add discovery bridge coverage, add OCR plugin dependency ([36c9f58](https://github.com/iamngoni/kanso/commit/36c9f588b66d58a518cce42e522535d91c9c73ad))
- **mobile:** The bridge and channel for journal, templates, reminders, boards and due chips (#48 F+G) ([58dde68](https://github.com/iamngoni/kanso/commit/58dde688f966e8ec6eda7f41a41de2bc8a001921))
- **mobile:** Journal, templates, reminders, boards and due chips, unfinished (#48 F+G) ([ba88dc7](https://github.com/iamngoni/kanso/commit/ba88dc75a502f8b5abf4488bd20363df4ddc6705))
- **checkpoint:** Journal/templates/reminders parity plus goldens; fmt, clippy and analyze clean ([1931724](https://github.com/iamngoni/kanso/commit/1931724a225dbbf2116b6f84183644524f7eaad4))
- Master into journal/templates/reminders parity ([6db6f1c](https://github.com/iamngoni/kanso/commit/6db6f1c5ae4334c659c9afd451ffe812cc88466a))
- Master into discovery, with the journal's sections and regenerated goldens ([e83a2fc](https://github.com/iamngoni/kanso/commit/e83a2fc4cbfbf06dc087eb2603c7215b090403cb))
- Bring master's journal, discovery, board and reminder work into the tablet branch ([97b4e56](https://github.com/iamngoni/kanso/commit/97b4e562210c7c1f57df6632890fade6dec53f2a))
- Bring master into the note drag branch ([9de85e3](https://github.com/iamngoni/kanso/commit/9de85e38cdc13b118ca9720a74bee3da1625e8a5))
- Update browse flow for WebView previews and lazy lists ([#149](https://github.com/iamngoni/kanso/pull/149)) ([fe72473](https://github.com/iamngoni/kanso/commit/fe724738a6e25629b6e3309a8937e3ba536aa73b))
- Update lock confirmation messages to clarify file sealing ([#151](https://github.com/iamngoni/kanso/pull/151)) ([c27339a](https://github.com/iamngoni/kanso/commit/c27339a2b39feafd0d0a2ba30290283e3001ffd8))
- **mobile:** Build the iOS app in Xcode Cloud ([8856e50](https://github.com/iamngoni/kanso/commit/8856e50555d18e7056d2f6cee7907c26c48c59ab))
- **ios:** Add the Xcode Cloud manifest and set the display name and category ([0e9da06](https://github.com/iamngoni/kanso/commit/0e9da06107cd3fab0a1856c83098ce0255a7af62))
- **mobile:** Record build 3's export compliance answer ([31791e5](https://github.com/iamngoni/kanso/commit/31791e5677df5f50cc247287ac59d0cb6bf57394))
- Add undo history tests for editor text revisions ([81a1037](https://github.com/iamngoni/kanso/commit/81a1037d3f2dfbdf0222bff26ff22b700bec810b))
- **render:** Native preview blocks, in progress ([f774dba](https://github.com/iamngoni/kanso/commit/f774dbad0014e8f8b22f2f316dc586c5cb19852e)) by Claude
- **render:** Native preview blocks, in progress ([5ab481e](https://github.com/iamngoni/kanso/commit/5ab481ead6e7bba5647004fd93af2bca8a1b10ef)) by Claude
- **render:** Native preview blocks, in progress ([c41b660](https://github.com/iamngoni/kanso/commit/c41b6607e26e7c8bbc2a35f3e9254dd2b937ac45)) by Claude
- **render:** Native preview blocks, in progress ([55c56e7](https://github.com/iamngoni/kanso/commit/55c56e77ad2e240bb31b20c9da4fde1220910a85)) by Claude
- **cloud:** Record the crash-report and sealed-size deploy ([69c5e93](https://github.com/iamngoni/kanso/commit/69c5e93b436aeec51bf4af4743e1ecd362071e20))
- **mobile:** Far jumps, long code blocks, selection colours, test teardown ([cb602c1](https://github.com/iamngoni/kanso/commit/cb602c1a4d755299e0206e8410c1cf31c6ca541f)) by Claude
- **app:** Render the snapshots on Linux ([c8817a4](https://github.com/iamngoni/kanso/commit/c8817a49044613729fdf3ccb0913d66b3025380b)) by Claude
- **mobile:** Redraw the note bar goldens with Live beside Preview and Edit ([5bb44c6](https://github.com/iamngoni/kanso/commit/5bb44c61771342fc96d327f29917b81b99861478))

## Unreleased

### New

- **mobile:** An anonymous daily usage count, shown in a one-time notice on the first production release: a closed list of app-active and feature-use counts (never notes, titles, tags, files or anything typed), a pseudonymous ID, the device form and OS version, and the sync/AI/note-count ranges documented in `docs/METRICS.md`; nothing is counted or sent before OK, and debug builds and tests send nothing
- **locked:** Sealing attachments of locked notes, step 1: the sealed-file layer. Each attachment gets a random file key; the bytes are a `0x01` container (`segments` header, content segment now) sealed with that key and bound to the file and its seal generation, and the key is wrapped under the note's lock key with the note's salt in the AAD, so rekeying stays 32 bytes per file. A sealed file's document carries `kansoFileLock` (app-only: kept on API merges, stripped from served output and imports), the store reads `Plain`, `Sealed` or `Missing`, the ordinary write path refuses to replace sealed bytes or drop the marker, and `MAX_SEALED_ATTACHMENT` (the 10 MiB limit plus 4 KiB) caps the stored container (#44 step 1)
- **mobile:** Choose the interface font, for menus, lists, labels and buttons, in Settings › Fonts. A downloaded Google Font now joins every font list (interface, editor, preview and code) so it can be chosen for each on its own, and the download field and button sit on one line
- **mobile:** Choose themes and icon themes, set preview text size, and choose the diagram renderer on phones and tablets.
- **mobile:** The Tasks place's library: open tasks across the notes grouped Overdue, Today, Upcoming and No date (and Completed when shown) as the desktop's Tasks view groups them, with a notebook filter, the overdue count for the place's badge, ticking one as one undo step, and locked notes left out whether their lock is open or not; the grouping now lives in the shared engine (#48 step 7)
- **mobile:** The Tasks place on the phone and tablet: a Tasks row in the library with the overdue count, and a list of the tasks in the desktop's groups with due chips, a notebook filter and Show Completed, ticking with a tap (struck through in place, with Undo), the note a tap opens, pull to sync, and the keyboard's arrows, space, return and undo on a tablet (#48 step 7)
- **mobile:** Journal navigation on phones and tablets: Previous day, Next day, Today, and a calendar to open or start daily notes (#48 parity F)
- **mobile:** Create notes from templates, create templates, and choose a daily template in Settings (#48 parity F)
- **mobile:** Notes and Tasks boards on phones and tablets, with status and task completion moves; due dates appear as chips (#48 parity G)
- **mobile:** Local task reminders plan on launch, resume, library changes and the Settings switch; Open targets the task line, with Mark Done and Dismiss actions (#48 parity G)
- **editor:** Formatting shortcuts in the note's text, shown in the toolbar's tooltips and More menu and in Note › Format: ⌘B bold, ⌘I italic, ⇧⌘X strikethrough, ⌘E code, ⌥⌘C code block, ⇧⌘K link, ⌥⌘1 to ⌥⌘6 headings, ⌥⌘9 task, ⌥⌘8 bulleted and ⌥⌘7 numbered list, ⇧⌘. quote (Ctrl on Windows and Linux). Each acts on the selection (every line it touches for line formats), takes the format off again when pressed twice, and is one undo step. They don't fire in the title or search, or in Vim's Normal mode
- **plugins:** The plugin directory is the `iamngoni/kanso-plugins` repository, fetched only from its path on raw.githubusercontent.com and signed by its CI with the directory key now compiled in; Kanso checks it 30 seconds after opening and every six hours while a plugin from it is installed, for withdrawn versions and updates; `kanso-plugin build-directory` and `verify-directory` build and check it; the Windows and Linux updaters replace `kanso-plugin-host` with `kanso`, all or nothing (#37)
- **themes:** The theme gallery's signed catalog, path rules, install plan, key list and SVG sanitizer, with imported and on-disk icon themes held to it (#49 steps 1–2)
- **themes:** Install, update and remove gallery color and icon themes under `<data>/gallery/`, with atomic swaps, crash-safe state, edited-copy detection, and the gallery-aware theme loaders (#49 step 3)
- **themes:** Browse the gallery from Settings › Themes and Icon Themes, with search, filters, paired cards, live color preview and install/use/remove actions (#49 step 4)
- **themes:** Check the gallery on the app's update timer, update unedited installed themes quietly, honour signed revocations (an edited revoked copy is disabled, not deleted), fall back when a selected theme is withdrawn, and add the "Keep gallery themes up to date" switch and `themes.log` (#49 step 5)
- **themes:** Document the gallery (Browse, badges, offline, edited copies, revocation, built-ins), add the loopback end-to-end test and Browse snapshots, and wire the `KANSO_GALLERY_INDEX` debug override (#49 step 7)
- **ai:** Answer with an agent on this computer: Settings › AI › Answers come from adds Claude Agent, Codex, Gemini CLI, OpenCode or a custom ACP command, used through the person's own subscription. Kanso starts the agent in an empty private folder with a scrubbed environment, offers it no file or terminal access, rejects its permission requests, and picks its most restrictive mode (#46 steps 0–4)
- **ai:** An agent may also search and read the notes itself through a read-only notes server: a short-lived token per session (Bearer on `/mcp` only, or a private loopback listener when the local API is off), never locked, trashed, `#ai-exclude` or out-of-scope notes, every note it reads under Used notes, and a notebook scope sheet; follow-ups keep one agent session per panel conversation; more agents from the ACP registry, marked "Not tested with Kanso", with a confirmed one-click `npm install -g` (#46 step 5)
- **ai:** Let an agent suggest edits to the text or title of the notes it may read, or a new note, from Settings › AI (off by default): each suggestion waits in a review queue with a diff and writes nothing until you apply it, once, if the note is unchanged and still not locked, trashed or `#ai-exclude`; a note's history names the agent (#46 step 7)
- **ai:** An inline assistant in the editor (Improve, Make Shorter, Fix Spelling and Grammar, Continue, Summarize the selection, from the right-click menu or the palette) and a note summary in the Ask panel, on the chosen backend with no tools and no notes access; a suggestion is applied only when you accept it, in one undo step, and a locked, trashed or `#ai-exclude` note is never sent (#46 step 6)
- **ai:** When an agent asks to use one of its own tools, a sheet names the agent, the tool and its kind, with Reject as the default (Enter and Escape reject) and Allow once for just that request; Kanso never answers "always", a request waits a minute at most and is cancelled with the turn, and without a window it is rejected (#46 step 4)
- **ai:** Copy diagnostics in Settings › AI, beside Test, copies a report to paste into an issue: Kanso's version and system, whether the agent was found and where, its name and version, whether it runs, and the names, timings and error codes of its last messages. It holds no note text, titles, answers, tokens or the agent's own output, and a crash can still be diagnosed (#46 step 8)
- **plugins:** Plugins, with notes kept safe: palette commands, fenced blocks and inspector panels from JavaScript plugins, each run in its own sandboxed `kanso-plugin-host` process with time and memory budgets. A plugin reaches only the notebooks you chose at install, never locked, trashed or `#ai-exclude` notes; its changes are staged, reviewed when they reach beyond the open note, and undone in one step. Network only to its listed HTTPS hosts through Kanso's proxy, tokens in the Keychain, a signed directory with author keys pinned on first install, developer mode for unsigned folders, safe mode (`--safe-mode` or Shift at launch), `kanso plugin dev` and the `kanso-plugin` packing and signing tool (docs/PLUGINS.md) (#37)
- **engine:** Lock sessions, account flows and sync scheduling move into the new UI-free `kanso-engine` crate, shared with the coming mobile app; the desktop behaves as before (#48 step 1)
- **engine:** Wiki-link and tag autocomplete, link copying, daily notes and templates, due dates and reminders, the Markdown edits and the theme format move into `kanso-engine`, which also gets a Markdown and code highlighter of its own, checked against the desktop editor's, with Enter continuing a list item and UTF-16 offsets for the mobile editor; the desktop behaves as before, except that Tab no longer fails on lines selected down to the start of the line below a list (#48 step 2)
- **engine:** The store, renderer, shared notes, importers, sync client and AI client build for iOS and Android and pass their tests on the iOS Simulator and an Android emulator, with a `mobile` release profile that is small and unwinds on a panic; phones check servers' certificates with the system's store (the new `kanso-tls` crate), and the desktop behaves as before (#48 step 3)
- **engine:** A note's version history (why a version was kept, who made it, the changed sections and words), the Replace options, words and batch IDs, and the backup schedule (folder, daily and manual backups, the list, deleting one made on request) move into `kanso-engine`, shared with the mobile app; the desktop behaves as before (#48 parity D, H)
- **mobile:** The library gives the phones a note's version history, what changed since a version and restoring one, Replace in all notes with its undo, and daily and on-request backups inside the library's folder; a locked note's versions are opened only while its lock is open, and Replace never touches a locked note (#48 parity D, H)
- **sync:** The sync server can wake a signed-in device with a silent push when another device changes something: devices register an APNs or FCM token (`PUT /v2/account/devices/<id>/push`), a push or a shared-note update wakes the account's other devices and the note's other members' devices, at most one push per device every 30 seconds with a trailing one never lost, tokens are cleared on sign-out, revoke, expiry or a new sign-in and when the provider reports them gone, and no push carries a note, title, count or name (#48 step 8b)
- **mobile:** The iPhone, iPad and Android apps register the device for silent push and wake to sync when another device changes something, from Settings › Sync › “Wake this device when notes change elsewhere”, on until turned off (off tells the server to forget the token); a push carries nothing, a token is handed to the account server whenever the app signs in or comes forward, and where push can't work the switch is dimmed and says so, with the phone syncing when opened (#48 step 8b)
- **mobile:** Release engineering for the iPhone, iPad and Android apps: one version from `Cargo.toml`, a signed IPA to TestFlight and a signed bundle to Play internal testing from the `v*` tag (every step waits for its secret), Shorebird patches made only from CI and signed, the export compliance answer, the apps' privacy manifest, and the store listing text (docs/RELEASING-MOBILE.md) (#48 step 13)
- **mobile:** A swipe or a change to several notes shows a short notice at the foot of the screen, worded as the desktop's is, with Undo that puts back what each note had (a field changed again since keeps its newer value), over the list and the note on a phone and a tablet (#48 step 7)
- **mobile:** The desktop's bundled icon themes, chosen under Settings › Appearance › Icons: each theme listed with a row of its icons and its author and licence, the choice applied to every icon in the app at once and kept on the phone, and an icon a theme leaves out drawn as Kanso's own; the icon theme format is read by `kanso-engine` (as the desktop reads it) and the library carries only the SVGs the phone draws (#48 step 7)
- **mobile:** Crash reports without an SDK and without note content: the iPhone, iPad and Android apps keep a report when they crash (MetricKit, Android's exit reasons and ANRs, uncaught JVM and Dart errors), built from a whitelist of fields (stack frames in a strict grammar, versions, the kind of device and the day), shown in full, and sent only when you tap Send; the sync server keeps it for 90 days with no IP address or account (docs/CRASH-REPORTS.md) (#48 step 13)
- **mobile:** Settings › About links to the release notes and help on kansoapp.app, shows the Shorebird patch number, and the store pictures for every required size are rendered from the golden tests on each build (#48 step 13)
- **site:** Release notes and Help pages, and a Crash reports section on the privacy page (#48 step 13)
- **engine:** The note graph and its layout, related notes' model with its download and indexer, the tags and twin a note's nearest notes suggest, search by meaning and reading the words in images move into `kanso-engine` (`discovery`), so the desktop and the phones read them from one copy; the desktop behaves as before, and the phones reach it through `kanso-mobile` until their views land (#48 package M)
- **mobile:** The note's preview for the phones' app (#48 step 7): the published page with tappable notes, wiki-links to headings, notes to make, tags, attachments, task boxes, due-date chips, query views and a copy link on every code block, headings anchored, code colored by the theme, failed diagrams shown with the reason and empty drawings as the desktop shows them; it carries no script, handler or fetch, and every address not on the app's short list is dropped. Ticking a box writes only that box and refuses a note changed since the page was drawn; a locked note's text is read only while it is open
- **mobile:** A note in Preview is its rendered page, under its title and facts: a tapped note link opens the note, a wiki-link to a note not made yet makes it and opens it, a tag opens its list, a heading link scrolls to it, a web or mail link goes to the system, an image attachment opens full screen and any other file opens a sheet to share it, a task box is ticked in one write with Undo, and a code block's button copies it; the page follows the theme and is drawn again when the library changes (#48 step 7)
- **mobile:** Locked notes on the phones' app (#48 step 9): lock a note or a notebook, unlock with the lock password or Face ID, Touch ID or fingerprint, change the lock password and remove a lock, the desktop's auto-lock and a lock when the app goes to the background, and a cover over a locked note in the app switcher (`FLAG_SECURE` on Android) so its text never shows there
- **mobile:** Accounts on the phones' app (#48 step 8): sign up, sign in with a password, passkey or Apple, Google or GitHub, join an account from another device by approval or with the recovery key, approve a new device from a phone, see and revoke devices, and sign out, all in the desktop's words; signing in publishes the account's sharing identity so others can share with a phone-only account at once
- **mobile:** Sync on the phones (#48 step 8): Settings › Sync says when the library last synced or why it couldn't ("Synced 5 min ago", paused until 14:05, signed out), with Sync now, Sync automatically and Sync every (1 minute to 4 hours) as the desktop has them; the app syncs when it comes to the front, soon after an edit and on the chosen interval, and pulling down the note list or Tasks syncs
- **mobile:** Shared notes on the phones (#48 step 8): share a note with people by username, with the first-share fingerprint to compare, roles, links that expire, requests to approve, stopping and leaving (each keeps a private copy), and Shared with you for invitations and joining by link; an open shared note is live, so what others type appears within a moment and what you type reaches them, and someone's typing never undoes yours
- **mobile:** Comments on shared notes (#48 step 8): the note's bar has Comments with the number open; the sheet lists them in text order, then those whose words were deleted, with Resolve and Reopen, Edit and Delete for your own, and @mentions; choosing one opens the editor on its words, the commented words are tinted, and Comment in the text menu comments on the selection
- **mobile:** Others in a shared note (#48 step 8): their carets with their names and their selections in their colours, as the desktop draws them; a status line under the bar ("Shared · Can view · Read only · Live", a private copy once sharing ends, and the size limit when it is near) and a banner for what couldn't be done, with Dismiss; a note shared to view or comment on opens read only
- **mobile:** The theme gallery on the phones: browse and search the gallery's color and icon themes from Settings › Appearance, with their badges, and install one from its card (#48 step 8)
- **mobile:** A note's preview is drawn by the app itself instead of a web view (#48): its headings, lists and task boxes, quotes, code with its language and copy button, tables, query views, due chips, math, diagrams and drawings come from the core as typed blocks and are set in the note's fonts and the preview theme's colours, with text that can be selected across them, links, task ticks and copies that do what the page's did, and a double tap to edit

### Fixed

- **mobile:** A note's preview follows the chosen preview theme and gallery themes: its colours are the preview theme's, fitted with the interface and syntax themes as the rest of the app's are, and its code is coloured by the syntax theme; it was drawn in the built-in palette of the code's theme, so the preview theme was ignored and a gallery theme left the preview undrawn
- **mobile:** HTML and PDF exports of a note are drawn in the chosen preview theme, gallery themes included, as the note's preview is: the preview theme's colours fitted with the interface and syntax themes, and Kanso Light or Dark when they can't be read; they were drawn in the built-in palette of the code's theme, so the preview theme was ignored and a gallery theme couldn't be exported
- **mobile:** A note's preview is set in the chosen preview and code fonts. The page's policy blocked the fonts it embeds, so a downloaded Google Font fell back to the system's, and the fonts bundled with the app were never embedded at all, so Inter, Source Serif 4, Literata, Instrument Sans, JetBrains Mono and IBM Plex Mono fell back too
- **mobile:** Long notes open and update quickly in preview: only the blocks on screen are drawn, and a tick or a sync draws again only the blocks that changed, so a note of thousands of blocks no longer stutters. Text can still be selected across blocks, now with the title and facts above, and a heading's link still scrolls to it
- **mobile:** Jumping far in a long note's preview, to a heading from a link or by dragging the scroll bar, is immediate, as it is in the note's text shown before the preview is drawn: the blocks far from the screen are never laid out, their heights are guessed until they come near, where every block on the way was laid out before (to the 2,500th of 5,000 blocks took about 4 s in a debug build).
- **mobile:** A long code block in a note's preview is drawn a piece at a time as it scrolls into view, rather than all its lines at once, and still looks like one block: one edge, its language and one copy button that copies all of it; its pieces scroll sideways together, and a selection across them keeps the line breaks.
- **mobile:** Selecting in an open note, its preview, title and facts, is drawn in the preview theme's accent, the handles too, rather than the app's own colour and the platform's blue.
- **mobile:** The bar above a note no longer runs off the edge of a narrow phone, or of the note pane on an iPad in portrait, when a journal note has more actions than fit beside the back button: its buttons keep their full size, and the ones that don't fit move into the More menu, the least used first (the note's info, then comments and share, then the calendar and the days before and after), where they ran under the back button and off the edge. The mode switch and More always stay in the bar.
- **locked:** The first time an older locked note is opened on the phones, its linked attachments are sealed and recorded in `kansoFiles`; opening it again leaves the note alone, so legacy locked notes no longer keep readable attachment bytes (#44 step 7, mobile)
- **locked:** Locking a note copies a picture or file that an unlocked note still shows, so the unlocked note keeps its readable original and the locked note gets its own sealed copy; locking a notebook seals a shared file once for all its notes, and removing one note's lock decrypts only that note's copy (#44 step 3, mobile)
- **export:** Published notes and exported pages keep their code readable when Kanso uses a dark theme: a light page takes the code colors of your light theme (Kanso Light's if it can't be found), where it had pale colors meant for a dark background
- **discovery:** A locked note (sealed, or in a locked notebook) or a note tagged `#ai-exclude` is never embedded for related notes, so it is never a related note, a suggested tag's source, a near-duplicate or a hit of a search by meaning, and any vector it already had is deleted; an image only such a note shows is never read for its text; and the graph leaves locked notes and their links out until the lock password has opened them. Locked notes were embedded by their title before (#48 package M)
- **accounts:** Approving a new device from another one works without restarting Kanso first, and cancelling the sign-in cancels that request
- **app:** Kanso is listed in Finder's and Explorer's Open With for Markdown and text files, and opens a file from there, from a double-click once chosen, or from `kanso file.md`, as a note that is shown and selected
- **sharing:** A comment or someone's caret in a shared note stays on its words after an accented letter or an emoji earlier in the text, where it could drift by a character or two
- **themes:** Code is easier to read in Kanso Dark and Kanso Light: functions, types, properties, attributes, keywords, strings and numbers each get their own colour, and operators and punctuation a quiet one, all at 4.5:1 or better on the editor and the code block background
- **replace:** Replace in all notes never writes into a locked note (sealed, or in a locked notebook), whatever the preview named, and Undo replace leaves a note that was locked since instead of failing or writing plain text into it (#48 parity H)

## [0.8.4](https://github.com/iamngoni/kanso/compare/v0.8.3...v0.8.4) - 2026-10-22

### New

- Import Apple Notes from its default library location ([184aa3e](https://github.com/iamngoni/kanso/commit/184aa3e4bada92bcbf7bb20245a11da9645aaabb))

### Fixed

- Prevent overlapping link scan matches ([b43bb79](https://github.com/iamngoni/kanso/commit/b43bb790ef90259e0cce570cce2c0bc513e074c9))

## [0.8.3](https://github.com/iamngoni/kanso/compare/v0.8.2...v0.8.3) - 2026-10-22

### Fixed

- **import:** Apple Notes builds on Linux and Windows again, and imports long first lines, links and Math Notes results intact ([f41ffb7](https://github.com/iamngoni/kanso/commit/f41ffb7f0dd22daed53ffcce904caad50d5b6a9e))
- **import:** A TextBundle or TextPack opened on its own imports, and any text.* is its body ([a22ac25](https://github.com/iamngoni/kanso/commit/a22ac252f7d8ad4c3f3f9ac1a8a5646cae669bf6))
- **import:** Roam pages keep their dates, block references, aliases and multi-line blocks ([90084d4](https://github.com/iamngoni/kanso/commit/90084d4d647d14caf165ae18c4cd84100311c192))
- **import:** Tomboy's deleted notes go to the trash, templates are left out, and notebooks are notebooks ([f472aac](https://github.com/iamngoni/kanso/commit/f472aac91e535f316ed3cf17e968d79c1f433eee))
- **import:** Deeply nested HTML no longer overflows the stack, and HTML links with spaces or parentheses resolve ([4c2c2df](https://github.com/iamngoni/kanso/commit/4c2c2df2889b93e2c8ade0176468e05b2e39b1af))
- **import:** Apple Journal entries keep their order, media, dates and places; Keep notes without an edit time import; big HTML pages strip scripts quickly ([0f87530](https://github.com/iamngoni/kanso/commit/0f87530425fe068c477d65fd9a75a3f33da1012c))

### Under the hood

- **import:** The link-finding property test only builds real code spans ([dd8db50](https://github.com/iamngoni/kanso/commit/dd8db50306c7ce4ca95ecda94f5bfe86bbf3b440))

## [0.8.2](https://github.com/iamngoni/kanso/compare/v0.8.1...v0.8.2) - 2026-10-22

### New

- Add support for seven new library import formats ([78d0792](https://github.com/iamngoni/kanso/commit/78d0792cbb972faf1576c283b668562652282826))

## [0.8.1](https://github.com/iamngoni/kanso/compare/v0.8.0...v0.8.1) - 2026-10-22

### New

- **voice:** The words you dictate show in the note as you say them ([63d9d0b](https://github.com/iamngoni/kanso/commit/63d9d0bb83c95d61ca0af6394558cf68d543284a))
- **icons:** Every icon theme from the themes repository comes with Kanso, on its own Settings page ([9b32f34](https://github.com/iamngoni/kanso/commit/9b32f34e23a3f161fa2908162ff255645ee5aaff))
- **editor:** Pick any installed font for writing, monospace and the preview, and size the preview on its own ([b073ad0](https://github.com/iamngoni/kanso/commit/b073ad0731f748fbf192ca477863480ddab96388))
- **app:** Zoom the whole app with ⌘= / ⌘− / ⌘0 ([c2a4eb6](https://github.com/iamngoni/kanso/commit/c2a4eb6b8a7f9bc1d15d6fb7dc623fbc0e54b083))
- **voice:** Spoken punctuation and commands for on-device dictation ([27f7cd4](https://github.com/iamngoni/kanso/commit/27f7cd4a5aa9c97ab12589ac60b2c727b84c98ec))
- **notes:** Go back and forward between notes, and copy a note's link ([f4b3c39](https://github.com/iamngoni/kanso/commit/f4b3c3983b64ebdc18173eee960152c3aa4a3f02))
- **editor:** Misspelled words in a note get a red wavy underline, with suggestions on right-click ([c12c129](https://github.com/iamngoni/kanso/commit/c12c1293bae88fa9d54d8b1973bb7b5eb4172533))
- **notes:** Open a note in its own window ([eedecab](https://github.com/iamngoni/kanso/commit/eedecabb886437a37fabc4f4c980e7c6f84e4132))
- **notes:** Select several notes and drag them onto a notebook or tag ([a41ef94](https://github.com/iamngoni/kanso/commit/a41ef944519b4233a44d945cc2ce4d256a2113a8))

### Fixed

- **voice:** Pressing the mic with System Dictation chosen starts dictation instead of closing Kanso ([7533747](https://github.com/iamngoni/kanso/commit/75337476b91e1c1bb80df7dd24fb7a6d53a4a40a))
- **history:** Versions stay newest first when the computer's clock goes back ([61a2570](https://github.com/iamngoni/kanso/commit/61a2570f2ffaa77eeb5273f9571aac33947c03be))
- **notes:** Copy link and open in window from a note window and from several selected notes ([a16afa1](https://github.com/iamngoni/kanso/commit/a16afa143c640587c8892ce0195cc589a5f6555b))
- **settings:** The spelling row uses an icon every built-in icon theme draws ([2cc3a54](https://github.com/iamngoni/kanso/commit/2cc3a5411f8c30dfd95e98f5474c177bf3057a71))
- Review fixes across the merged QOL features ([0466acb](https://github.com/iamngoni/kanso/commit/0466acb456349c7b6c3fe696f62f3eaa9bffa34b))
- **notes:** The back/forward buttons ask the store once per visit; Copy Note Link works in a note's own window ([b3da519](https://github.com/iamngoni/kanso/commit/b3da519752fd8d48b6f54f8f18bd8cf0d1e19f1b))

### Under the hood

- **themes:** The built-in themes come from the kanso-themes repository, as a submodule ([a05ee2a](https://github.com/iamngoni/kanso/commit/a05ee2a210ca223941966e3866f41da58221d790))
- Compile Speller::new only on macOS and in tests, fixing Linux clippy ([920fb4c](https://github.com/iamngoni/kanso/commit/920fb4c62e1a454973fa53d69e1b4661a9ca9a45)) by @gridopsbot[bot]

## [0.8.0](https://github.com/iamngoni/kanso/compare/v0.7.0...v0.8.0) - 2026-10-22

### New

- **icons:** Icon themes restyle every icon in Kanso and switch live, in place of icon packs ([02703c9](https://github.com/iamngoni/kanso/commit/02703c926d5e60df306eb963e419929b742e602c))

### Fixed

- **icons:** Imported icon themes keep their attachment icons, credits and licenses, and large Zed themes import ([a27f6c3](https://github.com/iamngoni/kanso/commit/a27f6c3288b07352792026b55202e61b81e73ba4))
- **collab:** Someone who joins a shared note sees the others' carets within seconds, not ten ([4b21503](https://github.com/iamngoni/kanso/commit/4b215030f2ea010df56f73d0a11158131676c114))

### Under the hood

- **icons:** The component library's own icons can follow an icon theme ([b6774a4](https://github.com/iamngoni/kanso/commit/b6774a4160c82aca64d121a6aa4df566ed730abb))
- **icons:** The README and the icon themes guide explain how to make, import and choose an icon theme ([b4c2186](https://github.com/iamngoni/kanso/commit/b4c2186c3a0923bd49c61def048244543de6741f))
- **cloud:** Record the 0.7.0 Worker deployment, its collaboration migration and the bookmark before it ([dba36ff](https://github.com/iamngoni/kanso/commit/dba36ff9c702d45339df1b0222b6892bd51fdd13))
- **site:** Parse each note once for quick open's destinations ([8bbd15a](https://github.com/iamngoni/kanso/commit/8bbd15a94fd82b68c706d75b4dd801b9282e7db2)) by @gridopsbot[bot]

## [0.7.0](https://github.com/iamngoni/kanso/compare/v0.6.1...v0.7.0) - 2026-10-21

### New

- **voice:** Kanso turns speech into text on the device with Parakeet ([b300d96](https://github.com/iamngoni/kanso/commit/b300d967354ed458145da8bad4b53dcbfa0d2f17))
- **editor:** Voice input writes what you say into the note ([8125099](https://github.com/iamngoni/kanso/commit/8125099925324487119dda878bdb650609b0b97b))
- **search:** The palette finds notes about what you type, not only notes with its words ([351d3b5](https://github.com/iamngoni/kanso/commit/351d3b5ec05f01fc8783348989c7d6b88c1dae75))
- **tags:** Kanso suggests the tags that notes about the same things have ([7816245](https://github.com/iamngoni/kanso/commit/78162452db0c71630ddf3b16a5dbae0887294fea))
- **editor:** A note that says nearly the same as another says so, and the two can be merged ([db1e581](https://github.com/iamngoni/kanso/commit/db1e581812dbabaa376dae9f53d89d09906f0f21))
- **voice:** A recording dropped into a note is transcribed on the device, under its link ([cd22f45](https://github.com/iamngoni/kanso/commit/cd22f45ffa48810a8be67f89fe4325dedc55b29e))
- **locked:** Windows Hello unlocks locked notes on Windows, with the lock password as the fallback ([6942a92](https://github.com/iamngoni/kanso/commit/6942a926bc0827e46bde246f647a3a27fe0c1d39))
- **locked:** Lock a whole notebook, and every note in it and in notebooks inside it is sealed, now and when notes go in later ([f3d3785](https://github.com/iamngoni/kanso/commit/f3d378548e6dbf4235099355938b1855e546c300))
- **board:** A board lays the list's notes out by status, and their tasks as To do and Done, moved by drag or keys ([c3e17e2](https://github.com/iamngoni/kanso/commit/c3e17e2d0eae0aec24f501dd63cdd7c6abf20822))
- **journal:** A month calendar marks the days with a daily note and opens or starts any day's note ([d4f9057](https://github.com/iamngoni/kanso/commit/d4f9057519791c623edb7887677d77a00e6921b2))
- **publish:** A notebook publishes as a small site at one link, sealed end to end, with its notes linked and its locked notes left out ([fb6db09](https://github.com/iamngoni/kanso/commit/fb6db0918d60e98bcfc545cd7c19a70ceb27c2ee))
- **collab:** Share a note with other Kanso users and edit it together, end-to-end encrypted ([ca8f47b](https://github.com/iamngoni/kanso/commit/ca8f47b9961135df853ad01c25b9d4ec8aa90ba6))
- **sidebar:** Notebooks fold like an accordion, and each can have its own icon ([d5f8aab](https://github.com/iamngoni/kanso/commit/d5f8aab4b75b0b698c441612b55da14a8073e85d)) by @gtchakama
- **sidebar:** The notebook icon picker has 144 icons and a search ([55c893a](https://github.com/iamngoni/kanso/commit/55c893af7c2fae1b3e69096c1c2e8e25aeb77770)) by @gtchakama
- **icons:** Icon packs give notebooks more icons and can draw Kanso's own ([3e9b295](https://github.com/iamngoni/kanso/commit/3e9b295a3a93fee9a9eeec5802427a500e01d143)) by @gtchakama
- **icons:** Browse and install icon packs from inside Kanso ([b012b07](https://github.com/iamngoni/kanso/commit/b012b074a48dec20d3ddca4763441137525cca6b)) by @gtchakama
- **icons:** An icon pack chosen for the app shows at once, without a restart ([483e991](https://github.com/iamngoni/kanso/commit/483e99198c48d9d88defb4ac73c82fedc3c5915e)) by @gtchakama

### Fixed

- **locked:** A locked notebook's text stays in Kanso through templates, moved notebooks, meaning results and late embeddings ([795bf8a](https://github.com/iamngoni/kanso/commit/795bf8ae13cfa78281ddcb155d5d341da122c53c))
- **journal:** The calendar builds on Windows and Linux, where the first weekday comes from no system setting ([283d34c](https://github.com/iamngoni/kanso/commit/283d34c3ae1bc06e264a31d529f422341e0b08f5))
- **collab:** Shared notes survive leaving members, deleted accounts, long text, late updates, large edits and locked copies ([8545620](https://github.com/iamngoni/kanso/commit/854562024a86bb9824816b26c2ca79ac41b560d5))
- **icons:** A notebook's pack icon shows from launch, and no icon pack, however it is made, can slow or stop Kanso ([2fa94e9](https://github.com/iamngoni/kanso/commit/2fa94e9beaedd605793249849c92d02e81b7cea5))
- **icons:** Uninstalling a pack keeps the files you added to it, and an install finishes and says so with the gallery closed ([c7ae156](https://github.com/iamngoni/kanso/commit/c7ae15684b66232670d3095cd959a1df7d0f6af6))
- **sidebar:** The arrow on a notebook around the one showing folds it, and the palette shows a locked notebook's lock ([f6b9ed8](https://github.com/iamngoni/kanso/commit/f6b9ed8a0ec2bb7e2b478c41d50d6f8c06b7dc5a))
- **icons:** Choosing an app icon pack lets go of the old window as closing it would, and voice input carries over ([2f9c4c0](https://github.com/iamngoni/kanso/commit/2f9c4c0a0eba1823a1ca221c5c15ce6e9d677dc4))

### Under the hood

- **app:** Voice input's rules, apart from the microphone ([da66f3b](https://github.com/iamngoni/kanso/commit/da66f3bbba7b7186795c4534d5b82868811ca758))
- **app:** The config remembers how voice input listens ([7545a4b](https://github.com/iamngoni/kanso/commit/7545a4b676a8abb1ea0021b319f7bbcd4357b080))
- **app:** Model downloads carry on where they stopped ([ead993f](https://github.com/iamngoni/kanso/commit/ead993fb0f3ae7d0498116c1efc265743cd2ac4b))
- **macos:** Kanso may ask for the microphone ([6903679](https://github.com/iamngoni/kanso/commit/690367966122fe788dcaacaee982b2cc4a6636de))
- **cloud:** The socket rate-limit test passes on a runner slow enough to refill every frame ([e5c76ca](https://github.com/iamngoni/kanso/commit/e5c76ca2b51b7bcf055fd753733d4c1c96e6bf27))
- **cloud:** The presence budget test waits for every frame's answer on a slow runner ([7f6d2d5](https://github.com/iamngoni/kanso/commit/7f6d2d5c05b35c52c0489f9bda7c077b8aa7d072))

## [0.6.1](https://github.com/iamngoni/kanso/compare/v0.6.0...v0.6.1) - 2026-10-21

### New

- **app:** Windows and Linux get a menu bar ([25c1c19](https://github.com/iamngoni/kanso/commit/25c1c198d64c327954248dedffbf99894f137ff9))
- **app:** A note's menu is the same in the list and the editor ([7912ce4](https://github.com/iamngoni/kanso/commit/7912ce4c27c60ee8fc164f0153bc73e20c7a58ed))
- **app:** Settings copies the MCP setup for your AI agent ([8a6e575](https://github.com/iamngoni/kanso/commit/8a6e5754a5771e03f1bc109cc94951c42a08c65c))
- **editor:** A Dictate button starts the system's dictation in the note ([fc99d25](https://github.com/iamngoni/kanso/commit/fc99d253de47b5be1071bac724176d2b36498530))
- **sync:** Kanso's server is sync.kansoapp.app, and published notes open on share.kansoapp.app ([48a79f8](https://github.com/iamngoni/kanso/commit/48a79f869b475265d157dd1d3e274978abb6094d))
- **app:** Kanso tells people they can sync, without getting in the way ([7b61dfe](https://github.com/iamngoni/kanso/commit/7b61dfe3c3735ba04b370df1811833c3455c4297))
- **locked:** Lock a note with a password, sealed on this device and in sync ([aff0a7c](https://github.com/iamngoni/kanso/commit/aff0a7c711c4ab57198c9b2c4cf30b861ffee2a6)) by @gtchakama
- **app:** Password fields have an eye to show what was typed ([2d2ca64](https://github.com/iamngoni/kanso/commit/2d2ca64b402b81823a8caef0fedb67740a6915ac)) by @gtchakama
- **locked:** Touch ID unlocks locked notes, with the lock password as the fallback ([9051847](https://github.com/iamngoni/kanso/commit/90518471036c4a7fcf1d15ced46dec92be0e3588)) by @gtchakama

### Fixed

- **locked:** A locked note's old text is gone on every device, and the API never serves its seal ([e94888e](https://github.com/iamngoni/kanso/commit/e94888e00006b5d01207b168926abcc4b5f132f7))

### Under the hood

- **excerpt:** The sample roadmap lists wiki-links instead of naming another app ([4646f32](https://github.com/iamngoni/kanso/commit/4646f328e9aade7b35967f762b6be04b48af2c8b))
- Kanso names other apps only where it imports from them ([07761e6](https://github.com/iamngoni/kanso/commit/07761e6cde665538e8b801d92a0b1fc027dc7565))
- **editor:** Dictation lands in a note as spoken ([a1e0a72](https://github.com/iamngoni/kanso/commit/a1e0a7267eb66220cdab2a493ba5882f8ac52f24))

## [0.6.0](https://github.com/iamngoni/kanso/compare/v0.5.2...v0.6.0) - 2026-10-20

### New

- **cloud:** Short links are made on go.kansoapp.app ([2c13feb](https://github.com/iamngoni/kanso/commit/2c13febb560e61646d9276540646960054a91556))
- **site:** Kanso's website, kansoapp.app ([e339fd3](https://github.com/iamngoni/kanso/commit/e339fd36f17a7dc5a871e3161907a022603e1e27))
- **import:** Markdown folders, Obsidian, Logseq, Notion, Evernote, Bear, Joplin, Standard Notes, Simplenote and UpNote come in, with links, tags and attachments ([ccb4a03](https://github.com/iamngoni/kanso/commit/ccb4a034d1faf482ec2a4872fdcc0de88c053f39))
- **app:** Import… previews and imports other apps' notes, File › Open… brings in a Markdown file, and kanso import does both from a terminal ([d56f379](https://github.com/iamngoni/kanso/commit/d56f3799f4488399d70813b4b760a13529183869))
- **site:** The home page is Kanso, running in the browser ([4d03793](https://github.com/iamngoni/kanso/commit/4d037936d98b2a2f8383019880ff4d5bc9dc0dbb))

### Fixed

- **import:** A Joplin export's name comes from its file on Windows too ([df9b0aa](https://github.com/iamngoni/kanso/commit/df9b0aa56fbc4b413c51f36d8191ae2d2393a8b4))

### Under the hood

- **licence:** Kanso is free software under the AGPL-3.0-or-later ([9ff280e](https://github.com/iamngoni/kanso/commit/9ff280e1fea6c26cb3f969d959b00ec6a8c3ac22))
- **snapshots:** The demo roadmap lists wiki-links instead of naming another app ([40e8b93](https://github.com/iamngoni/kanso/commit/40e8b93deb8465296374e07787f96065fa20e771))
- **import:** The Notion export is written by its test, so Windows can check out the repository ([5c4da38](https://github.com/iamngoni/kanso/commit/5c4da38332106d8d864f587e79c8021d6f1e0ecb))

## [0.5.2](https://github.com/iamngoni/kanso/compare/v0.5.0...v0.5.2) - 2026-10-20

### New

- **cloud:** Published notes get a Rerout short link that previews the note's title ([46eb294](https://github.com/iamngoni/kanso/commit/46eb29486dc32741899efb2be47da632ef932923))
- **sync:** Publishing sends the note's title and shows the short link with the key ([d8dfc5f](https://github.com/iamngoni/kanso/commit/d8dfc5fa8b25ed5b3ab74d974eb99eb96d1a73b5))

### Fixed

- **import:** Importing a library on Windows builds a SQLite file URI it accepts ([515dcf8](https://github.com/iamngoni/kanso/commit/515dcf83f70ed27740ab84c071c2386f6bbf2ef7))
- **updates:** Restart quits and opens the new Kanso, instead of waiting for the next quit ([2cd9879](https://github.com/iamngoni/kanso/commit/2cd98798e2ca6d38f611f4d971180310e9e966a6))

### Under the hood

- Run every crate's tests but the app's on Windows and Linux too ([e4b2dfd](https://github.com/iamngoni/kanso/commit/e4b2dfd19183f4ed684bacc7d8afd386d0a83533))
- **e2e:** The wrong-password check holds across a rate-limit window boundary ([dedb0e3](https://github.com/iamngoni/kanso/commit/dedb0e355c433b4cf2bfb27d450e01037ec43db8))
- **changelog:** The 0.5.1 notes describe the Windows import fix without naming another app ([62594db](https://github.com/iamngoni/kanso/commit/62594dbefa32a50ff2bc30a28da8d4d6b77d6c20))
- Bind the e2e Worker's port in the same step that picks it ([2bd8f69](https://github.com/iamngoni/kanso/commit/2bd8f699e1f21ee7cf2b0fd14621ce82c5ffada2))

## [0.5.0](https://github.com/iamngoni/kanso/compare/v0.2.0...v0.5.0) - 2026-10-20

### New

- **release:** A per-user Windows installer beside the zip ([f66153a](https://github.com/iamngoni/kanso/commit/f66153a377dbab5b5aa139f56146421134c71a3a))
- **render:** Draw math, Mermaid diagrams and Excalidraw sketches in the theme's colours ([8594dfd](https://github.com/iamngoni/kanso/commit/8594dfd222cf8a03e8514d746db3e1ae7fbdaba2))
- **preview:** Show math, diagrams and Excalidraw drawings in the preview, and export notes as web pages or PDFs ([653584c](https://github.com/iamngoni/kanso/commit/653584c35a0a9b1dfda5e3eb734015af22a966b1))
- **mcp:** Add kanso-export-html to export a note as a standalone web page ([f39daf2](https://github.com/iamngoni/kanso/commit/f39daf284abae746c92179e2b19bed2f71635c67))
- **math:** Show $$…$$ on a line of its own as display math, centred, in the preview and exports ([4694608](https://github.com/iamngoni/kanso/commit/46946083be7502def1adbeb8a3e3f2b0399b57a2))
- **store:** Index each note's links, headings and tasks on this device ([14a021a](https://github.com/iamngoni/kanso/commit/14a021a1f20907eb93a9ae1c020421e2a815ba82))
- **mcp:** Add kanso-get-outline and kanso-resolve-wikilink tools ([489acc7](https://github.com/iamngoni/kanso/commit/489acc74adc4a8a20b72f6a8d7b54b8d3324a96b))
- **app:** An outline and backlinks beside the note, and wiki-links between notes ([aa87983](https://github.com/iamngoni/kanso/commit/aa87983703be72d9850c5cc38367265044ff93c6))
- **list:** Wiki-links read as their alias or title in note excerpts ([dd31e6f](https://github.com/iamngoni/kanso/commit/dd31e6f9f59a7afad74e8422db27024744d3e399))
- **list:** Math reads as its TeX in note excerpts ([7ffe1f5](https://github.com/iamngoni/kanso/commit/7ffe1f53a56f628077698d51e6bbf330ef895ae4))
- **cloud:** Publish a note as an encrypted page anyone with the link can read ([5f504da](https://github.com/iamngoni/kanso/commit/5f504da43a5069de48771180e5c938ca8b8bf2bc))
- **sync:** Seal and publish a note with its own key that never reaches the server ([99ef980](https://github.com/iamngoni/kanso/commit/99ef9805ee7937628839a484b42f39c0846eac84))
- **publish:** Publish a note to the web from its ⋯ menu, update it, or take it down ([28db263](https://github.com/iamngoni/kanso/commit/28db263732c6d77cbe482955bac2091ba21efec9))
- **store:** Daily notes, a Journal, Inbox and Templates, and tags nested with / ([8be2538](https://github.com/iamngoni/kanso/commit/8be2538476477481a700b18e4da74dadd5051a6b))
- **mcp:** Kanso-create-daily-note and kanso-create-from-template ([1e36f92](https://github.com/iamngoni/kanso/commit/1e36f9251fcc1170d954405e20236cde0df2fbee))
- **app:** Open at login on Windows, and as a macOS login item on macOS 13 and later ([4d93b00](https://github.com/iamngoni/kanso/commit/4d93b005aa815affde09893dcd0cfd6b46315f9e))
- **app:** Today's note, notes from templates, quick capture from any app and a tag tree ([028eb3f](https://github.com/iamngoni/kanso/commit/028eb3f7d33c6cba91ba84f3982071fde17b69e8))
- **cli:** Kanso capture --book saves to a notebook by its name or ID ([4b1ccc9](https://github.com/iamngoni/kanso/commit/4b1ccc962df0c81ea359b06fe525824b75078038))
- **editor:** Vim keys, with Normal, Insert and Visual modes in a note's text ([d19fc09](https://github.com/iamngoni/kanso/commit/d19fc096e6c2954969712807842a8a8aa0c8d6b5))
- **editor:** Vim keys gain H M L, ge, > and <, and . repeats Visual mode changes ([27e5964](https://github.com/iamngoni/kanso/commit/27e59645c93aa558f6c4aec47984b466ae8cf36c))
- **tasks:** Due dates show as chips in the preview and are tinted in the editor, with @ date completion ([6c91085](https://github.com/iamngoni/kanso/commit/6c9108560bca2fa856e22c0d873ebf74e0a95591))
- **tasks:** A Tasks view lists every task by when it's due, ticks them off in place and opens them at their line ([e436364](https://github.com/iamngoni/kanso/commit/e43636409cd846f1e2d2fc66cc0b9635c3b46e88))
- **reminders:** Tasks with a time send a notification when due, with Mark Done and Dismiss ([0e17f71](https://github.com/iamngoni/kanso/commit/0e17f71a1a1f2065e35062e7e553fe8d3a99cf7d))
- **replace:** Replace in all notes, with a live preview, options, and Undo replace ([ff34ddb](https://github.com/iamngoni/kanso/commit/ff34ddb4362f63af7587305af86543574f423b88))
- **history:** Note history shows what changed since each version, with each section to copy ([6e9bfc1](https://github.com/iamngoni/kanso/commit/6e9bfc161b9cbb4799be6e1ce011cd00f7cd7e0e))
- **images:** Drag an image's edge to resize it, size it with ![alt|300], or use Resize image… from the palette ([d8a3b0b](https://github.com/iamngoni/kanso/commit/d8a3b0b0ae7b04eaef89b4419611b813c618f3fa))
- **related:** Keep each note's embedding and the words read in its pictures on this device ([80b92c2](https://github.com/iamngoni/kanso/commit/80b92c2469a0a48252a410998e3aa8944d8b0fdd))
- **mcp:** Kanso-related-notes finds the notes closest in meaning to a note ([6a95ada](https://github.com/iamngoni/kanso/commit/6a95adaa7e83dcd6c80c46b89ad198ee5dcd9924))
- **graph:** See how notes link, around the open note in the inspector and across the whole library ([ef2cac7](https://github.com/iamngoni/kanso/commit/ef2cac740ee39b55f3213a572d76259a763387a4))
- **ocr:** Search finds notes by the words in their pictures, read with Vision on macOS ([1aaf62e](https://github.com/iamngoni/kanso/commit/1aaf62e3bb7abb4af28e06718c280a37aebd093e))

### Improved

- **app:** Typing never waits on the outline ([d95c0a0](https://github.com/iamngoni/kanso/commit/d95c0a06e73e743ade9627326f6de6458a880111))

### Fixed

- **app:** Plainer Local API and import wording ([76d2e0c](https://github.com/iamngoni/kanso/commit/76d2e0c6eba450fefca6ee37ddd30de5a8b63dc4))
- **release:** Build the Windows installer for dev versions too ([7b0fe29](https://github.com/iamngoni/kanso/commit/7b0fe29cb4e5e2f193f52fe0447fa6188ca59131))
- **render:** Draw Mermaid arrowheads smaller so they sit lightly on thin lines and clear the nodes ([58924a5](https://github.com/iamngoni/kanso/commit/58924a5e7f13b7b9d47dad967e008d80076c33ca))
- **preview:** Keep a drawing's actions in a quiet row under it that fades in on hover, never over the drawing ([f78cf44](https://github.com/iamngoni/kanso/commit/f78cf449c6f4aa8bee9dd2bc7dcbff74fb51cb9d))
- **export:** Export as PDF prints real pages on your default paper size, with margins ([c1de524](https://github.com/iamngoni/kanso/commit/c1de524adc81c3cc89246314e64e7ae44103cac2))
- **preview:** Text the Markdown parser can't handle shows as plain text instead of crashing ([77de02e](https://github.com/iamngoni/kanso/commit/77de02e393a94a29165553879a9546547ee43192))
- **cloud:** Retire unpublished links for good, serve only the checked upload, and let the owner take a note down ([ef1ef50](https://github.com/iamngoni/kanso/commit/ef1ef50d89bb1000d4154221816b3bc70a1be070))
- **sync:** Publish under a new link when the old one has expired or was retired, never reviving it ([fc31799](https://github.com/iamngoni/kanso/commit/fc317992986b9bf40027a718c7e6647ea6eb84dd))
- **api:** Keep a note's published link when the local API or MCP writes the note ([03563d8](https://github.com/iamngoni/kanso/commit/03563d8889ea63f3c84ed4c5663f0026424b712e))
- **publish:** Say your devices, not your Macs, in the Publish sheet ([225d09e](https://github.com/iamngoni/kanso/commit/225d09eba7c1439705e18c45ac18e27988cca1ee))
- **sync:** A Journal, Inbox or Templates notebook made on two devices offline is one notebook, not a conflict ([52d0a21](https://github.com/iamngoni/kanso/commit/52d0a21e35901a367d5e16540d0f8275275e86e4))
- **capture:** Quick notes are written in the editor's font and size from Settings ([ade79a3](https://github.com/iamngoni/kanso/commit/ade79a3dbdbce67252296674580bbd27f945710d))
- **app:** A new note made under a nested tag gets that tag ([b7803c0](https://github.com/iamngoni/kanso/commit/b7803c0af3f429c8800d48aaff1472bf175f8715))
- **reminders:** Reminders only reach the notification centre from an app bundle, and say when notifications are off ([185cb86](https://github.com/iamngoni/kanso/commit/185cb86e0bb9e086e30159d14910e859104476e5))
- **reminders:** Build cleanly on Windows and Linux, where the bundle check isn't needed ([f2f21f4](https://github.com/iamngoni/kanso/commit/f2f21f450fac76b56d88d195512e0d6320797209))
- **export:** Wiki-links in exported and published pages show as their label, and ![alt|300] images keep their width ([85f185e](https://github.com/iamngoni/kanso/commit/85f185e03dbf219002d00e2a6c325d7d706e1649))
- Say device instead of Mac on Windows and Linux, name each platform's secret store, and show Ctrl shortcuts off macOS ([13f3310](https://github.com/iamngoni/kanso/commit/13f33100387c6ccb6afd79839f87a7c459c08ee9))
- **related:** Say the same thing about the model download in the inspector and Settings ([29e496a](https://github.com/iamngoni/kanso/commit/29e496a64841f1345adfe057fd114ba7e833324f))

### Under the hood

- **release:** Publish each release automatically once every check passes ([9a01c36](https://github.com/iamngoni/kanso/commit/9a01c362e65336c3943af8baf09a32e74f05cce8))
- Describe Kanso on its own terms in the README, changelog and themes guide ([f54d468](https://github.com/iamngoni/kanso/commit/f54d46805699469586f30cef1d7f158b1eac2016))
- **readme:** Releases publish themselves once every check passes ([9a7f813](https://github.com/iamngoni/kanso/commit/9a7f813ec63c3330155e160a96302953940b1050))
- **readme:** The Mac download is notarized and updates itself ([45d8b8e](https://github.com/iamngoni/kanso/commit/45d8b8efa1a289e503d6f93377ae88b1f6125c5d))
- **app:** Measure typing latency with the outline open ([cb1bace](https://github.com/iamngoni/kanso/commit/cb1bacefb99d8b0a5b4c06c50ef4543f3916be36))
- **core:** Excerpt properties allow dots and brackets a note itself has ([618072b](https://github.com/iamngoni/kanso/commit/618072b9d757bd3be74962a8897cf61736ba4ed8))
- **app:** Snapshots date the notes they make by the demo clock ([437fb72](https://github.com/iamngoni/kanso/commit/437fb72cd09f6dbae676dc564d6e01167b1ededa))
- **e2e:** Give each end-to-end run its own Worker port so parallel runs never share one ([67c70c1](https://github.com/iamngoni/kanso/commit/67c70c1b2545f7cb6fb4515b73dc59b97336f774))
- **editor:** Snapshots of each Vim mode, and the typing benchmark with Vim keys on ([5771afc](https://github.com/iamngoni/kanso/commit/5771afc0d78cb634b6f48b7040c75291cafcadfc))
- **editor:** Vim snapshots show the caret as it is half a blink later ([30b016b](https://github.com/iamngoni/kanso/commit/30b016b19f14861025324408c6030471575d1992))
- **app:** Snapshots of the Tasks view, due chips, the replace sheet and the history changes ([436eeb2](https://github.com/iamngoni/kanso/commit/436eeb2499f2957b33f642c82de0ac40331c82f9))
- **editor:** Due dates stay tinted and @ completes dates with Vim keys on ([49b5243](https://github.com/iamngoni/kanso/commit/49b524353a1b046609663df6b77fcc29d07dbafd))
- Describe Kanso as it is now, with its newer shortcuts, kanso capture and device-neutral sync wording ([bff32fe](https://github.com/iamngoni/kanso/commit/bff32fe9adff9157a68681baa6127f0dd5fd8525))
- **changelog:** Release notes describe Kanso on its own terms ([c7be1b2](https://github.com/iamngoni/kanso/commit/c7be1b27da5fb98315d6756824dd95082611e174))

## [0.2.0](https://github.com/iamngoni/kanso/compare/v0.1.1...v0.2.0) - 2026-10-19

### New

- Close brackets, quotes and Markdown markers as you type ([4f383a9](https://github.com/iamngoni/kanso/commit/4f383a991bd116220cfe1848cb9345da67a87631))
- Tab and Shift-Tab indent list items, with what hangs under them ([6ea7aac](https://github.com/iamngoni/kanso/commit/6ea7aacf8eaa3624829e83373c36e91833525bf7))
- Format Table lines up a Markdown table ([3945fee](https://github.com/iamngoni/kanso/commit/3945fee951b59353366cbc4081e81f4e7cf18c6b))
- Smart paste: links over selected words, formatted text as Markdown, images as attachments ([0b023f6](https://github.com/iamngoni/kanso/commit/0b023f689f86e5dc15dc93744f054be2603723cc))
- Size attached images in the preview ([a1a7dfd](https://github.com/iamngoni/kanso/commit/a1a7dfdab76e0980cb2717da7b68edbd463ade5d))
- Word goal with a progress ring in focus mode ([8ee7204](https://github.com/iamngoni/kanso/commit/8ee7204287b572a1eafcd3b1b34db203217886e5))
- Typewriter scrolling, and the writing aids in Settings › Editor ([18e0836](https://github.com/iamngoni/kanso/commit/18e08363e197903d385588f2e0acc4b902173eef))

### Fixed

- Kanso comes to the front after Restart now installs an update, instead of opening behind other windows ([7982fed](https://github.com/iamngoni/kanso/commit/7982fed))

### Under the hood

- Snapshots and latency runs for the writing aids ([d3c7467](https://github.com/iamngoni/kanso/commit/d3c7467ec807eb709fd7a867e51f984a00f79a27))

## [0.1.1](https://github.com/iamngoni/kanso/compare/v0.1.0...v0.1.1) - 2026-10-19

### Fixed

- **release:** Brace variables before an ellipsis in release.sh ([6f819da](https://github.com/iamngoni/kanso/commit/6f819da6a4ae2585478dad4c27d0eceb4c308ab8))

## [0.1.0](https://github.com/iamngoni/kanso/commits/v0.1.0) - 2026-10-19

The first release: a calm, local-first Markdown notes app built on GPUI
for macOS, Windows and Linux. What follows sums up
the history so far; every commit is in [the history up to
v0.1.0](https://github.com/iamngoni/kanso/commits/v0.1.0).

### New

- Kanso updates itself from signed releases, says clearly when an update is ready, and shows what's new after it installs ([6bb7570](https://github.com/iamngoni/kanso/commit/6bb7570))
- Notes, notebooks within notebooks, tags and statuses in a SQLite store with trigram search, revisions and daily backups ([169d0e4](https://github.com/iamngoni/kanso/commit/169d0e4))
- Import a PouchDB notes library, inside the app or from the command line, keeping Kanso's own edits on a re-import ([e34c7bc](https://github.com/iamngoni/kanso/commit/e34c7bc), [e5479c4](https://github.com/iamngoni/kanso/commit/e5479c4))
- The editor with a live preview, side by side or alone, a draggable split and focus mode ([a531b38](https://github.com/iamngoni/kanso/commit/a531b38), [d0e68c2](https://github.com/iamngoni/kanso/commit/d0e68c2))
- The note list grouped by day, with search matches shown in the excerpts ([d7744b9](https://github.com/iamngoni/kanso/commit/d7744b9))
- The command palette and Open Note ([d268159](https://github.com/iamngoni/kanso/commit/d268159))
- Backlinks and each note's history, with a way back to any earlier version ([4b45548](https://github.com/iamngoni/kanso/commit/4b45548))
- Export a note, or every note, as Markdown ([0e35966](https://github.com/iamngoni/kanso/commit/0e35966))
- A local API and a native MCP endpoint for agents and other clients, compatible with popular local-server clients ([88ba474](https://github.com/iamngoni/kanso/commit/88ba474), [d5fd4d5](https://github.com/iamngoni/kanso/commit/d5fd4d5))
- Accounts with end-to-end encrypted sync between your computers, recovery keys, and a Settings page for the account and its devices ([f2d5a5a](https://github.com/iamngoni/kanso/commit/f2d5a5a), [666c77f](https://github.com/iamngoni/kanso/commit/666c77f), [b5fbac9](https://github.com/iamngoni/kanso/commit/b5fbac9))
- Settings inside the main window, and opening at login ([157cd61](https://github.com/iamngoni/kanso/commit/157cd61))
- Twenty themes, separate UI, syntax and preview themes, and importing theme packages and Zed themes ([1f9d3ff](https://github.com/iamngoni/kanso/commit/1f9d3ff), [8f050bc](https://github.com/iamngoni/kanso/commit/8f050bc))
- Code blocks colored in 23 languages, with a copy button ([3709232](https://github.com/iamngoni/kanso/commit/3709232), [27f54a0](https://github.com/iamngoni/kanso/commit/27f54a0))
- An app icon: an ink ensō with a vermilion seal ([c403439](https://github.com/iamngoni/kanso/commit/c403439))
- Builds for Windows and Linux as well as macOS ([62ec295](https://github.com/iamngoni/kanso/commit/62ec295))

### Improved

- A warm graphite and paper look and a calmer writing surface ([2e1b76b](https://github.com/iamngoni/kanso/commit/2e1b76b))
- Typing stays fast in long notes ([e0d7ad0](https://github.com/iamngoni/kanso/commit/e0d7ad0))

### Fixed

- Typed text is never lost, through saves, syncs and a note's revision moving underneath it ([f6a6583](https://github.com/iamngoni/kanso/commit/f6a6583), [9442143](https://github.com/iamngoni/kanso/commit/9442143))
- Sync keeps local edits through its own echoes, lost answers and restores ([7b0d30d](https://github.com/iamngoni/kanso/commit/7b0d30d), [652b439](https://github.com/iamngoni/kanso/commit/652b439), [3ab3f29](https://github.com/iamngoni/kanso/commit/3ab3f29))
- Recovery keys are kept until they're saved, and replaced with every password change ([645c5cd](https://github.com/iamngoni/kanso/commit/645c5cd), [ff8f89f](https://github.com/iamngoni/kanso/commit/ff8f89f))
- Auth keys, proofs and tokens stay out of logs and freed memory ([3bda3bd](https://github.com/iamngoni/kanso/commit/3bda3bd))
- Three rounds of review findings in the main window ([61f005e](https://github.com/iamngoni/kanso/commit/61f005e), [f325bfc](https://github.com/iamngoni/kanso/commit/f325bfc), [18b218d](https://github.com/iamngoni/kanso/commit/18b218d))

### Under the hood

- Property tests with a pinned seed across the workspace, and PNG snapshots of the real views ([ff9515f](https://github.com/iamngoni/kanso/commit/ff9515f))
- A release workflow that builds every platform into a draft release ([62ec295](https://github.com/iamngoni/kanso/commit/62ec295))
