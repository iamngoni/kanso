//! System audio on macOS: ScreenCaptureKit (macOS 13+) with video thrown
//! away. One app's audio, or everything the Mac plays, arrives as mono at
//! 48 kHz and goes into a ring buffer the recorder drains, the way the
//! microphone's does.
//!
//! The Screen Recording permission is asked for by [`request_access`] and by
//! nothing else here: [`running_apps`] reads the workspace's list of
//! running apps (no permission), and [`access`] only looks. Only starting a
//! capture touches ScreenCaptureKit, and the sheet gets there only after
//! the person chose system audio and the permission is on.
//!
//! No video is written. ScreenCaptureKit needs a display to build a filter
//! around, so the stream is told to make a 2 by 2 picture once a second,
//! and the picture is dropped the moment it arrives.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use block2::RcBlock;
use dispatch2::{DispatchQueue, DispatchRetained};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{AllocAnyThread, DefinedClass, define_class, msg_send};
use objc2_app_kit::{NSApplicationActivationPolicy, NSWorkspace};
use objc2_core_media::{CMSampleBuffer, CMTime};
use objc2_foundation::{NSArray, NSError, NSObject, NSObjectProtocol};
use objc2_screen_capture_kit::{
    SCContentFilter, SCRunningApplication, SCShareableContent, SCStream, SCStreamConfiguration,
    SCStreamDelegate, SCStreamOutput, SCStreamOutputType,
};

use crate::error::AudioError;
use crate::mic::Failure;
use crate::model::SAMPLE_RATE;
use crate::system_apps::{
    CaptureApp, PcmBuffer, ScreenAccess, append_mono, denied_words, picker_order,
};
use crate::system_audio::{
    Availability, SystemAudioBackend, SystemAudioDevice, SystemAudioSource, SystemAudioTarget,
    Unavailable,
};

/// How long to wait for ScreenCaptureKit to answer before giving up.
const ANSWER: Duration = Duration::from_secs(10);
/// Seconds of sound the ring buffer holds between two polls.
const BUFFER_SECS: usize = 4;
/// ScreenCaptureKit's error code for a person (or the system) declining the
/// permission.
const USER_DECLINED: isize = -3801;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

/// Whether Kanso may record other apps now. Never asks.
// no-prop-test: a system call.
pub fn access() -> ScreenAccess {
    // SAFETY: a plain query with no arguments.
    if unsafe { CGPreflightScreenCaptureAccess() } {
        ScreenAccess::Allowed
    } else {
        ScreenAccess::NotAllowed
    }
}

/// Asks for Screen Recording: the system shows its question the first time
/// and answers `NotAllowed` until the person says yes (and, on some
/// versions, until Kanso is reopened). Call it only when the person has
/// turned system audio on.
// no-prop-test: a system call.
pub fn request_access() -> ScreenAccess {
    // SAFETY: a plain call with no arguments.
    if unsafe { CGRequestScreenCaptureAccess() } {
        ScreenAccess::Allowed
    } else {
        ScreenAccess::NotAllowed
    }
}

/// The apps that can be picked: the ones with a Dock icon, apart from
/// Kanso. Needs no permission.
// no-prop-test: reads the machine; `picker_order` has the properties.
pub fn running_apps() -> Vec<CaptureApp> {
    let ours = std::process::id() as i32;
    let apps = NSWorkspace::sharedWorkspace().runningApplications();
    let found = apps
        .iter()
        .filter(|app| {
            app.activationPolicy() == NSApplicationActivationPolicy::Regular
                && !app.isTerminated()
                && app.processIdentifier() != ours
        })
        .filter_map(|app| {
            Some(CaptureApp {
                bundle_id: app.bundleIdentifier()?.to_string(),
                name: app.localizedName()?.to_string(),
            })
        })
        .collect();
    picker_order(found)
}

struct SinkState {
    producer: Mutex<rtrb::Producer<f32>>,
    dropped: Arc<AtomicU64>,
    failure: Failure,
    /// A format the sink can't read was seen once; say so once.
    unreadable: Mutex<bool>,
}

define_class!(
    /// Receives the stream's sound on its queue and the stream's failure.
    #[unsafe(super(NSObject))]
    #[name = "KansoSystemAudioSink"]
    #[ivars = SinkState]
    struct Sink;

    unsafe impl NSObjectProtocol for Sink {}

    unsafe impl SCStreamOutput for Sink {
        #[unsafe(method(stream:didOutputSampleBuffer:ofType:))]
        unsafe fn did_output(
            &self,
            _stream: &SCStream,
            sample: &CMSampleBuffer,
            kind: SCStreamOutputType,
        ) {
            // The pictures are dropped here; only audio goes on.
            if kind == SCStreamOutputType::Audio {
                self.take(sample);
            }
        }
    }

    unsafe impl SCStreamDelegate for Sink {
        #[unsafe(method(stream:didStopWithError:))]
        unsafe fn did_stop(&self, _stream: &SCStream, error: &NSError) {
            self.fail(format!("{}", error.localizedDescription()));
        }
    }
);

impl Sink {
    fn new(state: SinkState) -> Retained<Self> {
        let this = Self::alloc().set_ivars(state);
        // SAFETY: `NSObject`'s `init`.
        unsafe { msg_send![super(this), init] }
    }

    fn fail(&self, why: String) {
        if let Ok(mut slot) = self.ivars().failure.lock() {
            slot.get_or_insert(why);
        }
    }

    /// Mono samples from one audio sample buffer into the ring buffer.
    fn take(&self, sample: &CMSampleBuffer) {
        let mut mono = Vec::new();
        if let Err(why) = read_mono(sample, &mut mono) {
            let mut said = self.ivars().unreadable.lock().unwrap();
            if !*said {
                *said = true;
                drop(said);
                self.fail(why);
            }
            return;
        }
        let state = self.ivars();
        let Ok(mut producer) = state.producer.lock() else {
            return;
        };
        for sample in mono {
            if producer.push(sample).is_err() {
                state.dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// The samples of an audio sample buffer as one mono channel. ScreenCaptureKit
/// sends 32-bit float, planar or interleaved; anything else is refused.
// no-prop-test: needs a live sample buffer; `append_mono` has the properties.
fn read_mono(sample: &CMSampleBuffer, out: &mut Vec<f32>) -> Result<(), String> {
    use objc2_core_audio_types::{
        AudioBuffer, AudioBufferList, kAudioFormatFlagIsFloat, kAudioFormatFlagIsNonInterleaved,
    };
    use objc2_core_media::CMAudioFormatDescriptionGetStreamBasicDescription;

    // SAFETY: Core Media calls on a sample buffer the framework handed us; the
    // pointers stay inside this function, and the block buffer that owns the
    // sound is kept alive (`block`) until the samples are copied out.
    unsafe {
        let format = sample
            .format_description()
            .ok_or("the system sent sound with no format")?;
        let asbd = CMAudioFormatDescriptionGetStreamBasicDescription(&format);
        let asbd = asbd
            .as_ref()
            .ok_or("the system sent sound with no format")?;
        if asbd.mFormatFlags & kAudioFormatFlagIsFloat == 0 || asbd.mBitsPerChannel != 32 {
            return Err("the system sent sound in a format Kanso can't read".into());
        }
        let planar = asbd.mFormatFlags & kAudioFormatFlagIsNonInterleaved != 0;

        let mut needed = 0usize;
        let status = sample.audio_buffer_list_with_retained_block_buffer(
            &mut needed,
            std::ptr::null_mut(),
            0,
            None,
            None,
            0,
            std::ptr::null_mut(),
        );
        if needed == 0 || (status != 0 && status != -12737) {
            return Err(format!("the system's sound couldn't be read ({status})"));
        }
        // 8-byte aligned storage for the variable-length list.
        let mut storage = vec![0u64; needed.div_ceil(8)];
        let list = storage.as_mut_ptr().cast::<AudioBufferList>();
        let mut block: *mut objc2_core_media::CMBlockBuffer = std::ptr::null_mut();
        let status = sample.audio_buffer_list_with_retained_block_buffer(
            std::ptr::null_mut(),
            list,
            needed,
            None,
            None,
            0,
            &mut block,
        );
        if status != 0 {
            return Err(format!("the system's sound couldn't be read ({status})"));
        }
        // Releases the block buffer when it goes out of scope.
        let _block =
            std::ptr::NonNull::new(block).map(|b| objc2_core_foundation::CFRetained::from_raw(b));

        let count = (*list).mNumberBuffers as usize;
        let buffers: &[AudioBuffer] = std::slice::from_raw_parts((*list).mBuffers.as_ptr(), count);
        let mut views = Vec::with_capacity(count);
        for buffer in buffers {
            if buffer.mData.is_null() {
                continue;
            }
            let floats = buffer.mDataByteSize as usize / 4;
            let samples = std::slice::from_raw_parts(buffer.mData.cast::<f32>(), floats);
            let channels = if planar {
                1
            } else {
                (buffer.mNumberChannels as usize).max(1)
            };
            views.push(PcmBuffer { samples, channels });
        }
        append_mono(&views, out);
        Ok(())
    }
}

/// Keeps the stream alive; dropping it stops the capture.
struct Running {
    stream: Retained<SCStream>,
    _sink: Retained<Sink>,
    _queue: DispatchRetained<DispatchQueue>,
}

impl Drop for Running {
    fn drop(&mut self) {
        let (tx, rx) = mpsc::channel();
        let done = RcBlock::new(move |_: *mut NSError| {
            let _ = tx.send(());
        });
        // SAFETY: stops a stream this value started; the handler only sends.
        unsafe { self.stream.stopCaptureWithCompletionHandler(Some(&done)) };
        let _ = rx.recv_timeout(Duration::from_secs(3));
    }
}

/// What the system said about a failed call, as an [`AudioError`].
fn error_for(error: &NSError, what: &str) -> AudioError {
    if error.code() == USER_DECLINED {
        return AudioError::ScreenRecordingDenied;
    }
    AudioError::SystemAudio(format!("{what}: {}", error.localizedDescription()))
}

/// ScreenCaptureKit as a system audio backend. Its targets are apps
/// ([`SystemAudioTarget::App`] by bundle ID), or everything the Mac plays
/// ([`SystemAudioTarget::Default`]); [`running_apps`] lists the apps, and it
/// has no devices.
pub struct ScreenCaptureKit;

impl SystemAudioBackend for ScreenCaptureKit {
    fn availability(&self) -> Availability {
        match access() {
            ScreenAccess::Allowed => Availability::Available,
            ScreenAccess::NotAllowed => {
                Availability::Unavailable(Unavailable::Denied(denied_words()))
            }
        }
    }

    fn devices(&self) -> Vec<SystemAudioDevice> {
        Vec::new()
    }

    fn open(&self, target: &SystemAudioTarget) -> Result<Box<dyn SystemAudioSource>, AudioError> {
        Ok(Box::new(start(target)?))
    }
}

/// An open ScreenCaptureKit capture: mono 48 kHz sound out of a ring buffer.
struct MacSource {
    samples: rtrb::Consumer<f32>,
    dropped: Arc<AtomicU64>,
    failure: Failure,
    _running: Running,
}

impl SystemAudioSource for MacSource {
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn read(&mut self, out: &mut Vec<f32>) {
        let available = self.samples.slots();
        if available == 0 {
            return;
        }
        if let Ok(chunk) = self.samples.read_chunk(available) {
            let (a, b) = chunk.as_slices();
            out.extend_from_slice(a);
            out.extend_from_slice(b);
            chunk.commit_all();
        }
    }

    fn take_dropped(&mut self) -> u64 {
        self.dropped.swap(0, Ordering::Relaxed)
    }

    fn failure(&self) -> Option<String> {
        self.failure.lock().ok().and_then(|f| f.clone())
    }
}

/// Starts capturing `target`'s audio. Fails with
/// [`AudioError::ScreenRecordingDenied`] when Kanso isn't allowed, and with a
/// plain message when the chosen app isn't running.
// no-prop-test: needs a permission and a running app; `append_mono` and `GapFiller` have the properties.
fn start(target: &SystemAudioTarget) -> Result<MacSource, AudioError> {
    let failure: Failure = Failure::default();
    // The permission is asked for in the sheet; if it is off by now, stop
    // here rather than let the system ask in the middle of a recording.
    if access() != ScreenAccess::Allowed {
        return Err(AudioError::ScreenRecordingDenied);
    }
    let (tx, rx) = mpsc::channel();
    let handler = RcBlock::new(
        move |content: *mut SCShareableContent, error: *mut NSError| {
            // SAFETY: the system passes either a live object or null.
            let answer = unsafe {
                match (content.as_ref(), error.as_ref()) {
                    (Some(content), _) => Ok(Retained::retain(
                        content as *const _ as *mut SCShareableContent,
                    )),
                    (None, Some(error)) => {
                        Err(error_for(error, "couldn't list what can be captured"))
                    }
                    (None, None) => Err(AudioError::SystemAudio(
                        "the system didn't say what can be captured".into(),
                    )),
                }
            };
            let _ = tx.send(answer);
        },
    );
    // SAFETY: the handler is called once, from a system thread.
    unsafe { SCShareableContent::getShareableContentWithCompletionHandler(&handler) };
    let content = match rx.recv_timeout(ANSWER) {
        Ok(answer) => {
            answer?.ok_or_else(|| AudioError::SystemAudio("nothing to capture".into()))?
        }
        Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
            return Err(AudioError::SystemAudio(
                "the system didn't answer in time".into(),
            ));
        }
    };

    // SAFETY: plain reads of the content object.
    let (display, apps) = unsafe { (content.displays().firstObject(), content.applications()) };
    let display = display.ok_or_else(|| {
        AudioError::SystemAudio("there's no display to attach the capture to".into())
    })?;
    let none: Retained<NSArray<objc2_screen_capture_kit::SCWindow>> = NSArray::new();
    // SAFETY: filters built from objects ScreenCaptureKit just gave us.
    let filter = unsafe {
        match target {
            SystemAudioTarget::App(bundle_id) => {
                let app: Option<Retained<SCRunningApplication>> = apps
                    .iter()
                    .find(|a| a.bundleIdentifier().to_string() == *bundle_id);
                let app = app.ok_or_else(|| {
                    AudioError::SystemAudio(
                        "That app isn't running. Open it, then start again.".into(),
                    )
                })?;
                let included = NSArray::from_retained_slice(&[app]);
                SCContentFilter::initWithDisplay_includingApplications_exceptingWindows(
                    SCContentFilter::alloc(),
                    &display,
                    &included,
                    &none,
                )
            }
            SystemAudioTarget::Device(_) => {
                return Err(AudioError::SystemAudio(
                    "this system records an app, not a device".into(),
                ));
            }
            SystemAudioTarget::Default => {
                let excluded: Retained<NSArray<SCRunningApplication>> = NSArray::new();
                SCContentFilter::initWithDisplay_excludingApplications_exceptingWindows(
                    SCContentFilter::alloc(),
                    &display,
                    &excluded,
                    &none,
                )
            }
        }
    };

    // SAFETY: setters on a configuration this function owns.
    let config = unsafe {
        let config = SCStreamConfiguration::new();
        config.setCapturesAudio(true);
        config.setExcludesCurrentProcessAudio(true);
        config.setSampleRate(SAMPLE_RATE as isize);
        config.setChannelCount(1);
        // No video: the smallest picture, once a second, dropped on arrival.
        config.setWidth(2);
        config.setHeight(2);
        config.setMinimumFrameInterval(CMTime::new(1, 1));
        config.setShowsCursor(false);
        config
    };

    let rate = SAMPLE_RATE;
    let (producer, samples) = rtrb::RingBuffer::new(rate as usize * BUFFER_SECS);
    let dropped = Arc::new(AtomicU64::new(0));
    let sink = Sink::new(SinkState {
        producer: Mutex::new(producer),
        dropped: dropped.clone(),
        failure: failure.clone(),
        unreadable: Mutex::new(false),
    });
    let queue = DispatchQueue::new("app.kanso.system-audio", None);
    // SAFETY: the sink adopts both protocols; the queue outlives the stream
    // (both are held by `Running`).
    let stream = unsafe {
        let delegate: &ProtocolObject<dyn SCStreamDelegate> = ProtocolObject::from_ref(&*sink);
        let stream = SCStream::initWithFilter_configuration_delegate(
            SCStream::alloc(),
            &filter,
            &config,
            Some(delegate),
        );
        let output: &ProtocolObject<dyn SCStreamOutput> = ProtocolObject::from_ref(&*sink);
        for kind in [SCStreamOutputType::Audio, SCStreamOutputType::Screen] {
            stream
                .addStreamOutput_type_sampleHandlerQueue_error(output, kind, Some(&queue))
                .map_err(|e| error_for(&e, "couldn't listen to the app"))?;
        }
        stream
    };
    let (tx, rx) = mpsc::channel();
    let started = RcBlock::new(move |error: *mut NSError| {
        // SAFETY: null or a live error.
        let answer = unsafe { error.as_ref() }.map(|e| error_for(e, "couldn't start the capture"));
        let _ = tx.send(answer);
    });
    // SAFETY: the handler is called once.
    unsafe { stream.startCaptureWithCompletionHandler(Some(&started)) };
    let running = Running {
        stream,
        _sink: sink,
        _queue: queue,
    };
    match rx.recv_timeout(ANSWER) {
        Ok(None) => {}
        Ok(Some(error)) => return Err(error),
        Err(_) => {
            return Err(AudioError::SystemAudio(
                "the capture didn't start in time".into(),
            ));
        }
    }
    Ok(MacSource {
        samples,
        dropped,
        failure,
        _running: running,
    })
}
