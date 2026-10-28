package app.kanso.kanso_dictation

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import android.speech.SpeechRecognizer
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.embedding.engine.plugins.activity.ActivityAware
import io.flutter.embedding.engine.plugins.activity.ActivityPluginBinding
import io.flutter.plugin.common.PluginRegistry
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * The platform's speech recognizer. The device's own recognizer is used where
 * Android 12 and later has one; otherwise the system's, asked to work offline.
 *
 * `SpeechRecognizer` is for the main thread only, so every call that touches
 * one runs there.
 */
class KansoDictationPlugin :
    FlutterPlugin,
    ActivityAware,
    PluginRegistry.RequestPermissionsResultListener,
    DictationHostApi {
    private var context: Context? = null
    private var activity: Activity? = null
    private var binding: ActivityPluginBinding? = null
    private var session: DictationSession? = null
    private var asking: CompletableDeferred<Unit>? = null
    private var scope: CoroutineScope? = null
    private val events = Channel<DictationEvent>(Channel.UNLIMITED)

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        context = binding.applicationContext
        DictationHostApi.setUp(binding.binaryMessenger, this)
        val flutterApi = DictationFlutterApi(binding.binaryMessenger)
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
        this.scope = scope
        // One consumer sends the events one after the other, in order.
        scope.launch {
            for (event in events) {
                try {
                    flutterApi.onEvent(event)
                } catch (error: Exception) {
                    // A Dart side that is gone has no use for the event.
                }
            }
        }
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        DictationHostApi.setUp(binding.binaryMessenger, null)
        session?.cancel()
        session = null
        scope?.cancel()
        scope = null
        context = null
    }

    override fun onAttachedToActivity(binding: ActivityPluginBinding) {
        activity = binding.activity
        this.binding = binding
        binding.addRequestPermissionsResultListener(this)
    }

    override fun onDetachedFromActivityForConfigChanges() = onDetachedFromActivity()

    override fun onReattachedToActivityForConfigChanges(binding: ActivityPluginBinding) =
        onAttachedToActivity(binding)

    override fun onDetachedFromActivity() {
        binding?.removeRequestPermissionsResultListener(this)
        binding = null
        activity = null
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ): Boolean {
        if (requestCode != REQUEST_CODE) return false
        asking?.complete(Unit)
        asking = null
        return true
    }

    override suspend fun availability(localeTag: String?): DictationAvailability =
        withContext(Dispatchers.Main) {
            val context = context
            val permission = permission()
            val language = languageName(localeTag)
            when {
                context == null -> unavailable(permission, language, DictationUnavailableReason.NO_RECOGNIZER)
                Build.VERSION.SDK_INT >= Build.VERSION_CODES.S &&
                    SpeechRecognizer.isOnDeviceRecognitionAvailable(context) ->
                    DictationAvailability(available = true, onDevice = true, permission = permission, language = language)
                SpeechRecognizer.isRecognitionAvailable(context) ->
                    DictationAvailability(available = true, onDevice = false, permission = permission, language = language)
                else -> unavailable(permission, language, DictationUnavailableReason.NO_RECOGNIZER)
            }
        }

    override suspend fun requestPermission(): DictationPermission {
        val activity = activity ?: return permission()
        if (permission() == DictationPermission.GRANTED) return DictationPermission.GRANTED
        val answer = CompletableDeferred<Unit>()
        asking?.complete(Unit)
        asking = answer
        withContext(Dispatchers.Main) {
            preferences().edit().putBoolean(ASKED, true).apply()
            activity.requestPermissions(arrayOf(Manifest.permission.RECORD_AUDIO), REQUEST_CODE)
        }
        answer.await()
        return permission()
    }

    override suspend fun start(localeTag: String?) {
        withContext(Dispatchers.Main) {
            val context = context
            val running = session
            when {
                context == null -> refuse(DictationErrorCode.NOT_AVAILABLE)
                running != null && !running.finished -> refuse(DictationErrorCode.BUSY)
                permission() != DictationPermission.GRANTED -> refuse(DictationErrorCode.PERMISSION_DENIED)
                else -> {
                    val started = DictationSession(context, localeTag) { events.trySend(it) }
                    if (started.begin()) {
                        session = started
                    } else {
                        refuse(DictationErrorCode.NOT_AVAILABLE)
                    }
                }
            }
        }
    }

    override fun stop() {
        session?.stop()
    }

    override fun cancel() {
        session?.cancel()
    }

    override fun openSettings() {
        val context = context ?: return
        val intent =
            Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.parse("package:${context.packageName}"))
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        context.startActivity(intent)
    }

    /** A session that never started: the error, then its end. */
    private fun refuse(code: DictationErrorCode) {
        events.trySend(DictationEvent(kind = DictationEventKind.ERROR, error = code))
        events.trySend(DictationEvent(kind = DictationEventKind.ENDED))
    }

    private fun unavailable(
        permission: DictationPermission,
        language: String,
        reason: DictationUnavailableReason,
    ) =
        DictationAvailability(
            available = false,
            onDevice = false,
            permission = permission,
            language = language,
            reason = reason,
        )

    /** The language's name in the device's language. */
    private fun languageName(localeTag: String?): String {
        val locale = localeTag?.let { java.util.Locale.forLanguageTag(it) } ?: java.util.Locale.getDefault()
        return locale.getDisplayName(java.util.Locale.getDefault())
    }

    /**
     * Android cannot tell "never asked" from "refused for good", so the plugin
     * remembers that it asked.
     */
    private fun permission(): DictationPermission {
        val context = context ?: return DictationPermission.DENIED
        if (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) ==
            PackageManager.PERMISSION_GRANTED
        ) {
            return DictationPermission.GRANTED
        }
        return if (preferences().getBoolean(ASKED, false)) {
            DictationPermission.DENIED
        } else {
            DictationPermission.NOT_DETERMINED
        }
    }

    private fun preferences() = context!!.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    private companion object {
        const val REQUEST_CODE = 0x6469
        const val PREFERENCES = "kanso_dictation"
        const val ASKED = "asked_for_microphone"
    }
}
