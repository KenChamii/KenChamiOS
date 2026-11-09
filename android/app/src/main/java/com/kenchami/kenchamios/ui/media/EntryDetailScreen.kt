package com.kenchami.kenchamios.ui.media

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
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
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
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
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.common.ChoiceChips
import com.kenchami.kenchamios.ui.common.ConfirmDialog
import com.kenchami.kenchamios.ui.common.TintedIcon
import com.kenchami.kenchamios.ui.common.icon
import com.kenchami.kenchamios.ui.common.label
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint

@Composable
fun EntryDetailScreen(onBack: () -> Unit, viewModel: EntryDetailViewModel = hiltViewModel()) {
    val entry by viewModel.entry.collectAsStateWithLifecycle()
    var confirmDelete by remember { mutableStateOf(false) }

    AppScreen(
        title = entry?.title.orEmpty(),
        onBack = onBack,
        area = Area.Media,
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
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            TintedIcon(entry.mediaType.icon, Area.Media.tint)
            Text("${entry.mediaType.label} - ${progressText(entry)}", style = MaterialTheme.typography.titleMedium)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            FilledTonalButton(onClick = onSubtractOne, enabled = entry.progress > 0) { Text("-1") }
            Button(onClick = onAddOne, enabled = entry.canIncrement) { Text("+1") }
        }
        ChoiceChips(MediaStatus.entries, status, { it.label }, { status = it }, tint = Area.Media.tint)
        val number = KeyboardOptions(keyboardType = KeyboardType.Number)
        val wide = Modifier.fillMaxWidth()
        AppTextField(total, { total = it }, "Total", wide, singleLine = true, keyboardOptions = number)
        AppTextField(rating, { rating = it }, "Rating (1 to 10)", wide, singleLine = true, keyboardOptions = number)
        AppTextField(notes, { notes = it }, "Notes", wide)
        AppTextField(tags, { tags = it }, "Tags, separated by commas", wide)
        Button(onClick = { input?.let(onSave) }, enabled = input != null) { Text("Save") }
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
