package com.kenchami.kenchamios.sync

import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import java.util.concurrent.TimeUnit

const val PERIODIC_WORK = "sync-periodic"
const val SOON_WORK = "sync-soon"
const val PERIOD_MINUTES = 15L
const val DEBOUNCE_SECONDS = 5L

// Wi-Fi is approximated by an unmetered network; the SSID would need the location permission
private val homeNetwork = Constraints.Builder().setRequiredNetworkType(NetworkType.UNMETERED).build()

/** Plans the background syncs: every 15 minutes, and one shortly after a local change. */
class SyncScheduler(private val workManager: WorkManager) {
    fun schedulePeriodic() {
        val request = PeriodicWorkRequestBuilder<SyncWorker>(PERIOD_MINUTES, TimeUnit.MINUTES).setConstraints(homeNetwork).build()
        workManager.enqueueUniquePeriodicWork(PERIODIC_WORK, ExistingPeriodicWorkPolicy.KEEP, request)
    }

    /** Replacing the waiting request on every change makes the run wait for a quiet moment. */
    fun syncSoon() {
        val request = OneTimeWorkRequestBuilder<SyncWorker>()
            .setConstraints(homeNetwork)
            .setInitialDelay(DEBOUNCE_SECONDS, TimeUnit.SECONDS)
            .build()
        workManager.enqueueUniqueWork(SOON_WORK, ExistingWorkPolicy.REPLACE, request)
    }

    fun cancelAll() {
        workManager.cancelUniqueWork(PERIODIC_WORK)
        workManager.cancelUniqueWork(SOON_WORK)
    }
}
