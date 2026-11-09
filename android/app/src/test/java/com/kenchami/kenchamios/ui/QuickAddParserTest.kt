package com.kenchami.kenchamios.ui

import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.projectSummary
import com.kenchami.kenchamios.ui.quickadd.matchProject
import com.kenchami.kenchamios.ui.quickadd.parseQuickAdd
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.io.File
import java.time.LocalDate

@RunWith(RobolectricTestRunner::class)
class QuickAddParserTest {
    private val fixture = JSONObject(File("../../shared/fixtures/quick-add/parse.json").readText())

    private fun JSONObject.text(key: String): String? = if (isNull(key)) null else getString(key)

    @Test
    fun everySharedCaseParsesLikeTheDesktop() {
        val cases = fixture.getJSONArray("cases")
        for (index in 0 until cases.length()) {
            val case = cases.getJSONObject(index)
            val today = LocalDate.parse(case.text("today") ?: fixture.getString("today"))
            val expected = case.getJSONObject("expected")

            val parsed = parseQuickAdd(case.getString("input"), today)

            val input = case.getString("input")
            assertEquals(input, expected.getString("title"), parsed.title)
            assertEquals(input, expected.text("due"), parsed.due?.toString())
            assertEquals(input, expected.text("priority"), parsed.priority?.wire)
            assertEquals(input, expected.text("project"), parsed.project)
            assertEquals(input, expected.text("recurrence"), parsed.recurrence)
        }
    }

    @Test
    fun priorityWordsMapToTheStoredValues() {
        assertEquals(Priority.Medium, parseQuickAdd("x !m", LocalDate.of(2026, 11, 10)).priority)
    }

    @Test
    fun aTagMatchesASlugOrANameIgnoringCase() {
        val projects = listOf(projectSummary("p1", "Team Sync"), projectSummary("p2", "Home"))

        assertEquals("p2", matchProject("HOME", projects)?.project?.id)
        assertEquals("p1", matchProject("team sync", projects)?.project?.id)
        assertEquals(null, matchProject("work", projects))
    }
}
