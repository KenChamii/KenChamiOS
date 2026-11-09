package com.kenchami.kenchamios.ui.media

import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import com.kenchami.kenchamios.data.db.LibraryEntry

/** The "+1 episode, chapter or hour" button of a list row; nothing once the total is reached. */
@Composable
fun AddOneButton(entry: LibraryEntry, onAddOne: (String) -> Unit) {
    if (entry.canIncrement) TextButton(onClick = { onAddOne(entry.id) }) { Text("+1") }
}
