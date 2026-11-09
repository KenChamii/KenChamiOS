package com.kenchami.kenchamios.ui.projects

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.ui.common.AppTextField

@Composable
fun SavePointDialog(onSave: (note: String, done: String, next: String) -> Unit, onDismiss: () -> Unit) {
    var note by rememberSaveable { mutableStateOf("") }
    var done by rememberSaveable { mutableStateOf("") }
    var next by rememberSaveable { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("New save point") },
        text = {
            Column {
                AppTextField(note, { note = it }, "Note", Modifier.fillMaxWidth())
                AppTextField(done, { done = it }, "Done", Modifier.fillMaxWidth().padding(top = 8.dp))
                AppTextField(next, { next = it }, "Next", Modifier.fillMaxWidth().padding(top = 8.dp))
            }
        },
        confirmButton = {
            TextButton(onClick = { onSave(note, done, next) }, enabled = note.isNotBlank()) { Text("Save") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    )
}
