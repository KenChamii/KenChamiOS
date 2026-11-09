package com.kenchami.kenchamios.ui.quickadd

import androidx.compose.material3.DatePicker
import androidx.compose.material3.DatePickerDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberDatePickerState
import androidx.compose.runtime.Composable
import java.util.concurrent.TimeUnit

enum class DueChoice(val label: String) {
    None("No date"),
    Today("Today"),
    Tomorrow("Tomorrow"),
    Pick("Pick date"),
}

/** Calls [onPick] with the chosen day as an epoch day. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DatePickerSheet(onPick: (epochDay: Long) -> Unit, onDismiss: () -> Unit) {
    val state = rememberDatePickerState()
    DatePickerDialog(
        onDismissRequest = onDismiss,
        confirmButton = {
            TextButton(
                onClick = { state.selectedDateMillis?.let { onPick(TimeUnit.MILLISECONDS.toDays(it)) } },
                enabled = state.selectedDateMillis != null,
            ) { Text("OK") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    ) {
        DatePicker(state)
    }
}
