package com.kenchami.kenchamios.widget

import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceModifier
import androidx.glance.GlanceTheme
import androidx.glance.action.actionParametersOf
import androidx.glance.action.actionStartActivity
import androidx.glance.action.clickable
import androidx.glance.appwidget.action.actionRunCallback
import androidx.glance.appwidget.cornerRadius
import androidx.glance.background
import androidx.glance.layout.Alignment
import androidx.glance.layout.Column
import androidx.glance.layout.Row
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.padding
import androidx.glance.text.FontWeight
import androidx.glance.text.Text
import androidx.glance.text.TextStyle
import com.kenchami.kenchamios.MainActivity
import com.kenchami.kenchamios.data.repo.nextLabel

/** Tasks due today and the continue row. [dayStart] tells overdue tasks from those due today. */
@Composable
fun TodayWidgetContent(data: WidgetData, dayStart: Long) {
    Column(
        GlanceModifier
            .fillMaxSize()
            .background(GlanceTheme.colors.background)
            .cornerRadius(16.dp)
            .padding(12.dp)
            .clickable(actionStartActivity<MainActivity>()),
    ) {
        Heading("Tasks due")
        if (data.due.isEmpty()) Line("Nothing due or overdue")
        data.due.forEach { task ->
            Line(if (task.dueAt < dayStart) "${task.title} - overdue" else task.title)
        }

        Heading("Continue")
        if (data.continuing.isEmpty()) Line("Nothing in progress")
        data.continuing.forEach { entry ->
            Row(GlanceModifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                Text(
                    "${entry.title} - ${nextLabel(entry)}",
                    style = TextStyle(color = GlanceTheme.colors.onSurface, fontSize = 13.sp),
                    maxLines = 1,
                    modifier = GlanceModifier.defaultWeight(),
                )
                if (entry.canIncrement) {
                    Text(
                        "+1",
                        style = TextStyle(color = GlanceTheme.colors.primary, fontWeight = FontWeight.Bold, fontSize = 14.sp),
                        modifier = GlanceModifier
                            .padding(horizontal = 8.dp, vertical = 4.dp)
                            .clickable(actionRunCallback<AddOneAction>(actionParametersOf(EntryIdKey to entry.id))),
                    )
                }
            }
        }
    }
}

@Composable
private fun Heading(text: String) {
    Text(
        text,
        style = TextStyle(color = GlanceTheme.colors.primary, fontWeight = FontWeight.Bold, fontSize = 13.sp),
        modifier = GlanceModifier.padding(top = 6.dp, bottom = 2.dp),
    )
}

@Composable
private fun Line(text: String) {
    Text(text, style = TextStyle(color = GlanceTheme.colors.onSurface, fontSize = 13.sp), maxLines = 1)
}
