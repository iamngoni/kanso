package app.kanso.kanso_background

import android.app.NotificationChannel
import android.app.Notification
import android.app.NotificationManager
import android.content.Context
import android.content.pm.ServiceInfo
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.ForegroundInfo
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.OutOfQuotaPolicy
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.util.concurrent.TimeUnit

/**
 * One bounded sync with no app (WorkManager). The periodic job and the
 * expedited one-time job (a silent push's, #48 step 8b) are this same class.
 *
 * The result of `kanso_background_sync` decides how the job ends: 0 or more
 * is a success; -3 (the server) and -4 (out of time) are tried again; the
 * rest (signed out, the Keystore refused, internal) fail.
 */
class SyncWorker(
    context: Context,
    params: WorkerParameters,
) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result =
        withContext(Dispatchers.IO) {
            val code = BackgroundSync.run(applicationContext)
            when {
                code >= 0 -> Result.success()
                code == -3 || code == -4 -> Result.retry()
                else -> Result.failure()
            }
        }

    /** What Android 10 and 11 show while an expedited job runs. */
    override suspend fun getForegroundInfo(): ForegroundInfo {
        val manager =
            applicationContext.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.createNotificationChannel(
            NotificationChannel(CHANNEL, "Sync", NotificationManager.IMPORTANCE_MIN),
        )
        val notification =
            Notification.Builder(applicationContext, CHANNEL)
                .setSmallIcon(android.R.drawable.stat_notify_sync)
                .setContentTitle("Syncing")
                .setOngoing(true)
                .build()
        return ForegroundInfo(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
    }

    companion object {
        /** The unique name of the periodic work. */
        const val PERIODIC = "app.kanso.sync.periodic"

        /** The unique name of the expedited work. */
        const val NOW = "app.kanso.sync.now"

        private const val CHANNEL = "app.kanso.sync"
        private const val NOTIFICATION_ID = 4801

        private val needsNetwork =
            Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build()

        /** Every 15 minutes with a network; one at a time, and a pending one is kept. */
        @JvmStatic
        fun schedulePeriodic(context: Context) {
            val request =
                PeriodicWorkRequestBuilder<SyncWorker>(15, TimeUnit.MINUTES)
                    .setConstraints(needsNetwork)
                    .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 1, TimeUnit.MINUTES)
                    .build()
            WorkManager.getInstance(context)
                .enqueueUniquePeriodicWork(PERIODIC, ExistingPeriodicWorkPolicy.KEEP, request)
        }

        /**
         * One sync as soon as the system lets it run (expedited; when the
         * quota is spent it runs as ordinary work). A job already waiting is
         * kept.
         */
        @JvmStatic
        fun enqueueNow(context: Context) {
            val request =
                OneTimeWorkRequestBuilder<SyncWorker>()
                    .setConstraints(needsNetwork)
                    .setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST)
                    .build()
            WorkManager.getInstance(context)
                .enqueueUniqueWork(NOW, ExistingWorkPolicy.KEEP, request)
        }
    }
}
