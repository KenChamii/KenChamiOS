package com.kenchami.kenchamios.widget

import android.content.Context
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.glance.GlanceId
import androidx.glance.GlanceTheme
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.provideContent
import com.kenchami.kenchamios.core.dayBounds
import com.kenchami.kenchamios.data.repo.continueItems
import dagger.hilt.android.EntryPointAccessors
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map

private const val MAX_TASKS = 5
private const val MAX_CONTINUE = 3

class TodayWidget : GlanceAppWidget() {
    override suspend fun provideGlance(context: Context, id: GlanceId) {
        val app = EntryPointAccessors.fromApplication(context.applicationContext, WidgetEntryPoint::class.java)
        val clock = app.clock()
        val today = dayBounds(clock.millis(), clock.zone)
        val data = combine(
            app.tasks().observeDue(today.end),
            app.media().observeLibrary().map(::continueItems),
        ) { due, continuing -> WidgetData(due.take(MAX_TASKS), continuing.take(MAX_CONTINUE)) }

        provideContent {
            val current by data.collectAsState(WidgetData())
            GlanceTheme { TodayWidgetContent(current, today.start) }
        }
    }
}
