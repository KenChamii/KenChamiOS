package com.kenchami.kenchamios.ui.media

import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.FilterChip
import androidx.compose.material3.ListItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.EmptyText
import com.kenchami.kenchamios.ui.common.QuickAddButton
import com.kenchami.kenchamios.ui.common.label

@Composable
fun MediaScreen(onOpenEntry: (String) -> Unit, onQuickAdd: () -> Unit, viewModel: MediaViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    AppScreen("Media", actions = { QuickAddButton(onQuickAdd) }) { padding ->
        MediaContent(state, viewModel::setType, viewModel::setStatus, viewModel::addOne, onOpenEntry, padding)
    }
}

@Composable
fun MediaContent(
    state: MediaState,
    onType: (MediaType?) -> Unit,
    onStatus: (MediaStatus?) -> Unit,
    onAddOne: (String) -> Unit,
    onOpenEntry: (String) -> Unit,
    padding: PaddingValues,
) {
    Column(Modifier.padding(padding)) {
        ChipRow("All types", MediaType.entries, state.filters.type, { it.label }, onType)
        ChipRow("All statuses", MediaStatus.entries, state.filters.status, { it.label }, onStatus)
        if (state.entries.isEmpty()) EmptyText("Nothing here. Use + to add a title.")
        LazyColumn(Modifier.testTag("media")) {
            items(state.entries, key = { it.id }) { entry ->
                ListItem(
                    headlineContent = { Text(entry.title) },
                    supportingContent = { Text("${entry.mediaType.label} - ${entry.status.label} - ${progressText(entry)}") },
                    trailingContent = { AddOneButton(entry, onAddOne) },
                    modifier = Modifier.clickable { onOpenEntry(entry.id) },
                )
            }
        }
    }
}

fun progressText(entry: LibraryEntry): String = entry.progressTotal?.let { "${entry.progress}/$it" } ?: "${entry.progress}"

@Composable
private fun <T> ChipRow(allLabel: String, options: List<T>, selected: T?, label: (T) -> String, onSelect: (T?) -> Unit) {
    Row(
        Modifier.horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        FilterChip(selected = selected == null, onClick = { onSelect(null) }, label = { Text(allLabel) })
        options.forEach { option ->
            FilterChip(selected = selected == option, onClick = { onSelect(option) }, label = { Text(label(option)) })
        }
    }
}
