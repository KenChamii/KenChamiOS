package com.kenchami.kenchamios.context

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.DayOfWeek
import java.time.LocalDateTime
import java.time.ZoneOffset

class RuleEngineTest {
    private val zone = ZoneOffset.UTC
    private val now = LocalDateTime.parse("2026-06-10T08:05").toInstant(ZoneOffset.UTC).toEpochMilli()
    private val minute = 60 * 1000L

    private fun fired(event: ContextEvent, rules: List<Rule>, lastFired: Map<String, Long> = emptyMap(), at: Long = now) =
        RuleEngine.evaluate(event, rules, lastFired, at, zone).map { it.id }

    @Test
    fun aPlaceEventFiresOnlyTheRuleItNamesWithTheSameTransition() {
        val rules = listOf(placeRule("a"), placeRule("b"), placeRule("c", PlaceTransition.Arrive))
        assertEquals(listOf("a"), fired(ContextEvent.PlaceChanged("a", PlaceTransition.Leave), rules))
        assertEquals(emptyList<String>(), fired(ContextEvent.PlaceChanged("c", PlaceTransition.Leave), rules))
        assertEquals(listOf("c"), fired(ContextEvent.PlaceChanged("c", PlaceTransition.Arrive), rules))
    }

    @Test
    fun aWifiEventFiresTheRulesOfThatTransition() {
        val rules = listOf(wifiRule("a"), wifiRule("b", WifiTransition.Disconnect), wifiRule("c"))
        assertEquals(listOf("a", "c"), fired(ContextEvent.WifiChanged(WifiTransition.Connect), rules))
        assertEquals(listOf("b"), fired(ContextEvent.WifiChanged(WifiTransition.Disconnect), rules))
    }

    @Test
    fun disabledRulesNeverFire() {
        val rules = listOf(placeRule("a", enabled = false), wifiRule("w", enabled = false), timeRule("t", enabled = false))
        assertEquals(emptyList<String>(), fired(ContextEvent.PlaceChanged("a", PlaceTransition.Leave), rules))
        assertEquals(emptyList<String>(), fired(ContextEvent.WifiChanged(WifiTransition.Connect), rules))
        assertEquals(emptyList<String>(), fired(ContextEvent.TimeCheck, rules))
    }

    @Test
    fun anEventOfAnotherKindFiresNothing() {
        val rules = listOf(placeRule("a"), timeRule("t"))
        assertEquals(emptyList<String>(), fired(ContextEvent.WifiChanged(WifiTransition.Connect), rules))
        assertEquals(listOf("t"), fired(ContextEvent.TimeCheck, rules))
    }

    @Test
    fun aRuleThatFiredRecentlyWaitsForTheCooldown() {
        val rules = listOf(placeRule("a"), wifiRule("w"))
        val event = ContextEvent.PlaceChanged("a", PlaceTransition.Leave)
        assertEquals(emptyList<String>(), fired(event, rules, mapOf("a" to now - COOLDOWN_MS + 1)))
        assertEquals(listOf("a"), fired(event, rules, mapOf("a" to now - COOLDOWN_MS)))
        assertEquals(emptyList<String>(), fired(ContextEvent.WifiChanged(WifiTransition.Connect), rules, mapOf("w" to now - minute)))
    }

    @Test
    fun theCooldownOfOneRuleDoesNotBlockAnother() {
        val rules = listOf(wifiRule("a"), wifiRule("b"))
        assertEquals(listOf("b"), fired(ContextEvent.WifiChanged(WifiTransition.Connect), rules, mapOf("a" to now)))
    }

    @Test
    fun aTimeRuleFiresOnceForEachScheduledSlot() {
        val rules = listOf(timeRule("t"))
        val slot = now - 5 * minute
        assertEquals(listOf("t"), fired(ContextEvent.TimeCheck, rules, mapOf("t" to slot - minute)))
        assertEquals(emptyList<String>(), fired(ContextEvent.TimeCheck, rules, mapOf("t" to slot)))
        assertEquals(emptyList<String>(), fired(ContextEvent.TimeCheck, rules, mapOf("t" to now)))
    }

    @Test
    fun aTimeRuleThatNeverFiredIsDueWithinTheGrace() {
        assertEquals(listOf("t"), fired(ContextEvent.TimeCheck, listOf(timeRule("t")), emptyMap()))
    }

    @Test
    fun aTimeRuleIsDroppedWhenTheRunIsTooLate() {
        val late = now + TIME_GRACE_MS
        assertEquals(emptyList<String>(), fired(ContextEvent.TimeCheck, listOf(timeRule("t")), emptyMap(), late))
    }

    @Test
    fun aTimeRuleOnOtherWeekdaysDoesNotFireToday() {
        // 2026-06-10 is a Wednesday, so a Sunday-only rule has no slot within the grace
        val sunday = timeRule("t", days = setOf(DayOfWeek.SUNDAY))
        assertEquals(emptyList<String>(), fired(ContextEvent.TimeCheck, listOf(sunday)))
    }
}
