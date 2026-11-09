package com.kenchami.kenchamios.context

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ChecklistRunTest {
    private val run = ChecklistRun.start(placeRule("p"), RuleAction.Checklist(listOf("Keys", "Wallet")))

    @Test
    fun aNewRunHasNothingTicked() {
        assertEquals(listOf(false, false), run.items.map { it.checked })
        assertEquals("Place p", run.ruleName)
        assertFalse(run.complete)
    }

    @Test
    fun togglingFlipsOneItemAndCompletesWhenAllAreTicked() {
        val half = run.toggled(1)
        assertEquals(listOf(false, true), half.items.map { it.checked })
        assertFalse(half.complete)
        assertTrue(half.toggled(0).complete)
        assertEquals(listOf(false, false), half.toggled(1).items.map { it.checked })
    }
}
