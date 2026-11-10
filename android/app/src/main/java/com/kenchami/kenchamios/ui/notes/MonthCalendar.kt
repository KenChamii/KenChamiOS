package com.kenchami.kenchamios.ui.notes

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalLocale
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint
import java.time.DayOfWeek
import java.time.LocalDate
import java.time.YearMonth
import java.time.format.DateTimeFormatter
import java.time.format.TextStyle
import java.time.temporal.TemporalAdjusters

private const val DAYS_PER_WEEK = 7

/** Whole weeks, Monday first, that cover [month]. */
fun monthWeeks(month: YearMonth): List<List<LocalDate>> {
    val last = month.atEndOfMonth()
    return generateSequence(month.atDay(1).with(TemporalAdjusters.previousOrSame(DayOfWeek.MONDAY))) { it.plusWeeks(1) }
        .takeWhile { !it.isAfter(last) }
        .map { monday -> List(DAYS_PER_WEEK) { monday.plusDays(it.toLong()) } }
        .toList()
}

@Composable
fun MonthCalendar(
    month: YearMonth,
    selected: LocalDate,
    today: LocalDate,
    noteDays: Set<String>,
    onSelect: (LocalDate) -> Unit,
    onMonth: (Long) -> Unit,
    modifier: Modifier = Modifier,
) {
    val locale = LocalLocale.current.platformLocale
    Column(modifier) {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            IconButton(onClick = { onMonth(-1) }) {
                Icon(Icons.AutoMirrored.Filled.KeyboardArrowLeft, contentDescription = "Previous month")
            }
            Text(
                DateTimeFormatter.ofPattern("MMMM yyyy", locale).format(month),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.weight(1f),
                textAlign = TextAlign.Center,
            )
            IconButton(onClick = { onMonth(1) }) {
                Icon(Icons.AutoMirrored.Filled.KeyboardArrowRight, contentDescription = "Next month")
            }
        }
        Row(Modifier.fillMaxWidth()) {
            DayOfWeek.entries.forEach { weekday ->
                Text(
                    weekday.getDisplayName(TextStyle.SHORT, locale),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.weight(1f),
                )
            }
        }
        monthWeeks(month).forEach { week ->
            Row(Modifier.fillMaxWidth()) {
                week.forEach { day ->
                    DayCell(day, day.month == month.month, day == selected, day == today, day.toString() in noteDays, onSelect)
                }
            }
        }
    }
}

@Composable
private fun RowScope.DayCell(
    day: LocalDate,
    inMonth: Boolean,
    selected: Boolean,
    today: Boolean,
    hasNote: Boolean,
    onSelect: (LocalDate) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val accent = Area.Focus.tint.content
    val text = when {
        selected -> colors.inverseOnSurface
        today -> accent
        inMonth -> colors.onSurface
        else -> colors.onSurfaceVariant
    }
    val dot = when {
        !hasNote -> Color.Transparent
        selected -> text
        else -> accent
    }
    Column(
        Modifier
            .weight(1f)
            .height(44.dp)
            .padding(2.dp)
            .background(if (selected) colors.inverseSurface else Color.Transparent, MaterialTheme.shapes.small)
            .clickable { onSelect(day) }
            .semantics {
                contentDescription = if (hasNote) "$day, has a note" else day.toString()
                this.selected = selected
            },
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Text(day.dayOfMonth.toString(), color = text, fontWeight = if (today) FontWeight.SemiBold else null)
        Box(Modifier.padding(top = 2.dp).size(4.dp).background(dot, CircleShape))
    }
}
