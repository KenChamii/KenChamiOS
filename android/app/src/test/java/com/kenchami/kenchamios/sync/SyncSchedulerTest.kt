package com.kenchami.kenchamios.sync

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.work.Configuration
import androidx.work.NetworkType
import androidx.work.WorkInfo
import androidx.work.WorkManager
import androidx.work.testing.WorkManagerTestInitHelper
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class SyncSchedulerTest {
    private lateinit var workManager: WorkManager
    private lateinit var scheduler: SyncScheduler

    @Before
    fun setUp() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        WorkManagerTestInitHelper.initializeTestWorkManager(context, Configuration.Builder().build())
        workManager = WorkManager.getInstance(context)
        scheduler = SyncScheduler(workManager)
    }

    private fun infos(name: String) = workManager.getWorkInfosForUniqueWork(name).get()

    @Test
    fun theSchedulerRunsEveryFifteenMinutesOnAnUnmeteredNetwork() {
        scheduler.schedulePeriodic()
        scheduler.schedulePeriodic()

        val info = infos(PERIODIC_WORK).single()
        assertEquals(WorkInfo.State.ENQUEUED, info.state)
        assertEquals(NetworkType.UNMETERED, info.constraints.requiredNetworkType)
        assertEquals(PERIOD_MINUTES * 60 * 1000, info.periodicityInfo?.repeatIntervalMillis)
    }

    @Test
    fun manyChangesInARowLeaveOneWaitingRun() {
        repeat(5) { scheduler.syncSoon() }

        val waiting = infos(SOON_WORK).filter { it.state == WorkInfo.State.ENQUEUED }
        assertEquals(1, waiting.size)
        assertEquals(NetworkType.UNMETERED, waiting.single().constraints.requiredNetworkType)
    }

    @Test
    fun cancellingRemovesBothSchedules() {
        scheduler.schedulePeriodic()
        scheduler.syncSoon()

        scheduler.cancelAll()

        assertEquals(emptyList<WorkInfo>(), infos(PERIODIC_WORK).filter { it.state == WorkInfo.State.ENQUEUED })
        assertEquals(emptyList<WorkInfo>(), infos(SOON_WORK).filter { it.state == WorkInfo.State.ENQUEUED })
    }
}
