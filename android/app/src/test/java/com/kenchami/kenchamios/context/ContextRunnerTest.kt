package com.kenchami.kenchamios.context

import com.kenchami.kenchamios.data.LocalSettings
import com.kenchami.kenchamios.data.TestDatabase
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

private data class Posted(val ruleId: String, val title: String, val text: String)

@RunWith(RobolectricTestRunner::class)
class ContextRunnerTest {
    private lateinit var env: TestDatabase
    private lateinit var rules: RuleRepository
    private lateinit var runs: ChecklistRunRepository
    private lateinit var runner: ContextRunner
    private val posted = mutableListOf<Posted>()
    private var due = listOf<String>()

    @Before
    fun setUp() {
        env = TestDatabase()
        val settings = LocalSettings(env.database.settingDao(), env.stamper)
        rules = RuleRepository(settings, env.clock)
        runs = ChecklistRunRepository(settings)
        runner = ContextRunner(rules, runs, { due }, { id, title, text -> posted += Posted(id, title, text) }, env.clock)
    }

    @After
    fun tearDown() = env.close()

    private val leaveHome = placeRule("home", action = RuleAction.Checklist(listOf("Keys", "Wallet", "Phone")))

    @Test
    fun aChecklistRuleStartsARunAndNotifies() = runTest {
        rules.save(leaveHome)

        runner.handle(ContextEvent.PlaceChanged("home", PlaceTransition.Leave))

        assertEquals(listOf("Keys", "Wallet", "Phone"), runs.observe().first().single().items.map { it.text })
        assertEquals(listOf(Posted("home", "Place home", "Check 3 things before you go.")), posted)
    }

    @Test
    fun aNewTriggerStartsFreshAfterTheCooldown() = runTest {
        rules.save(leaveHome)
        runner.handle(ContextEvent.PlaceChanged("home", PlaceTransition.Leave))
        runs.toggle("home", 0)

        env.clock.ms += COOLDOWN_MS
        runner.handle(ContextEvent.PlaceChanged("home", PlaceTransition.Leave))

        assertEquals(listOf(false, false, false), runs.observe().first().single().items.map { it.checked })
        assertEquals(2, posted.size)
    }

    @Test
    fun aFlappingSignalFiresOnce() = runTest {
        rules.save(leaveHome)

        repeat(5) { runner.handle(ContextEvent.PlaceChanged("home", PlaceTransition.Leave)) }

        assertEquals(1, posted.size)
    }

    @Test
    fun theMasterSwitchStopsEverything() = runTest {
        rules.save(leaveHome)
        rules.setEnabled(false)

        runner.handle(ContextEvent.PlaceChanged("home", PlaceTransition.Leave))

        assertTrue(posted.isEmpty())
        assertTrue(runs.observe().first().isEmpty())
    }

    @Test
    fun aRemindRuleShowsItsText() = runTest {
        rules.save(placeRule("p", action = RuleAction.Remind("Take the bins out")))

        runner.handle(ContextEvent.PlaceChanged("p", PlaceTransition.Leave))

        assertEquals(listOf(Posted("p", "Place p", "Take the bins out")), posted)
    }

    @Test
    fun showTasksListsTheFirstFiveAndCountsTheRest() = runTest {
        rules.save(wifiRule("w"))
        due = List(7) { "Task ${it + 1}" }

        runner.handle(ContextEvent.WifiChanged(WifiTransition.Connect))

        assertEquals("Task 1, Task 2, Task 3, Task 4, Task 5 and 2 more", posted.single().text)
    }

    @Test
    fun showTasksStaysQuietWithoutTasks() = runTest {
        rules.save(wifiRule("w"))

        runner.handle(ContextEvent.WifiChanged(WifiTransition.Connect))

        assertTrue(posted.isEmpty())
    }

    @Test
    fun aDueTimeRuleFiresOnceAndIsRecorded() = runTest {
        rules.save(timeRule("t", minuteOfDay = 0))
        // the clock passes the next midnight, which is when the rule is due
        env.clock.ms += 6_401_000

        runner.handle(ContextEvent.TimeCheck)
        runner.handle(ContextEvent.TimeCheck)

        assertEquals(1, posted.size)
    }

    @Test
    fun theFirstWifiReadingIsOnlyRemembered() = runTest {
        rules.save(wifiRule("w", WifiTransition.Connect))
        due = listOf("A")

        runner.onWifiState(true)
        assertTrue(posted.isEmpty())

        runner.onWifiState(false)
        runner.onWifiState(false)
        env.clock.ms += COOLDOWN_MS
        runner.onWifiState(true)
        assertEquals(1, posted.size)
    }

    @Test
    fun aWifiChangeFiresTheMatchingTransitionOnly() = runTest {
        rules.save(wifiRule("off", WifiTransition.Disconnect))
        due = listOf("A")
        runner.onWifiState(true)

        runner.onWifiState(false)

        assertEquals(listOf("off"), posted.map { it.ruleId })
    }
}
