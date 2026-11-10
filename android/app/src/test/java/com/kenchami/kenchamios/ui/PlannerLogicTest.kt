package com.kenchami.kenchamios.ui

import com.kenchami.kenchamios.core.DayBounds
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.plannedTask
import com.kenchami.kenchamios.ui.notes.monthWeeks
import com.kenchami.kenchamios.ui.planner.candidateNotes
import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.DayOfWeek
import java.time.LocalDate
import java.time.YearMonth

class PlannerLogicTest {
    private val today = LocalDate.of(2026, 11, 10)
    private val bounds = DayBounds(start = 10_000, end = 20_000)

    @Test
    fun suggestionsSayWhyTheyAreOffered() {
        assertEquals(listOf("Overdue"), candidateNotes(plannedTask("A", dueAt = 9_999, plannedOn = null), today, bounds))
        assertEquals(listOf("Due today"), candidateNotes(plannedTask("A", dueAt = 10_000, plannedOn = null), today, bounds))
        assertEquals(emptyList<String>(), candidateNotes(plannedTask("A", dueAt = 20_000, plannedOn = null), today, bounds))
        assertEquals(
            listOf("Left over", "In progress"),
            candidateNotes(plannedTask("A", status = TaskStatus.Review, plannedOn = "2026-11-09"), today, bounds),
        )
    }

    @Test
    fun aMonthIsWholeWeeksFromMonday() {
        val weeks = monthWeeks(YearMonth.of(2026, 11))

        assertEquals(6, weeks.size)
        assertEquals(LocalDate.of(2026, 10, 26), weeks.first().first())
        assertEquals(LocalDate.of(2026, 12, 6), weeks.last().last())
        assertEquals(true, weeks.all { it.size == 7 && it.first().dayOfWeek == DayOfWeek.MONDAY })
        assertEquals(4, monthWeeks(YearMonth.of(2027, 2)).size)
    }
}
