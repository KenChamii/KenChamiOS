package com.kenchami.kenchamios.ui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsOff
import androidx.compose.ui.test.assertIsOn
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isToggleable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.context.ChecklistRun
import com.kenchami.kenchamios.context.RuleAction
import com.kenchami.kenchamios.context.placeRule
import com.kenchami.kenchamios.core.DayBounds
import com.kenchami.kenchamios.ui.today.ChecklistActions
import com.kenchami.kenchamios.ui.today.TimerActions
import com.kenchami.kenchamios.ui.today.TodayContent
import com.kenchami.kenchamios.ui.today.TodayState
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class ChecklistCardTest {
    @get:Rule
    val compose = createComposeRule()

    private val state =
        TodayState(DayBounds(0, 1), now = 0, due = emptyList(), continueItems = emptyList(), projects = emptyList(), activity = emptyList())
    private val run = ChecklistRun.start(placeRule("home"), RuleAction.Checklist(listOf("Keys", "Wallet", "Phone"))).toggled(1)

    @Test
    fun todayShowsTheChecklistWithItsTickedItems() {
        compose.setContent {
            TodayContent(state, null, TimerActions({}, {}, {}), {}, {}, {}, {}, PaddingValues(0.dp), listOf(run))
        }

        compose.onNodeWithText("Place home").assertIsDisplayed()
        compose.onNodeWithText("Keys").assertIsDisplayed()
        compose.onNodeWithText("Wallet").assertIsDisplayed()
        compose.onAllNodes(isToggleable())[1].assertIsOn()
        compose.onAllNodes(isToggleable())[0].assertIsOff()
    }

    @Test
    fun tickingAnItemAndDismissingAreReported() {
        val ticks = mutableListOf<Pair<String, Int>>()
        val dismissed = mutableListOf<String>()
        compose.setContent {
            TodayContent(
                state, null, TimerActions({}, {}, {}), {}, {}, {}, {}, PaddingValues(0.dp), listOf(run),
                ChecklistActions({ id, index -> ticks += id to index }, { dismissed += it }),
            )
        }

        compose.onNodeWithText("Keys").performClick()
        compose.onNodeWithTag("today").performScrollToNode(hasText("Dismiss"))
        compose.onNodeWithText("Dismiss").performClick()

        assertEquals(listOf("home" to 0), ticks)
        assertEquals(listOf("home"), dismissed)
    }
}
