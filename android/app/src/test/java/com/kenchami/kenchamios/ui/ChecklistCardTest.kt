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
import com.kenchami.kenchamios.todayState
import com.kenchami.kenchamios.ui.today.ChecklistActions
import com.kenchami.kenchamios.ui.today.TimerActions
import com.kenchami.kenchamios.ui.today.TodayContent
import com.kenchami.kenchamios.ui.today.TodayNav
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class ChecklistCardTest {
    @get:Rule
    val compose = createComposeRule()

    private val noNav = TodayNav({}, {}, {}, {})
    private val state = todayState()
    private val run = ChecklistRun.start(placeRule("home"), RuleAction.Checklist(listOf("Keys", "Wallet", "Phone"))).toggled(1)

    private fun scrollTo(text: String) = compose.onNodeWithTag("today").performScrollToNode(hasText(text))

    @Test
    fun todayShowsTheChecklistWithItsTickedItems() {
        compose.setContent {
            TodayContent(state, null, TimerActions({}, {}, {}), {}, {}, noNav, PaddingValues(0.dp), listOf(run))
        }

        scrollTo("Place home")
        compose.onNodeWithText("Place home").assertIsDisplayed()
        compose.onNodeWithText("Keys").assertIsDisplayed()
        scrollTo("Wallet")
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
                state, null, TimerActions({}, {}, {}), {}, {}, noNav, PaddingValues(0.dp), listOf(run),
                ChecklistActions({ id, index -> ticks += id to index }, { dismissed += it }),
            )
        }

        scrollTo("Keys")
        compose.onNodeWithText("Keys").performClick()
        scrollTo("Dismiss")
        compose.onNodeWithText("Dismiss").performClick()

        assertEquals(listOf("home" to 0), ticks)
        assertEquals(listOf("home"), dismissed)
    }
}
