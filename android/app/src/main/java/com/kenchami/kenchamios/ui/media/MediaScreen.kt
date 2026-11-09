package com.kenchami.kenchamios.ui.media

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
import com.kenchami.kenchamios.ui.common.ChipRow
import com.kenchami.kenchamios.ui.common.EmptyState
import com.kenchami.kenchamios.ui.common.QuickAddButton
import com.kenchami.kenchamios.ui.common.TintedIcon
import com.kenchami.kenchamios.ui.common.icon
import com.kenchami.kenchamios.ui.common.label
import com.kenchami.kenchamios.ui.common.listMotion
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint

@Composable
fun MediaScreen(onOpenEntry: (String) -> Unit, onQuickAdd: () -> Unit, viewModel: MediaViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    AppScreen("Media", area = Area.Media, actions = { QuickAddButton(onQuickAdd) }) { padding ->
        MediaContent(state, viewModel::setType, viewModel::setStatus, viewModel::addOne, onOpenEntry, onQuickAdd, padding)
    }
}

@Composable
fun MediaContent(
    state: MediaState,
    onType: (MediaType?) -> Unit,
    onStatus: (MediaStatus?) -> Unit,
    onAddOne: (String) -> Unit,
    onOpenEntry: (String) -> Unit,
    onAddTitle: () -> Unit,
    padding: PaddingValues,
) {
    val tint = Area.Media.tint
    val filtered = state.filters != MediaFilters()
    val chips = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)
    Column(Modifier.padding(padding)) {
        ChipRow(listOf(null) + MediaType.entries, state.filters.type, { it?.label ?: "All types" }, onType, chips, tint, { it?.icon })
        ChipRow(listOf(null) + MediaStatus.entries, state.filters.status, { it?.label ?: "All statuses" }, onStatus, chips, tint)
        when {
            state.entries.isNotEmpty() -> Unit
            filtered -> EmptyState(Area.Media.icon, "Nothing matches these filters.", "Show everything", onAction = {
                onType(null)
                onStatus(null)
            }, tint = tint)
            else -> EmptyState(Area.Media.icon, "Your library is empty.", "Add a title", onAddTitle, tint = tint)
        }
        LazyColumn(Modifier.testTag("media")) {
            items(state.entries, key = { it.id }) { entry ->
                ListItem(
                    headlineContent = { Text(entry.title) },
                    supportingContent = { Text("${entry.mediaType.label} - ${entry.status.label} - ${progressText(entry)}") },
                    leadingContent = { TintedIcon(entry.mediaType.icon, tint) },
                    trailingContent = { AddOneButton(entry, onAddOne) },
                    modifier = Modifier.listMotion(this).clickable { onOpenEntry(entry.id) },
                )
            }
        }
    }
}

fun progressText(entry: LibraryEntry): String = entry.progressTotal?.let { "${entry.progress}/$it" } ?: "${entry.progress}"
