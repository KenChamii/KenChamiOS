package com.kenchami.kenchamios.ui.trash

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.ListItem
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.data.db.TrashRow
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.EmptyState
import com.kenchami.kenchamios.ui.common.label
import com.kenchami.kenchamios.ui.common.listMotion
import com.kenchami.kenchamios.ui.common.timeAgo
import kotlinx.coroutines.launch

@Composable
fun TrashScreen(onBack: () -> Unit, viewModel: TrashViewModel = hiltViewModel()) {
    val rows by viewModel.rows.collectAsStateWithLifecycle()
    val snackbar = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()

    AppScreen("Trash", onBack = onBack, snackbar = snackbar) { padding ->
        LazyColumn(contentPadding = padding, modifier = Modifier.testTag("trash")) {
            if (rows?.isEmpty() == true) {
                item(key = "trash-empty") { EmptyState(Icons.Filled.Delete, "The trash is empty.", "Back", onBack) }
            }
            items(rows.orEmpty(), key = { "${it.kind.wire}-${it.id}" }) { row ->
                ListItem(
                    headlineContent = { Text(row.title) },
                    supportingContent = { Text(details(row, System.currentTimeMillis())) },
                    trailingContent = {
                        TextButton(onClick = {
                            scope.launch {
                                val failure = runCatching { viewModel.restore(row) }.exceptionOrNull()
                                snackbar.showSnackbar(failure?.let { "Could not restore: ${it.message}" } ?: "Restored ${row.title}")
                            }
                        }) { Text("Restore") }
                    },
                    modifier = Modifier.listMotion(this),
                )
            }
        }
    }
}

private fun details(row: TrashRow, now: Long): String =
    listOfNotNull(row.kind.label, row.detail, "deleted ${timeAgo(row.deletedAt, now)}").joinToString(" - ")
