# Recordings

Kanso can record your microphone, the sound from another app, or both. A recording is written to
disk as it is made, in Ogg Opus parts. If Kanso closes unexpectedly, open it again and choose
**Recover** to keep the parts that were written. A recovered recording can be played while the rest
of the session is missing.

## Choose what to record

- **Microphone** records the selected input, such as your built-in mic or headset.
- **Microphone + System Audio** records your voice and a meeting app together. Kanso keeps them on
  separate channels so the transcript can label paragraphs **You** and **Meeting**.
- **System Audio** records the chosen app without your microphone.

On macOS, choose a running app or the whole system. App capture leaves out other apps' notification
sounds. On Windows, Kanso records the chosen output's loopback audio. On Linux, system audio is
available when PulseAudio or PipeWire exposes a monitor source. If it is not available, the system
audio choices are disabled; microphone recording can still work.

You can save a recording in a new note or in the open note. After you stop, Kanso transcribes it on
this computer using its downloaded speech model. The transcript's timestamps seek to that moment in
the player. Two-channel recordings can label a paragraph **You** or **Meeting** by which channel is
louder. This is a guide, not a person-recognition system; crosstalk and voices picked up by the
microphone can make a label wrong.

## Permissions and consent

The operating system asks before Kanso can use the microphone. You can change that choice in the
system's privacy settings:

- **macOS:** Microphone permission is needed for the microphone. Recording system audio also needs
  Screen & System Audio Recording permission. Kanso records audio only, not the screen. You can
  choose one app so other apps' sound is left out. The system shows its own recording indicators.
- **Windows:** Allow microphone access for Kanso in **Settings › Privacy & security › Microphone**.
  Windows provides system audio through the selected output; there is no separate screen-capture
  permission.
- **Linux:** Allow microphone access through your desktop's audio settings. System audio needs a
  PulseAudio or PipeWire monitor source; some desktop or sound-server setups do not provide one.

When you choose system audio, Kanso reminds you that other people's voices may be recorded. Tell
everyone in the meeting and follow the rules where you are. A reminder cannot provide consent for
you.

## Where the audio goes

Audio and transcripts stay on this computer unless you sync or choose a summary provider. Sync
encrypts attached audio before it leaves the computer. The sync service can see that files exist,
their sizes and when they were uploaded, but not their names, contents or transcripts.

While recording, the working folder is private to your user account. The library database and its
local attachments are not encrypted by Kanso at rest. Use your operating system's disk encryption
to protect them if someone gets access to the computer or its disk.

### Locked notes

Unlock a note before recording into it. Kanso seals recordings attached to a locked note with that
note's lock key. The name, type, size and existence of an attachment can still be visible to sync.
Locking a note does not change older copies that may already be in a backup.

### Summaries

Summaries are an explicit action and are off until you set up an AI provider. Kanso sends the
timestamped transcript, note title and date, not the audio or other notes. An agent can read only
notes you link to it; it does not get search or the notes server for this action. A local model keeps
the text on this computer. If you choose an online provider, its retention rules apply to the text.
The confirmation sheet names what will be sent and where; cancel there to send nothing.

## Delete a recording

Delete the recording from its note when you no longer want the audio. Kanso removes the live file
attachment. If **Keep transcript only** is selected, it removes the audio after transcription
succeeds and keeps the words in the note. A failed transcription keeps the audio so you can try
again.

Deleting an attachment does not erase earlier copies in daily backups. Those copies are removed
when the backup that contains them expires. Sync may also retain encrypted copies under the
retention described in [Accounts](ACCOUNTS.md#what-the-server-can-and-cant-see). Keep this in mind before
recording a meeting or deleting its audio.

Publishing leaves recordings out by default. You can choose to include them when publishing a
notebook. The transcript can still be part of the note you publish; check the note before sharing
it.
