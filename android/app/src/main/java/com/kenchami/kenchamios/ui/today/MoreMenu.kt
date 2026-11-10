package com.kenchami.kenchamios.ui.today

import androidx.compose.foundation.layout.Box
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue

/** The pages that have no place in the bottom bar. */
@Composable
fun MoreMenu(nav: TodayNav) {
    var open by remember { mutableStateOf(false) }
    Box {
        IconButton(onClick = { open = true }) {
            Icon(Icons.Filled.MoreVert, contentDescription = "More")
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            listOf("Inbox" to nav.openInbox, "Habits" to nav.openHabits, "Trash" to nav.openTrash).forEach { (label, go) ->
                DropdownMenuItem(
                    text = { Text(label) },
                    onClick = {
                        open = false
                        go()
                    },
                )
            }
        }
    }
}
