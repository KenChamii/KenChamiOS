package com.kenchami.kenchamios.widget

import androidx.glance.GlanceTheme
import androidx.glance.action.actionParametersOf
import androidx.glance.appwidget.testing.unit.assertHasRunCallbackClickAction
import androidx.glance.appwidget.testing.unit.runGlanceAppWidgetUnitTest
import androidx.glance.testing.unit.hasText
import androidx.test.core.app.ApplicationProvider
import com.kenchami.kenchamios.dueTask
import com.kenchami.kenchamios.libraryEntry
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class TodayWidgetContentTest {
    private val dayStart = 1_000L

    @Test
    fun listsDueAndOverdueTasks() = runGlanceAppWidgetUnitTest {
        setContext(ApplicationProvider.getApplicationContext())
        val data = WidgetData(due = listOf(dueTask("Write docs", 2_000), dueTask("Old task", 500)))

        provideComposable { GlanceTheme(colors = WidgetColors) { TodayWidgetContent(data, dayStart) } }

        onNode(hasText("Write docs")).assertExists()
        onNode(hasText("Old task - overdue")).assertExists()
    }

    @Test
    fun saysWhenThereIsNothingToShow() = runGlanceAppWidgetUnitTest {
        setContext(ApplicationProvider.getApplicationContext())

        provideComposable { GlanceTheme(colors = WidgetColors) { TodayWidgetContent(WidgetData(), dayStart) } }

        onNode(hasText("Nothing due or overdue")).assertExists()
        onNode(hasText("Nothing in progress")).assertExists()
    }

    @Test
    fun continueRowHasAPlusOneButtonThatRunsTheAction() = runGlanceAppWidgetUnitTest {
        setContext(ApplicationProvider.getApplicationContext())
        val data = WidgetData(continuing = listOf(libraryEntry("e1", progress = 12), libraryEntry("e2", progress = 3)))

        provideComposable { GlanceTheme(colors = WidgetColors) { TodayWidgetContent(data, dayStart) } }

        onNode(hasText("Frieren - Episode 4")).assertExists()
        onNode(hasText("+1")).assertHasRunCallbackClickAction<AddOneAction>(actionParametersOf(EntryIdKey to "e2"))
    }
}
