package com.kenchami.kenchamios.ui

import com.kenchami.kenchamios.context.PlaceTransition
import com.kenchami.kenchamios.context.RuleAction
import com.kenchami.kenchamios.context.Trigger
import com.kenchami.kenchamios.context.WifiTransition
import com.kenchami.kenchamios.context.placeRule
import com.kenchami.kenchamios.context.timeRule
import com.kenchami.kenchamios.context.wifiRule
import com.kenchami.kenchamios.ui.rules.ActionKind
import com.kenchami.kenchamios.ui.rules.FormResult
import com.kenchami.kenchamios.ui.rules.RuleForm
import com.kenchami.kenchamios.ui.rules.TriggerKind
import com.kenchami.kenchamios.ui.rules.actionSummary
import com.kenchami.kenchamios.ui.rules.clockLabel
import com.kenchami.kenchamios.ui.rules.ruleSummary
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.DayOfWeek

class RuleFormTest {
    private fun valid(form: RuleForm) = (form.toRule("id") as FormResult.Valid).rule

    private fun invalid(form: RuleForm) = (form.toRule("id") as FormResult.Invalid).message

    @Test
    fun everyKindOfRuleSurvivesTheFormRoundTrip() {
        val rules = listOf(
            placeRule("a", PlaceTransition.Arrive, action = RuleAction.Checklist(listOf("Keys", "Wallet"))),
            timeRule("b", 7 * 60 + 5, setOf(DayOfWeek.MONDAY, DayOfWeek.SUNDAY)),
            wifiRule("c", WifiTransition.Disconnect),
        )

        rules.forEach { assertEquals(it, valid(RuleForm.of(it)).copy(id = it.id)) }
    }

    @Test
    fun numbersAreParsedAndTrimmed() {
        val form = RuleForm(name = " Home ", latitude = " 1.5 ", longitude = "-2.25", radius = " 200 ", text = " bye ")

        val rule = valid(form)

        assertEquals("Home", rule.name)
        assertEquals(Trigger.Place(1.5, -2.25, 200, PlaceTransition.Leave), rule.trigger)
        assertEquals(RuleAction.Remind("bye"), rule.action)
    }

    @Test
    fun textThatIsNotANumberNamesTheProblem() {
        assertEquals("Latitude, longitude and radius must be numbers.", invalid(RuleForm(latitude = "north", longitude = "1")))
        assertEquals("Latitude, longitude and radius must be numbers.", invalid(RuleForm(latitude = "1", longitude = "1", radius = "")))
        assertEquals(
            "Hour is 0 to 23 and minute is 0 to 59.",
            invalid(RuleForm(trigger = TriggerKind.Time, hour = "24")),
        )
        assertEquals(
            "Hour is 0 to 23 and minute is 0 to 59.",
            invalid(RuleForm(trigger = TriggerKind.Time, minute = "x")),
        )
    }

    @Test
    fun checklistItemsAreOnePerNonEmptyLine() {
        val form = RuleForm(trigger = TriggerKind.Wifi, action = ActionKind.Checklist, items = " Keys \n\nWallet\n  ")

        assertEquals(RuleAction.Checklist(listOf("Keys", "Wallet")), valid(form).action)
    }

    @Test
    fun theLeavingHomePresetIsALeavePlaceRuleWithThreeItems() {
        val form = RuleForm.leavingHome().copy(latitude = "0", longitude = "0")

        val rule = valid(form)

        assertEquals("Leaving home", rule.name)
        assertEquals(PlaceTransition.Leave, (rule.trigger as Trigger.Place).transition)
        assertEquals(RuleAction.Checklist(listOf("Keys", "Wallet", "Phone")), rule.action)
    }

    @Test
    fun summariesDescribeTheRuleWithoutCoordinates() {
        assertEquals("When you leave a place (150 m) - Remind: Keys", ruleSummary(placeRule()))
        assertEquals(
            "Mon, Sun at 07:05 - Remind: Stand up",
            ruleSummary(timeRule(minuteOfDay = 425, days = setOf(DayOfWeek.SUNDAY, DayOfWeek.MONDAY))),
        )
        assertEquals("Every day at 08:00 - Remind: Stand up", ruleSummary(timeRule()))
        assertEquals("When you join a Wi-Fi network - Show tasks due today", ruleSummary(wifiRule()))
        assertEquals("Checklist of 1 item", actionSummary(RuleAction.Checklist(listOf("a"))))
        assertEquals("09:30", clockLabel(570))
        assertTrue(!ruleSummary(placeRule()).contains("0.0"))
    }
}
