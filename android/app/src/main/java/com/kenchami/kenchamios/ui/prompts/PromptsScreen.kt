package com.kenchami.kenchamios.ui.prompts

import android.content.ClipData
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.style.TextOverflow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.data.db.entity.PromptEntity
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.EmptyState
import com.kenchami.kenchamios.ui.common.QuickAddButton
import com.kenchami.kenchamios.ui.common.TintedIcon
import com.kenchami.kenchamios.ui.common.icon
import com.kenchami.kenchamios.ui.common.listMotion
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint
import kotlinx.coroutines.launch

@Composable
fun PromptsScreen(onQuickAdd: () -> Unit, viewModel: PromptsViewModel = hiltViewModel()) {
    val prompts by viewModel.prompts.collectAsStateWithLifecycle()
    val clipboard = LocalClipboard.current
    val snackbar = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()

    AppScreen("Prompts", area = Area.Workflow, snackbar = snackbar, actions = { QuickAddButton(onQuickAdd) }) { padding ->
        PromptsContent(
            prompts = prompts,
            onCopy = { prompt ->
                scope.launch {
                    val text = viewModel.copyText(prompt.id)
                    clipboard.setClipEntry(ClipEntry(ClipData.newPlainText(prompt.title, text)))
                    snackbar.showSnackbar("Copied ${prompt.title}")
                }
            },
            onFavorite = { viewModel.setFavorite(it.id, !it.favorite) },
            onDelete = { viewModel.delete(it.id) },
            onAdd = onQuickAdd,
            padding = padding,
        )
    }
}

@Composable
fun PromptsContent(
    prompts: List<PromptEntity>,
    onCopy: (PromptEntity) -> Unit,
    onFavorite: (PromptEntity) -> Unit,
    onDelete: (PromptEntity) -> Unit,
    onAdd: () -> Unit,
    padding: PaddingValues,
) {
    val tint = Area.Workflow.tint
    if (prompts.isEmpty()) {
        EmptyState(Area.Workflow.icon, "No prompts yet.", "Add a prompt", onAdd, Modifier.padding(padding).testTag("prompts-empty"), tint)
        return
    }
    LazyColumn(contentPadding = padding, modifier = Modifier.testTag("prompts")) {
        items(prompts, key = { it.id }) { prompt ->
            ListItem(
                headlineContent = { Text(prompt.title) },
                supportingContent = { Text(prompt.bodyMd, maxLines = 2, overflow = TextOverflow.Ellipsis) },
                overlineContent = { Text("Used ${prompt.useCount} times") },
                leadingContent = { TintedIcon(Area.Workflow.icon, tint) },
                trailingContent = {
                    Row {
                        IconButton(onClick = { onFavorite(prompt) }) {
                            Icon(
                                Icons.Filled.Star,
                                contentDescription = if (prompt.favorite) "Remove from favorites" else "Add to favorites",
                                tint = if (prompt.favorite) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outline,
                            )
                        }
                        IconButton(onClick = { onDelete(prompt) }) {
                            Icon(Icons.Filled.Delete, contentDescription = "Delete prompt")
                        }
                    }
                },
                modifier = Modifier.listMotion(this).clickable { onCopy(prompt) },
            )
        }
    }
}
