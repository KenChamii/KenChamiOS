package com.kenchami.kenchamios.context

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.work.Configuration
import androidx.work.WorkInfo
import androidx.work.WorkManager
import androidx.work.testing.WorkManagerTestInitHelper
import com.google.android.gms.location.Geofence
import com.kenchami.kenchamios.TestClock
import com.kenchami.kenchamios.context.platform.TIME_WORK
import com.kenchami.kenchamios.context.platform.WorkTimePlan
import com.kenchami.kenchamios.context.platform.placeTransitionOf
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class ContextPlatformTest {
    private lateinit var workManager: WorkManager
    private val clock = TestClock()

    @Before
    fun setUp() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        WorkManagerTestInitHelper.initializeTestWorkManager(context, Configuration.Builder().build())
        workManager = WorkManager.getInstance(context)
    }

    private fun waiting() = workManager.getWorkInfosForUniqueWork(TIME_WORK).get().filter { it.state == WorkInfo.State.ENQUEUED }

    @Test
    fun planningAgainReplacesTheWaitingWakeUp() {
        val plan = WorkTimePlan(workManager, clock)

        plan.wakeAt(clock.ms + 60_000)
        plan.wakeAt(clock.ms + 120_000)

        assertEquals(1, waiting().size)
    }

    @Test
    fun cancellingRemovesTheWakeUp() {
        val plan = WorkTimePlan(workManager, clock)
        plan.wakeAt(clock.ms + 60_000)

        plan.wakeAt(null)

        assertEquals(0, waiting().size)
    }

    @Test
    fun aWakeUpInThePastRunsRightAway() {
        WorkTimePlan(workManager, clock).wakeAt(clock.ms - 1)

        assertEquals(1, workManager.getWorkInfosForUniqueWork(TIME_WORK).get().size)
    }

    @Test
    fun geofenceTransitionsMapToPlaceTransitions() {
        assertEquals(PlaceTransition.Arrive, placeTransitionOf(Geofence.GEOFENCE_TRANSITION_DWELL))
        assertEquals(PlaceTransition.Leave, placeTransitionOf(Geofence.GEOFENCE_TRANSITION_EXIT))
        assertNull(placeTransitionOf(Geofence.GEOFENCE_TRANSITION_ENTER))
    }
}
