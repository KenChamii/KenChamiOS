package com.kenchami.kenchamios.ui.inbox

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Email
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.common.ConfirmDialog
import com.kenchami.kenchamios.ui.common.EmptyState
import com.kenchami.kenchamios.ui.common.listMotion
import com.kenchami.kenchamios.ui.projects.TaskRow
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint
import java.time.ZoneId

private const val PROJECT_LIST_MAX_DP = 360

@Composable
fun InboxScreen(onBack: () -> Unit, viewModel: InboxViewModel = hiltViewModel()) {
    val inbox by viewModel.inbox.collectAsStateWithLifecycle()
    val projects by viewModel.projects.collectAsStateWithLifecycle()
    var text by rememberSaveable { mutableStateOf("") }
    var moving by remember { mutableStateOf<String?>(null) }
    var deleting by remember { mutableStateOf<String?>(null) }
    val captureFocus = remember { FocusRequester() }
    val zone = ZoneId.systemDefault()

    AppScreen("Inbox", onBack = onBack, area = Area.Projects) { padding ->
        LazyColumn(contentPadding = padding, modifier = Modifier.testTag("inbox")) {
            item {
                Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    AppTextField(
                        text,
                        { text = it },
                        "Capture a task",
                        Modifier.weight(1f).focusRequester(captureFocus),
                        singleLine = true,
                    )
                    IconButton(
                        onClick = {
                            viewModel.capture(text)
                            text = ""
                        },
                        enabled = text.isNotBlank(),
                    ) { Icon(Icons.Filled.Add, contentDescription = "Capture") }
                }
            }
            if (inbox?.isEmpty() == true) {
                item(key = "inbox-empty") {
                    EmptyState(Icons.Filled.Email, "Nothing waiting.", "Capture something", {
                        captureFocus.requestFocus()
                    }, tint = Area.Projects.tint)
                }
            }
            items(inbox.orEmpty(), key = { it.id }) { task ->
                TaskRow(
                    task,
                    zone,
                    canStartFocus = false,
                    onStartFocus = {},
                    onStatus = { viewModel.setStatus(task.id, it) },
                    onDelete = { deleting = task.id },
                    onMove = { moving = task.id },
                    modifier = Modifier.listMotion(this),
                )
            }
        }
    }

    moving?.let { id ->
        MoveDialog(projects, onPick = { projectId ->
            moving = null
            viewModel.moveTo(id, projectId)
        }, onDismiss = { moving = null })
    }
    deleting?.let { id ->
        ConfirmDialog(
            title = "Delete task",
            text = "The task and its subtasks are deleted.",
            confirmLabel = "Delete",
            onConfirm = {
                deleting = null
                viewModel.delete(id)
            },
            onDismiss = { deleting = null },
        )
    }
}

@Composable
private fun MoveDialog(projects: List<ProjectSummary>, onPick: (String) -> Unit, onDismiss: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Move to project") },
        text = {
            if (projects.isEmpty()) {
                Text("There are no projects yet.")
            } else {
                LazyColumn(Modifier.heightIn(max = PROJECT_LIST_MAX_DP.dp)) {
                    items(projects, key = { it.project.id }) { summary ->
                        ListItem(
                            headlineContent = { Text(summary.project.name) },
                            modifier = Modifier.clickable { onPick(summary.project.id) },
                        )
                    }
                }
            }
        },
        confirmButton = {},
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    )
}
