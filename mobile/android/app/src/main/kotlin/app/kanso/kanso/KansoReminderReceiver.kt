package app.kanso.kanso

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/**
 * Where the reminders' broadcasts land: an alarm going off (posts the
 * notification), a button of the notification (answers without opening the
 * app), and the system saying the alarms are gone (a reboot or an update of
 * the app), which sets them again from what [KansoReminders] kept.
 */
class KansoReminderReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            KansoReminders.ACTION_FIRE ->
                intent.getStringExtra(KansoReminders.EXTRA_KEY)?.let {
                    KansoReminders.fire(context, it)
                }
            KansoReminders.ACTION_MARK_DONE ->
                KansoReminders.handleButton(context, intent, ReminderRules.MARK_DONE)
            KansoReminders.ACTION_DISMISS ->
                KansoReminders.handleButton(context, intent, ReminderRules.DISMISS)
            Intent.ACTION_BOOT_COMPLETED,
            Intent.ACTION_MY_PACKAGE_REPLACED -> KansoReminders.rearm(context)
        }
    }
}
