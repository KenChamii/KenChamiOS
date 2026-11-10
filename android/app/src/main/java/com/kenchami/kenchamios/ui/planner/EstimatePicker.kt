package com.kenchami.kenchamios.ui.planner

import androidx.compose.foundation.layout.Box
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import com.kenchami.kenchamios.core.ESTIMATES
import com.kenchami.kenchamios.core.formatMinutes

/** A button with the task's estimate that opens the list of estimates. */
@Composable
fun EstimatePicker(title: String, value: Int?, onChange: (Int?) -> Unit) {
    var open by remember { mutableStateOf(false) }
    // keep an odd length from a desktop time block selectable
    val choices = (ESTIMATES + listOfNotNull(value)).distinct().sorted()
    Box {
        TextButton(onClick = { open = true }, modifier = Modifier.semantics { contentDescription = "Estimate for $title" }) {
            Text(value?.let(::formatMinutes) ?: "No estimate")
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            (listOf<Int?>(null) + choices).forEach { minutes ->
                DropdownMenuItem(
                    text = { Text(minutes?.let(::formatMinutes) ?: "No estimate") },
                    onClick = {
                        open = false
                        onChange(minutes)
                    },
                )
            }
        }
    }
}
