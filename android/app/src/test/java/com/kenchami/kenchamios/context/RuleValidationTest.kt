package com.kenchami.kenchamios.context

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class RuleValidationTest {
    private fun check(rule: Rule, others: Int = 0) = RuleValidation.check(rule, others)

    private fun place(latitude: Double = 0.0, longitude: Double = 0.0, radius: Int = 150) =
        placeRule().copy(trigger = Trigger.Place(latitude, longitude, radius, PlaceTransition.Leave))

    @Test
    fun aValidRuleOfEachKindPasses() {
        assertNull(check(placeRule()))
        assertNull(check(timeRule()))
        assertNull(check(wifiRule()))
        assertNull(check(placeRule().copy(action = RuleAction.Checklist(listOf("Keys")))))
    }

    @Test
    fun theNameIsRequiredAndShort() {
        assertEquals("Give the rule a name.", check(placeRule().copy(name = "  ")))
        assertEquals("The name is longer than 60 characters.", check(placeRule().copy(name = "x".repeat(61))))
    }

    @Test
    fun theRadiusAndTheCoordinatesAreBounded() {
        assertEquals("Latitude must be between -90 and 90.", check(place(latitude = 90.5)))
        assertEquals("Longitude must be between -180 and 180.", check(place(longitude = -181.0)))
        assertEquals("The radius must be between 100 and 5000 metres.", check(place(radius = 99)))
        assertEquals("The radius must be between 100 and 5000 metres.", check(place(radius = 5001)))
        assertNull(check(place(radius = 100)))
        assertNull(check(place(radius = 5000)))
    }

    @Test
    fun theHundredAndFirstEnabledPlaceIsRefused() {
        assertNull(check(placeRule(), others = 99))
        assertEquals("Android allows 100 place rules at most. Turn one off.", check(placeRule(), others = 100))
        assertNull(check(placeRule(enabled = false), others = 100))
    }

    @Test
    fun aTimeRuleNeedsAWeekdayAndAValidMinute() {
        assertEquals("Pick at least one weekday.", check(timeRule(days = emptySet())))
        assertEquals("Pick a time of day.", check(timeRule(minuteOfDay = 1440)))
        assertNull(check(timeRule(minuteOfDay = 1439)))
    }

    @Test
    fun theActionTextsAreChecked() {
        fun withAction(action: RuleAction) = check(wifiRule().copy(action = action))
        assertEquals("Write the reminder text.", withAction(RuleAction.Remind(" ")))
        assertEquals("The text is longer than 200 characters.", withAction(RuleAction.Remind("x".repeat(201))))
        assertEquals("Add at least one checklist item.", withAction(RuleAction.Checklist(emptyList())))
        assertEquals("Checklist items cannot be empty.", withAction(RuleAction.Checklist(listOf("a", " "))))
        assertEquals("A checklist can have 20 items at most.", withAction(RuleAction.Checklist(List(21) { "i$it" })))
        assertEquals("An item is longer than 60 characters.", withAction(RuleAction.Checklist(listOf("x".repeat(61)))))
    }
}
