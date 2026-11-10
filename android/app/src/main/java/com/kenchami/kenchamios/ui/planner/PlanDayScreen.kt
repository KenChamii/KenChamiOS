package com.kenchami.kenchamios.ui.planner

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.core.DayBounds
import com.kenchami.kenchamios.data.db.PlannedTask
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.common.SectionTitle
import com.kenchami.kenchamios.ui.common.TaskTitle
import com.kenchami.kenchamios.ui.common.listMotion
import com.kenchami.kenchamios.ui.theme.Area
import java.time.LocalDate

/** What Plan my day can do with a task. */
class PlanActions(
    val onSearch: (String) -> Unit,
    val onAdd: (PlannedTask) -> Unit,
    val onEstimate: (PlannedTask, Int?) -> Unit,
    val onRemove: (PlannedTask) -> Unit,
)

@Composable
fun PlanDayScreen(onBack: () -> Unit, viewModel: PlanDayViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val query by viewModel.query.collectAsStateWithLifecycle()
    AppScreen("Plan my day", onBack = onBack, area = Area.Focus) { padding ->
        state?.let {
            PlanDayContent(it, query, PlanActions(viewModel::search, viewModel::add, viewModel::setEstimate, viewModel::remove), padding)
        }
    }
}

@Composable
fun PlanDayContent(state: PlanDayState, query: String, actions: PlanActions, padding: PaddingValues) {
    LazyColumn(contentPadding = padding, modifier = Modifier.testTag("plan-day")) {
        item { CapacityBar(state.capacity, Modifier.padding(horizontal = 20.dp, vertical = 8.dp)) }
        item { SectionTitle("Today") }
        if (state.planned.isEmpty()) item(key = "planned-empty") { Hint("Nothing planned yet. Add tasks below.") }
        items(state.planned, key = { "planned-" + it.id }) { task ->
            ListItem(
                headlineContent = { TaskTitle(task.title, task.status == TaskStatus.Done) },
                supportingContent = { Text(task.projectName) },
                trailingContent = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        EstimatePicker(task.title, task.durationMin) { actions.onEstimate(task, it) }
                        IconButton(onClick = { actions.onRemove(task) }) {
                            Icon(Icons.Filled.Close, contentDescription = "Take ${task.title} off today")
                        }
                    }
                },
                modifier = Modifier.listMotion(this),
            )
        }
        item { SectionTitle("Add tasks") }
        item {
            AppTextField(
                query,
                actions.onSearch,
                "Search open tasks",
                Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                singleLine = true,
                leadingIcon = Icons.Filled.Search,
            )
        }
        if (state.candidates.isEmpty()) {
            item(key = "candidates-empty") {
                Hint(if (query.isBlank()) "Nothing due, in progress or in the Inbox." else "No open task matches.")
            }
        }
        items(state.candidates, key = { "candidate-" + it.id }) { task ->
            val details = listOf(task.projectName) + candidateNotes(task, state.today, state.bounds)
            ListItem(
                headlineContent = { Text(task.title) },
                supportingContent = { Text(details.joinToString(" - ")) },
                trailingContent = {
                    FilledTonalButton(
                        onClick = { actions.onAdd(task) },
                        modifier = Modifier.semantics { contentDescription = "Plan ${task.title} for today" },
                    ) { Text("Today") }
                },
                modifier = Modifier.listMotion(this),
            )
        }
    }
}

/** Why a task is offered: due, left over from an earlier plan or already started. */
fun candidateNotes(task: PlannedTask, today: LocalDate, bounds: DayBounds): List<String> = listOfNotNull(
    task.dueAt?.let {
        when {
            it < bounds.start -> "Overdue"
            it < bounds.end -> "Due today"
            else -> null
        }
    },
    "Left over".takeIf { task.plannedOn != null && task.plannedOn < today.toString() },
    "In progress".takeIf { task.status != TaskStatus.Todo },
)

@Composable
internal fun Hint(text: String) {
    Text(
        text,
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp),
    )
}
