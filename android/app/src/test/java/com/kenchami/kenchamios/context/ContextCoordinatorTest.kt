package com.kenchami.kenchamios.context

import com.kenchami.kenchamios.data.LocalSettings
import com.kenchami.kenchamios.data.TestDatabase
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.time.LocalDateTime
import java.time.ZoneOffset

private class FakeGeofences : Geofences {
    var registered: List<PlaceFence>? = null
    var error: String? = null

    override suspend fun replaceAll(places: List<PlaceFence>): String? {
        registered = places
        return error
    }

    override suspend fun clear() {
        registered = emptyList()
    }
}

private class FakeTimePlan : TimePlan {
    var wake: Long? = -1

    override fun wakeAt(epochMs: Long?) {
        wake = epochMs
    }
}

private class FakeWifi(var isConnected: Boolean = true) : WifiWatch {
    var watching = false

    override fun start() {
        watching = true
    }

    override fun stop() {
        watching = false
    }

    override fun connected() = isConnected
}

private class FakePermissions(var fine: Boolean = true, var background: Boolean = true) : ContextPermissions {
    override fun location() = fine

    override fun backgroundLocation() = background

    override fun notifications() = true
}

@RunWith(RobolectricTestRunner::class)
class ContextCoordinatorTest {
    private lateinit var env: TestDatabase
    private lateinit var rules: RuleRepository
    private val geofences = FakeGeofences()
    private val timePlan = FakeTimePlan()
    private val wifi = FakeWifi()
    private val permissions = FakePermissions()
    private lateinit var coordinator: ContextCoordinator

    @Before
    fun setUp() {
        env = TestDatabase()
        rules = RuleRepository(LocalSettings(env.database.settingDao(), env.stamper), env.clock)
        coordinator = ContextCoordinator(rules, geofences, timePlan, wifi, permissions, env.clock)
    }

    @After
    fun tearDown() = env.close()

    @Test
    fun enabledPlaceRulesBecomeGeofences() = runTest {
        rules.save(placeRule("a"))
        rules.save(placeRule("b", enabled = false))

        assertNull(coordinator.refresh())

        assertEquals(listOf("a"), geofences.registered?.map { it.ruleId })
    }

    @Test
    fun withoutTheBackgroundPermissionNothingIsRegistered() = runTest {
        rules.save(placeRule("a"))
        permissions.background = false

        coordinator.refresh()

        assertEquals(emptyList<PlaceFence>(), geofences.registered)
    }

    @Test
    fun withoutLocationTheOtherKindsStillWork() = runTest {
        permissions.fine = false
        rules.save(placeRule("a"))
        rules.save(timeRule("t"))
        rules.save(wifiRule("w"))

        coordinator.refresh()

        assertEquals(emptyList<PlaceFence>(), geofences.registered)
        assertEquals(true, timePlan.wake != null)
        assertEquals(true, wifi.watching)
    }

    @Test
    fun aFailedRegistrationIsReported() = runTest {
        rules.save(placeRule("a"))
        geofences.error = "Location is turned off on this phone."

        assertEquals("Location is turned off on this phone.", coordinator.refresh())
    }

    @Test
    fun theWakeUpIsTheEarliestTimeRule() = runTest {
        val midnight = LocalDateTime.parse("2026-06-10T00:00").toInstant(ZoneOffset.UTC).toEpochMilli()
        env.clock.ms = midnight
        rules.save(timeRule("late", minuteOfDay = 20 * 60))
        rules.save(timeRule("early", minuteOfDay = 7 * 60))
        rules.save(timeRule("off", minuteOfDay = 60, enabled = false))

        coordinator.refresh()

        assertEquals(midnight + 7 * 60 * 60 * 1000, timePlan.wake)
    }

    @Test
    fun withoutTimeRulesTheWakeUpIsCancelled() = runTest {
        coordinator.refresh()

        assertNull(timePlan.wake)
    }

    @Test
    fun theWifiWatchRunsOnlyWhileAWifiRuleIsEnabled() = runTest {
        rules.save(wifiRule("w"))
        coordinator.refresh()
        assertEquals(true, wifi.watching)
        assertEquals(true, rules.wifiConnected())

        rules.save(wifiRule("w", enabled = false))
        coordinator.refresh()
        assertEquals(false, wifi.watching)
    }

    @Test
    fun theMasterSwitchOffUnregistersEverything() = runTest {
        rules.save(placeRule("a"))
        rules.save(timeRule("t"))
        rules.save(wifiRule("w"))
        coordinator.refresh()

        rules.setEnabled(false)
        coordinator.refresh()

        assertEquals(emptyList<PlaceFence>(), geofences.registered)
        assertNull(timePlan.wake)
        assertEquals(false, wifi.watching)
    }

    @Test
    fun noMoreThanThePlatformLimitIsRegistered() = runTest {
        repeat(MAX_PLACE_RULES + 5) { rules.save(placeRule("p%03d".format(it))) }

        coordinator.refresh()

        assertEquals(MAX_PLACE_RULES, geofences.registered?.size)
    }
}
