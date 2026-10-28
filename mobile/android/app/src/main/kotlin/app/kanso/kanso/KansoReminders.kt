package app.kanso.kanso

import android.Manifest
import android.app.Activity
import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

/**
 * Reminders on Android: the `app.kanso/reminders` method channel
 * (docs/journal-tasks.md). Dart decides what is due and sends the words; this
 * keeps them, sets one alarm per reminder, and posts the notification when it
 * goes off ([KansoReminderReceiver]).
 *
 * A reminder keeps only its words, its fire time and `key`, `noteId`, `line`.
 * A tap on the notification (`open`), on Mark Done or on Dismiss is answered
 * once: live, over the channel, when the app runs and can take it, or kept in
 * SharedPreferences until `takeResponses`. Never both, as Dart applies
 * whatever it receives exactly once.
 *
 * Alarms are exact (`setExactAndAllowWhileIdle`) where the system lets the app
 * set them, otherwise inexact (`setAndAllowWhileIdle`, which can be minutes
 * late in Doze). The app doesn't declare `SCHEDULE_EXACT_ALARM`: Play keeps
 * it for alarm and calendar apps and, from Android 14, it is off by default
 * for new installs; a reminder a few minutes late is no harm.
 */
object KansoReminders {
    /** The method channel the Dart side asks on. */
    const val CHANNEL = "app.kanso/reminders"

    const val NOTIFICATION_CHANNEL = "reminders"
    const val NOTIFICATION_TAG = "kanso-reminder"

    const val ACTION_FIRE = "app.kanso.kanso.REMINDER_FIRE"
    const val ACTION_OPEN = "app.kanso.kanso.REMINDER_OPEN"
    const val ACTION_MARK_DONE = "app.kanso.kanso.REMINDER_MARK_DONE"
    const val ACTION_DISMISS = "app.kanso.kanso.REMINDER_DISMISS"

    const val EXTRA_KEY = "app.kanso.kanso.reminder.key"
    const val EXTRA_NOTE_ID = "app.kanso.kanso.reminder.noteId"
    const val EXTRA_LINE = "app.kanso.kanso.reminder.line"

    private const val PREFERENCES = "kanso_reminders"
    private const val SCHEDULED = "scheduled"
    private const val RESPONSES = "responses"
    private const val ASKED = "asked_permission"
    private const val PERMISSION_REQUEST = 4801

    private val lock = Any()
    private val main = Handler(Looper.getMainLooper())

    private var channel: MethodChannel? = null
    private var activity: Activity? = null
    private var pendingPermission: MethodChannel.Result? = null

    /** Whether the activity is in front; a button's answer goes live only then. */
    @Volatile
    var inFront: Boolean = false

    /** Answers the `app.kanso/reminders` channel; call from `configureFlutterEngine`. */
    @JvmStatic
    fun register(activity: Activity, engine: FlutterEngine) {
        this.activity = activity
        ensureChannel(activity)
        val channel = MethodChannel(engine.dartExecutor.binaryMessenger, CHANNEL)
        this.channel = channel
        channel.setMethodCallHandler { call, result ->
            val context = activity.applicationContext
            when (call.method) {
                "permission" -> result.success(permission(context))
                "requestPermission" -> requestPermission(activity, result)
                "schedule" -> {
                    schedule(context, reminders(call.arguments))
                    result.success(null)
                }
                "show" -> {
                    for (reminder in reminders(call.arguments)) {
                        post(context, reminder)
                    }
                    result.success(null)
                }
                "cancelAll" -> {
                    cancelAll(context)
                    result.success(null)
                }
                "takeResponses" -> result.success(takeResponses(context).map { it.toMap() })
                else -> result.notImplemented()
            }
        }
    }

    /** The engine is going away: answers are kept from here on. */
    @JvmStatic
    fun unregister() {
        channel?.setMethodCallHandler(null)
        channel = null
        activity = null
        pendingPermission?.success(false)
        pendingPermission = null
    }

    /** Hands `onRequestPermissionsResult` of the activity on. */
    @JvmStatic
    fun onPermissionResult(requestCode: Int, grantResults: IntArray) {
        if (requestCode != PERMISSION_REQUEST) {
            return
        }
        val granted = grantResults.firstOrNull() == PackageManager.PERMISSION_GRANTED
        pendingPermission?.success(granted)
        pendingPermission = null
    }

    /**
     * A tap on the notification opened the activity with [intent]; answers it
     * once and takes the reminder out of the intent, so a recreated activity
     * doesn't answer again.
     */
    @JvmStatic
    fun handleOpen(context: Context, intent: Intent?) {
        if (intent?.action != ACTION_OPEN) {
            return
        }
        val answer = answer(intent, ReminderRules.OPEN)
        intent.action = null
        intent.removeExtra(EXTRA_KEY)
        intent.removeExtra(EXTRA_NOTE_ID)
        intent.removeExtra(EXTRA_LINE)
        if (answer != null) {
            respond(context.applicationContext, answer)
        }
    }

    /** A button of a notification was pressed: the notification goes, the answer is given. */
    internal fun handleButton(context: Context, intent: Intent, action: String) {
        val answer = answer(intent, action) ?: return
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.cancel(NOTIFICATION_TAG, ReminderRules.notificationId(answer.key))
        respond(context, answer)
    }

    private fun answer(intent: Intent, action: String): Answer? {
        val key = intent.getStringExtra(EXTRA_KEY) ?: return null
        val noteId = intent.getStringExtra(EXTRA_NOTE_ID) ?: return null
        return Answer(key, noteId, intent.getIntExtra(EXTRA_LINE, 0), action)
    }

    /** Live, or kept; one of them. */
    private fun respond(context: Context, answer: Answer) {
        val live = channel
        if (live != null &&
            ReminderRules.deliverLive(answer.action, engineUp = true, appInFront = inFront)
        ) {
            main.post {
                live.invokeMethod("response", answer.toMap(), object : MethodChannel.Result {
                    override fun success(result: Any?) {}

                    // Dart isn't listening yet (or failed): keep it for takeResponses.
                    override fun error(code: String, message: String?, details: Any?) =
                        keep(context, answer)

                    override fun notImplemented() = keep(context, answer)
                })
            }
        } else {
            keep(context, answer)
        }
    }

    private fun keep(context: Context, answer: Answer) {
        synchronized(lock) {
            val preferences = preferences(context)
            val kept = ReminderRules.decodeAnswers(preferences.getString(RESPONSES, null))
            preferences.edit()
                .putString(RESPONSES, ReminderRules.encodeAnswers(kept + answer))
                .commit()
        }
    }

    private fun takeResponses(context: Context): List<Answer> =
        synchronized(lock) {
            val preferences = preferences(context)
            val kept = ReminderRules.decodeAnswers(preferences.getString(RESPONSES, null))
            preferences.edit().remove(RESPONSES).commit()
            kept
        }

    private fun reminders(arguments: Any?): List<Reminder> =
        (arguments as? List<*>)?.mapNotNull { (it as? Map<*, *>)?.let(Reminder::fromMap) }
            ?: emptyList()

    // Permission

    private fun permission(context: Context): String {
        val enabled = context.getSystemService(NotificationManager::class.java)
            .areNotificationsEnabled()
        val granted = Build.VERSION.SDK_INT < 33 ||
            context.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) ==
            PackageManager.PERMISSION_GRANTED
        return ReminderRules.permission(
            Build.VERSION.SDK_INT,
            runtimeGranted = granted,
            enabled = enabled,
            asked = preferences(context).getBoolean(ASKED, false),
        )
    }

    private fun requestPermission(activity: Activity, result: MethodChannel.Result) {
        val context = activity.applicationContext
        if (Build.VERSION.SDK_INT < 33) {
            result.success(permission(context) == "granted")
            return
        }
        if (permission(context) == "granted") {
            result.success(true)
            return
        }
        pendingPermission?.success(false)
        pendingPermission = result
        preferences(context).edit().putBoolean(ASKED, true).apply()
        activity.requestPermissions(
            arrayOf(Manifest.permission.POST_NOTIFICATIONS),
            PERMISSION_REQUEST,
        )
    }

    // Alarms

    /** Replaces every pending reminder with [reminders]. */
    private fun schedule(context: Context, reminders: List<Reminder>) {
        synchronized(lock) {
            cancelAlarms(context)
            val upcoming = ReminderRules.upcoming(reminders, System.currentTimeMillis())
            preferences(context).edit()
                .putString(SCHEDULED, ReminderRules.encodeReminders(upcoming))
                .commit()
            setAlarms(context, upcoming)
        }
    }

    /** Sets the alarms of the kept reminders again: after a reboot or an update. */
    internal fun rearm(context: Context) {
        synchronized(lock) {
            val kept = ReminderRules.decodeReminders(
                preferences(context).getString(SCHEDULED, null)
            )
            val upcoming = ReminderRules.upcoming(kept, System.currentTimeMillis())
            preferences(context).edit()
                .putString(SCHEDULED, ReminderRules.encodeReminders(upcoming))
                .commit()
            setAlarms(context, upcoming)
        }
    }

    private fun setAlarms(context: Context, reminders: List<Reminder>) {
        val alarms = context.getSystemService(AlarmManager::class.java)
        val exact = Build.VERSION.SDK_INT < Build.VERSION_CODES.S || alarms.canScheduleExactAlarms()
        for (reminder in reminders) {
            val intent = alarmIntent(context, reminder.key, PendingIntent.FLAG_UPDATE_CURRENT)
                ?: continue
            val at = reminder.triggerAtMillis()
            try {
                if (exact) {
                    alarms.setExactAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, at, intent)
                } else {
                    alarms.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, at, intent)
                }
            } catch (_: SecurityException) {
                // Exact was withdrawn between the check and the call.
                alarms.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, at, intent)
            }
        }
    }

    private fun cancelAlarms(context: Context) {
        val alarms = context.getSystemService(AlarmManager::class.java)
        val kept = ReminderRules.decodeReminders(preferences(context).getString(SCHEDULED, null))
        for (reminder in kept) {
            alarmIntent(context, reminder.key, PendingIntent.FLAG_NO_CREATE)?.let {
                alarms.cancel(it)
                it.cancel()
            }
        }
    }

    private fun alarmIntent(context: Context, key: String, flag: Int): PendingIntent? {
        val intent = Intent(context, KansoReminderReceiver::class.java)
            .setAction(ACTION_FIRE)
            .putExtra(EXTRA_KEY, key)
        return PendingIntent.getBroadcast(
            context,
            ReminderRules.requestCode("fire", key),
            intent,
            flag or PendingIntent.FLAG_IMMUTABLE,
        )
    }

    /** Removes every pending alarm and shown notification of Kanso's reminders. */
    private fun cancelAll(context: Context) {
        synchronized(lock) {
            cancelAlarms(context)
            preferences(context).edit().remove(SCHEDULED).commit()
        }
        val manager = context.getSystemService(NotificationManager::class.java)
        for (shown in manager.activeNotifications) {
            if (shown.tag == NOTIFICATION_TAG) {
                manager.cancel(shown.tag, shown.id)
            }
        }
    }

    /** The alarm for [key] went off: shows it, and forgets it. */
    internal fun fire(context: Context, key: String) {
        val reminder = synchronized(lock) {
            val preferences = preferences(context)
            val kept = ReminderRules.decodeReminders(preferences.getString(SCHEDULED, null))
            preferences.edit()
                .putString(SCHEDULED, ReminderRules.encodeReminders(kept.filter { it.key != key }))
                .commit()
            kept.firstOrNull { it.key == key }
        } ?: return
        post(context, reminder)
    }

    // Notifications

    private fun ensureChannel(context: Context) {
        val manager = context.getSystemService(NotificationManager::class.java)
        if (manager.getNotificationChannel(NOTIFICATION_CHANNEL) == null) {
            manager.createNotificationChannel(
                NotificationChannel(
                    NOTIFICATION_CHANNEL,
                    "Reminders",
                    NotificationManager.IMPORTANCE_HIGH,
                )
            )
        }
    }

    /** Posts the notification for [reminder]; quiet when notifications are off. */
    internal fun post(context: Context, reminder: Reminder) {
        val manager = context.getSystemService(NotificationManager::class.java)
        if (!manager.areNotificationsEnabled() ||
            (Build.VERSION.SDK_INT >= 33 &&
                context.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) !=
                PackageManager.PERMISSION_GRANTED)
        ) {
            return
        }
        ensureChannel(context)
        val open = Intent(context, MainActivity::class.java)
            .setAction(ACTION_OPEN)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP)
        carry(open, reminder)
        val notification = Notification.Builder(context, NOTIFICATION_CHANNEL)
            .setSmallIcon(R.drawable.ic_stat_reminder)
            .setContentTitle(reminder.title)
            .setContentText(reminder.body)
            .setCategory(Notification.CATEGORY_REMINDER)
            .setAutoCancel(true)
            .setContentIntent(
                PendingIntent.getActivity(
                    context,
                    ReminderRules.requestCode("open", reminder.key),
                    open,
                    PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
                )
            )
            .addAction(button(context, reminder, ACTION_MARK_DONE, "Mark Done"))
            .addAction(button(context, reminder, ACTION_DISMISS, "Dismiss"))
            .build()
        manager.notify(NOTIFICATION_TAG, ReminderRules.notificationId(reminder.key), notification)
    }

    private fun button(
        context: Context,
        reminder: Reminder,
        action: String,
        label: String,
    ): Notification.Action {
        val intent = Intent(context, KansoReminderReceiver::class.java).setAction(action)
        carry(intent, reminder)
        val pending = PendingIntent.getBroadcast(
            context,
            ReminderRules.requestCode(action, reminder.key),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        return Notification.Action.Builder(null, label, pending).build()
    }

    /** The three ids, and nothing else of the note. */
    private fun carry(intent: Intent, reminder: Reminder) {
        intent.putExtra(EXTRA_KEY, reminder.key)
            .putExtra(EXTRA_NOTE_ID, reminder.noteId)
            .putExtra(EXTRA_LINE, reminder.line)
    }

    private fun preferences(context: Context) =
        context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)
}
