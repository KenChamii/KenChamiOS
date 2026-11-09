package com.kenchami.kenchamios.ui.media

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.repo.EntryInput
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.ConfirmDialog
import com.kenchami.kenchamios.ui.common.label

@Composable
fun EntryDetailScreen(onBack: () -> Unit, viewModel: EntryDetailViewModel = hiltViewModel()) {
    val entry by viewModel.entry.collectAsStateWithLifecycle()
    var confirmDelete by remember { mutableStateOf(false) }

    AppScreen(
        title = entry?.title.orEmpty(),
        onBack = onBack,
        actions = {
            IconButton(onClick = { confirmDelete = true }) {
                Icon(Icons.Filled.Delete, contentDescription = "Delete entry")
            }
        },
    ) { padding ->
        entry?.let { EntryForm(it, viewModel::save, viewModel::addOne, viewModel::subtractOne, padding) }
    }

    if (confirmDelete) {
        ConfirmDialog(
            title = "Remove from library",
            text = "The entry and its progress are deleted.",
            confirmLabel = "Remove",
            onConfirm = {
                confirmDelete = false
                viewModel.delete()
                onBack()
            },
            onDismiss = { confirmDelete = false },
        )
    }
}

@Composable
fun EntryForm(entry: LibraryEntry, onSave: (EntryInput) -> Unit, onAddOne: () -> Unit, onSubtractOne: () -> Unit, padding: PaddingValues) {
    var status by rememberSaveable(entry.id) { mutableStateOf(entry.status) }
    var total by rememberSaveable(entry.id) { mutableStateOf(entry.progressTotal?.toString().orEmpty()) }
    var rating by rememberSaveable(entry.id) { mutableStateOf(entry.rating?.toString().orEmpty()) }
    var notes by rememberSaveable(entry.id) { mutableStateOf(entry.notes) }
    var tags by rememberSaveable(entry.id) { mutableStateOf(entry.tags.joinToString(", ")) }

    val input = formInput(entry, status, total, rating, notes, tags)

    Column(
        Modifier.padding(padding).padding(16.dp).verticalScroll(rememberScrollState()).testTag("entry-form"),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("${entry.mediaType.label} - ${progressText(entry)}")
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedButton(onClick = onSubtractOne, enabled = entry.progress > 0) { Text("-1") }
            Button(onClick = onAddOne, enabled = entry.canIncrement) { Text("+1") }
        }
        StatusChips(status) { status = it }
        OutlinedTextField(
            value = total,
            onValueChange = { total = it },
            label = { Text("Total") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = rating,
            onValueChange = { rating = it },
            label = { Text("Rating (1 to 10)") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(notes, { notes = it }, label = { Text("Notes") }, modifier = Modifier.fillMaxWidth())
        OutlinedTextField(tags, { tags = it }, label = { Text("Tags, separated by commas") }, modifier = Modifier.fillMaxWidth())
        Button(onClick = { input?.let(onSave) }, enabled = input != null) { Text("Save") }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun StatusChips(selected: MediaStatus, onSelect: (MediaStatus) -> Unit) {
    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        MediaStatus.entries.forEach { status ->
            FilterChip(selected = selected == status, onClick = { onSelect(status) }, label = { Text(status.label) })
        }
    }
}

// null while a field holds something the repository would reject
private fun formInput(entry: LibraryEntry, status: MediaStatus, total: String, rating: String, notes: String, tags: String): EntryInput? {
    val totalValue = total.trim().takeIf { it.isNotEmpty() }?.let { it.toIntOrNull() ?: return null }
    val ratingValue = rating.trim().takeIf { it.isNotEmpty() }?.let { it.toIntOrNull() ?: return null }
    val input = EntryInput(
        status = status,
        progressTotal = totalValue,
        rating = ratingValue,
        notes = notes,
        tags = tags.split(","),
        startedAt = entry.startedAt,
        finishedAt = entry.finishedAt,
        platform = entry.platform,
    )
    val valid = (totalValue == null || totalValue >= entry.progress) && (ratingValue == null || ratingValue in 1..10)
    return input.takeIf { valid }
}
