package com.kenchami.kenchamios.context.platform

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.kenchami.kenchamios.context.ContextEvent
import com.kenchami.kenchamios.context.TimePlan
import dagger.hilt.android.EntryPointAccessors
import java.time.Clock
import java.util.concurrent.TimeUnit

const val TIME_WORK = "context-time"

/** One inexact wake-up for the earliest time rule; the worker plans the next one when it has run. */
class WorkTimePlan(private val workManager: WorkManager, private val clock: Clock) : TimePlan {
    override fun wakeAt(epochMs: Long?) {
        if (epochMs == null) {
            workManager.cancelUniqueWork(TIME_WORK)
            return
        }
        val delay = (epochMs - clock.millis()).coerceAtLeast(0)
        val request = OneTimeWorkRequestBuilder<TimeRulesWorker>().setInitialDelay(delay, TimeUnit.MILLISECONDS).build()
        workManager.enqueueUniqueWork(TIME_WORK, ExistingWorkPolicy.REPLACE, request)
    }
}

class TimeRulesWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val graph = EntryPointAccessors.fromApplication(applicationContext, ContextEntryPoint::class.java)
        graph.contextRunner().handle(ContextEvent.TimeCheck)
        graph.coordinator().refresh()
        return Result.success()
    }
}
