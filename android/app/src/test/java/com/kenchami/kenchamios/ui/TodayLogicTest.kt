package com.kenchami.kenchamios.ui

import com.kenchami.kenchamios.ui.today.SetupStep
import com.kenchami.kenchamios.ui.today.dateLabel
import com.kenchami.kenchamios.ui.today.greeting
import com.kenchami.kenchamios.ui.today.setupSteps
import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.LocalDate
import java.time.LocalTime
import java.util.Locale

class TodayLogicTest {
    @Test
    fun greetingFollowsTheTimeOfDay() {
        assertEquals("Good evening", greeting(LocalTime.of(4, 59)))
        assertEquals("Good morning", greeting(LocalTime.of(5, 0)))
        assertEquals("Good morning", greeting(LocalTime.of(11, 59)))
        assertEquals("Good afternoon", greeting(LocalTime.of(12, 0)))
        assertEquals("Good afternoon", greeting(LocalTime.of(17, 59)))
        assertEquals("Good evening", greeting(LocalTime.of(18, 0)))
        assertEquals("Good evening", greeting(LocalTime.of(23, 30)))
    }

    @Test
    fun dateLabelNamesTheWeekdayAndMonth() {
        assertEquals("Tuesday, 10 November", dateLabel(LocalDate.of(2026, 11, 10), Locale.UK))
    }

    @Test
    fun aNewInstallGetsEveryStep() {
        assertEquals(SetupStep.entries, setupSteps(hasProjects = false, hasMedia = false, paired = false))
    }

    @Test
    fun pairingDropsOutOnceItIsDone() {
        assertEquals(
            listOf(SetupStep.CreateProject, SetupStep.AddTask, SetupStep.AddMedia),
            setupSteps(hasProjects = false, hasMedia = false, paired = true),
        )
    }

    @Test
    fun anyProjectOrMediaEndsTheChecklist() {
        assertEquals(emptyList<SetupStep>(), setupSteps(hasProjects = true, hasMedia = false, paired = false))
        assertEquals(emptyList<SetupStep>(), setupSteps(hasProjects = false, hasMedia = true, paired = false))
    }
}
