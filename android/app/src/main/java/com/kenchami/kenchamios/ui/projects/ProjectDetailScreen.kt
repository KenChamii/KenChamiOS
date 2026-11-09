package com.kenchami.kenchamios.ui.projects

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.data.model.Stage
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.ConfirmDialog
import com.kenchami.kenchamios.ui.common.EmptyText
import com.kenchami.kenchamios.ui.common.SectionTitle
import com.kenchami.kenchamios.ui.common.formatDate
import com.kenchami.kenchamios.ui.common.label
import java.time.ZoneId

@Composable
fun ProjectDetailScreen(onBack: () -> Unit, viewModel: ProjectDetailViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var confirmDelete by remember { mutableStateOf(false) }
    val current = state

    LaunchedEffect(current) {
        if (current != null && current.summary == null) onBack()
    }

    AppScreen(
        title = current?.summary?.project?.name.orEmpty(),
        onBack = onBack,
        actions = {
            IconButton(onClick = { confirmDelete = true }) {
                Icon(Icons.Filled.Delete, contentDescription = "Delete project")
            }
        },
    ) { padding ->
        if (current?.summary != null) {
            ProjectDetailContent(current, viewModel, ZoneId.systemDefault(), padding)
        }
    }

    if (confirmDelete) {
        ConfirmDialog(
            title = "Delete project",
            text = "The project with its tasks, save points and prompts is deleted.",
            confirmLabel = "Delete",
            onConfirm = {
                confirmDelete = false
                viewModel.deleteProject()
            },
            onDismiss = { confirmDelete = false },
        )
    }
}

@Composable
private fun ProjectDetailContent(state: ProjectDetailState, actions: ProjectDetailViewModel, zone: ZoneId, padding: PaddingValues) {
    val project = checkNotNull(state.summary).project
    var newTask by rememberSaveable { mutableStateOf("") }
    var savePointDialog by remember { mutableStateOf(false) }
    var taskToDelete by remember { mutableStateOf<String?>(null) }
    val startFocus = rememberFocusStarter(actions::startFocus)

    LazyColumn(contentPadding = padding, modifier = Modifier.testTag("project-detail")) {
        item {
            Row(
                Modifier.horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Stage.entries.forEach { stage ->
                    FilterChip(
                        selected = project.stage == stage,
                        onClick = { actions.setStage(stage) },
                        label = { Text(stage.label) },
                    )
                }
            }
        }
        if (project.description.isNotEmpty()) item { Text(project.description, Modifier.padding(16.dp)) }

        item { SectionTitle("Tasks") }
        item {
            Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
                OutlinedTextField(
                    value = newTask,
                    onValueChange = { newTask = it },
                    label = { Text("New task") },
                    singleLine = true,
                    modifier = Modifier.weight(1f),
                )
                IconButton(
                    onClick = {
                        actions.addTask(newTask)
                        newTask = ""
                    },
                    enabled = newTask.isNotBlank(),
                ) { Icon(Icons.Filled.Add, contentDescription = "Add task") }
            }
        }
        if (state.tasks.isEmpty()) item { EmptyText("No tasks yet.") }
        TaskStatus.entries.forEach { status ->
            val inColumn = state.tasks.filter { it.status == status }
            if (inColumn.isNotEmpty()) {
                item(key = "header-${status.wire}") { SectionTitle(status.label) }
                items(inColumn, key = { it.id }) { task ->
                    TaskRow(
                        task,
                        zone,
                        canStartFocus = !state.focusRunning,
                        onStartFocus = { startFocus(task.id) },
                        onStatus = { actions.setTaskStatus(task.id, it) },
                        onDelete = { taskToDelete = task.id },
                    )
                }
            }
        }

        item { SectionTitle("Save points") }
        item {
            TextButton(onClick = { savePointDialog = true }, modifier = Modifier.padding(horizontal = 8.dp)) {
                Text("New save point")
            }
        }
        if (state.savePoints.isEmpty()) item { EmptyText("No save points yet.") }
        items(state.savePoints, key = { it.id }) { point ->
            ListItem(
                headlineContent = { Text(point.note) },
                supportingContent = {
                    Text(
                        listOfNotNull(
                            "${point.stage.label} - ${formatDate(point.createdAt, zone)}",
                            point.doneMd.takeIf { it.isNotEmpty() }?.let { "Done: $it" },
                            point.nextMd.takeIf { it.isNotEmpty() }?.let { "Next: $it" },
                        ).joinToString("\n"),
                    )
                },
                trailingContent = {
                    IconButton(onClick = { actions.deleteSavePoint(point.id) }) {
                        Icon(Icons.Filled.Delete, contentDescription = "Delete save point")
                    }
                },
            )
        }
    }

    if (savePointDialog) {
        SavePointDialog(
            onSave = { note, done, next ->
                savePointDialog = false
                actions.addSavePoint(note, done, next)
            },
            onDismiss = { savePointDialog = false },
        )
    }
    taskToDelete?.let { id ->
        ConfirmDialog(
            title = "Delete task",
            text = "The task and its subtasks are deleted.",
            confirmLabel = "Delete",
            onConfirm = {
                taskToDelete = null
                actions.deleteTask(id)
            },
            onDismiss = { taskToDelete = null },
        )
    }
}
