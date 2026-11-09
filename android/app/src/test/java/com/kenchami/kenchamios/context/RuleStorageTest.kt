package com.kenchami.kenchamios.context

import com.kenchami.kenchamios.data.LocalSettings
import com.kenchami.kenchamios.data.TestDatabase
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.time.DayOfWeek

@RunWith(RobolectricTestRunner::class)
class RuleStorageTest {
    private lateinit var env: TestDatabase
    private lateinit var rules: RuleRepository
    private lateinit var runs: ChecklistRunRepository

    @Before
    fun setUp() {
        env = TestDatabase()
        val settings = LocalSettings(env.database.settingDao(), env.stamper)
        rules = RuleRepository(settings, env.clock)
        runs = ChecklistRunRepository(settings)
    }

    @After
    fun tearDown() = env.close()

    @Test
    fun everyKindOfRuleSurvivesTheJsonRoundTrip() = runTest {
        val all = listOf(
            placeRule("a", action = RuleAction.Checklist(listOf("Keys", "Wallet", "Phone"))),
            timeRule("b", 7 * 60 + 30, setOf(DayOfWeek.MONDAY, DayOfWeek.FRIDAY)),
            wifiRule("c", WifiTransition.Disconnect, enabled = false),
        )
        all.forEach { rules.save(it) }

        assertEquals(all, rules.rules())
        assertEquals(all, rules.observeRules().first())
        assertEquals(all[1], rules.get("b"))
    }

    @Test
    fun aDamagedRowIsSkippedInsteadOfBreakingTheList() = runTest {
        rules.save(wifiRule("ok"))
        env.database.openHelper.writableDatabase.execSQL(
            "INSERT INTO settings (id, created_at, updated_at, device_id, deleted, key, value_json) " +
                "VALUES ('x', 1, 1, 'd', 0, 'context.rule.bad', '{\"name\":1}')",
        )

        assertEquals(listOf("ok"), rules.rules().map { it.id })
        assertNull(rules.get("bad"))
    }

    @Test
    fun savingAgainReplacesTheRule() = runTest {
        rules.save(wifiRule("w"))
        rules.save(wifiRule("w", enabled = false))

        assertEquals(listOf(false), rules.rules().map { it.enabled })
    }

    @Test
    fun deletingARuleAlsoRemovesItsStateAndItsChecklist() = runTest {
        rules.save(wifiRule("w"))
        rules.markFired("w", 5)
        runs.start(ChecklistRun.start(placeRule("w"), RuleAction.Checklist(listOf("Keys"))))

        rules.delete("w")

        assertTrue(rules.rules().isEmpty())
        assertTrue(rules.lastFired().isEmpty())
        assertTrue(runs.observe().first().isEmpty())
    }

    @Test
    fun aSavedTimeRuleStartsCountingFromNow() = runTest {
        rules.save(timeRule("t"))
        rules.save(wifiRule("w"))

        assertEquals(mapOf("t" to env.clock.millis()), rules.lastFired())
    }

    @Test
    fun theSwitchIsOnUntilTurnedOff() = runTest {
        assertTrue(rules.enabled())
        rules.setEnabled(false)
        assertFalse(rules.enabled())
        assertFalse(rules.observeEnabled().first())
        rules.setEnabled(true)
        assertTrue(rules.observeEnabled().first())
    }

    @Test
    fun theWifiStateIsUnknownUntilStored() = runTest {
        assertNull(rules.wifiConnected())
        rules.setWifiConnected(true)
        assertEquals(true, rules.wifiConnected())
    }

    @Test
    fun aNewRunReplacesTheOldOneWithEmptyBoxes() = runTest {
        val rule = placeRule("p")
        val action = RuleAction.Checklist(listOf("Keys", "Wallet"))
        runs.start(ChecklistRun.start(rule, action))
        runs.toggle("p", 0)
        assertEquals(listOf(true, false), runs.observe().first().single().items.map { it.checked })

        runs.start(ChecklistRun.start(rule, action))

        assertEquals(listOf(false, false), runs.observe().first().single().items.map { it.checked })
    }

    @Test
    fun tickingTheLastItemFinishesTheRun() = runTest {
        runs.start(ChecklistRun.start(placeRule("p"), RuleAction.Checklist(listOf("Keys"))))

        runs.toggle("p", 0)

        assertTrue(runs.observe().first().isEmpty())
    }

    @Test
    fun dismissingRemovesTheRunAndTogglingAMissingRunDoesNothing() = runTest {
        runs.start(ChecklistRun.start(placeRule("p"), RuleAction.Checklist(listOf("Keys", "Wallet"))))
        runs.dismiss("p")
        runs.toggle("p", 0)

        assertTrue(runs.observe().first().isEmpty())
    }

    @Test
    fun ruleDataIsNeverPartOfTheSyncLog() = runTest {
        rules.save(placeRule("a"))
        runs.start(ChecklistRun.start(placeRule("a"), RuleAction.Checklist(listOf("Keys"))))

        assertEquals(0, env.count("SELECT COUNT(*) FROM change_log WHERE table_name = 'settings'"))
    }
}
