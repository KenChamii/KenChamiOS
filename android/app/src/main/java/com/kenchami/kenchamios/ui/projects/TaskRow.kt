package com.kenchami.kenchamios.ui.projects

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.Checkbox
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.kenchami.kenchamios.core.RepeatRule
import com.kenchami.kenchamios.data.db.entity.TaskEntity
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.ui.common.TaskTitle
import com.kenchami.kenchamios.ui.common.formatDate
import com.kenchami.kenchamios.ui.common.label
import java.time.ZoneId

@Composable
fun TaskRow(
    task: TaskEntity,
    zone: ZoneId,
    canStartFocus: Boolean,
    onStartFocus: () -> Unit,
    onStatus: (TaskStatus) -> Unit,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
    onMove: (() -> Unit)? = null,
) {
    var menuOpen by remember { mutableStateOf(false) }
    val details = listOfNotNull(
        task.priority.label,
        task.dueAt?.let { "due ${formatDate(it, zone)}" },
        task.recurrence?.let { RepeatRule.parse(it)?.label },
    )
    ListItem(
        modifier = modifier,
        headlineContent = { TaskTitle(task.title, task.status == TaskStatus.Done) },
        supportingContent = { Text(details.joinToString(" - ")) },
        leadingContent = {
            Checkbox(
                checked = task.status == TaskStatus.Done,
                onCheckedChange = { onStatus(if (it) TaskStatus.Done else TaskStatus.Todo) },
            )
        },
        trailingContent = {
            IconButton(onClick = { menuOpen = true }) {
                Icon(Icons.Filled.MoreVert, contentDescription = "Task actions")
            }
            DropdownMenu(expanded = menuOpen, onDismissRequest = { menuOpen = false }) {
                if (canStartFocus && task.status != TaskStatus.Done) {
                    DropdownMenuItem(
                        text = { Text("Start focus") },
                        onClick = {
                            menuOpen = false
                            onStartFocus()
                        },
                    )
                }
                TaskStatus.entries.filter { it != task.status }.forEach { status ->
                    DropdownMenuItem(
                        text = { Text("Move to ${status.label}") },
                        onClick = {
                            menuOpen = false
                            onStatus(status)
                        },
                    )
                }
                onMove?.let { move ->
                    DropdownMenuItem(
                        text = { Text("Move to project") },
                        onClick = {
                            menuOpen = false
                            move()
                        },
                    )
                }
                DropdownMenuItem(
                    text = { Text("Delete") },
                    onClick = {
                        menuOpen = false
                        onDelete()
                    },
                )
            }
        },
    )
}
