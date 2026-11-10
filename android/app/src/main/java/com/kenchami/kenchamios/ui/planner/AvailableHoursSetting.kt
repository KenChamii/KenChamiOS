package com.kenchami.kenchamios.ui.planner

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.core.parseHours
import com.kenchami.kenchamios.ui.common.AppTextField

private const val MINUTES_PER_HOUR = 60.0

/** The hours of focused work a day has, which Plan my day compares the estimates with. */
@Composable
fun AvailableHoursSetting(saved: Int, onSave: (Int) -> Unit) {
    var draft by rememberSaveable(saved) { mutableStateOf(hoursText(saved)) }
    val parsed = parseHours(draft)
    Column(Modifier.padding(horizontal = 16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(
            "Plan my day compares the estimates of today's tasks with the hours you have for focused work, " +
                "and warns when the plan does not fit.",
            style = MaterialTheme.typography.bodyMedium,
        )
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            AppTextField(
                draft,
                { draft = it },
                "Hours you have each day",
                Modifier.weight(1f),
                singleLine = true,
                isError = parsed == null,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
            )
            Button(onClick = { parsed?.let(onSave) }, enabled = parsed != null && parsed != saved) { Text("Save") }
        }
        if (parsed == null) {
            Text("Use half an hour to 24 hours, like 6 or 7.5.", color = MaterialTheme.colorScheme.error)
        }
    }
}

// 360 shows as "6", 450 as "7.5"
private fun hoursText(minutes: Int): String {
    val hours = minutes / MINUTES_PER_HOUR
    return if (hours % 1.0 == 0.0) hours.toInt().toString() else hours.toString()
}
