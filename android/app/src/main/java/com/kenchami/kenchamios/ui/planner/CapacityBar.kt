package com.kenchami.kenchamios.ui.planner

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.core.Capacity
import com.kenchami.kenchamios.core.formatMinutes
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint

/** Planned time against the hours available today, with a warning when it does not fit. */
@Composable
fun CapacityBar(capacity: Capacity, modifier: Modifier = Modifier) {
    val over = capacity.overMin > 0
    Column(modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(
                "${formatMinutes(capacity.plannedMin)} of ${formatMinutes(capacity.availableMin)}",
                style = MaterialTheme.typography.titleMedium,
            )
            if (capacity.unestimated > 0) {
                Text(
                    "${capacity.unestimated} without an estimate",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        LinearProgressIndicator(
            progress = { capacity.fill },
            color = if (over) MaterialTheme.colorScheme.error else Area.Focus.tint.content,
            trackColor = MaterialTheme.colorScheme.surfaceContainerHigh,
            modifier = Modifier.fillMaxWidth().semantics { contentDescription = "Planned time" },
        )
        if (over) {
            Text(
                "${formatMinutes(capacity.overMin)} more than you have. Move something to another day.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.error,
            )
        }
    }
}
