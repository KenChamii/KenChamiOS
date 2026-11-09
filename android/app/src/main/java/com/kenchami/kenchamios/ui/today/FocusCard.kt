package com.kenchami.kenchamios.ui.today

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.data.model.TimerStatus
import com.kenchami.kenchamios.timer.TimerState
import com.kenchami.kenchamios.ui.common.TintedIcon
import com.kenchami.kenchamios.ui.common.TonalCard
import com.kenchami.kenchamios.ui.common.formatClock
import com.kenchami.kenchamios.ui.common.icon
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint
import kotlinx.coroutines.delay

private const val TICK_MS = 1_000L

/** The running timer with its task, as the Today screen shows it. */
data class FocusCard(val timer: TimerState, val taskTitle: String)

@Composable
fun FocusCardView(card: FocusCard, now: () -> Long, onPause: () -> Unit, onResume: () -> Unit, onStop: () -> Unit) {
    val current by produceState(now(), card) {
        while (true) {
            value = now()
            delay(TICK_MS)
        }
    }
    val timer = card.timer
    TonalCard(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp).testTag("focus-card")) {
        Column(Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            val tint = Area.Focus.tint
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                TintedIcon(Area.Focus.icon, tint, size = 28.dp)
                Text(
                    style = MaterialTheme.typography.labelLarge,
                    color = tint.content,
                    text = when (timer.phase) {
                        TimerPhase.Focus -> "Focus"
                        TimerPhase.Break -> "Break"
                        TimerPhase.LongBreak -> "Long break"
                    },
                )
            }
            Text(
                formatClock(timer.remainingS(current)),
                style = MaterialTheme.typography.displayLarge,
                modifier = Modifier.testTag("focus-clock"),
            )
            Text(card.taskTitle, style = MaterialTheme.typography.titleMedium)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                if (timer.status == TimerStatus.Running) {
                    Button(onClick = onPause) { Text("Pause") }
                } else {
                    Button(onClick = onResume) { Text("Resume") }
                }
                FilledTonalButton(onClick = onStop) { Text("Stop") }
            }
        }
    }
}

class TimerActions(val onPause: () -> Unit, val onResume: () -> Unit, val onStop: () -> Unit)
