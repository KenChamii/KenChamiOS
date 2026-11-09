package com.kenchami.kenchamios.ui.today

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ListItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.context.ChecklistRun
import com.kenchami.kenchamios.data.repo.nextLabel
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.EmptyText
import com.kenchami.kenchamios.ui.common.QuickAddButton
import com.kenchami.kenchamios.ui.common.RulesButton
import com.kenchami.kenchamios.ui.common.SectionTitle
import com.kenchami.kenchamios.ui.common.SyncButton
import com.kenchami.kenchamios.ui.common.label
import com.kenchami.kenchamios.ui.common.timeAgo
import com.kenchami.kenchamios.ui.media.AddOneButton

@Composable
fun TodayScreen(
    onOpenProject: (String) -> Unit,
    onOpenEntry: (String) -> Unit,
    onQuickAdd: () -> Unit,
    onOpenSync: () -> Unit,
    onOpenRules: () -> Unit,
    viewModel: TodayViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val focus by viewModel.focus.collectAsStateWithLifecycle()
    val checklists by viewModel.checklists.collectAsStateWithLifecycle()
    AppScreen("Today", actions = {
        RulesButton(onOpenRules)
        SyncButton(onOpenSync)
        QuickAddButton(onQuickAdd)
    }) { padding ->
        state?.let {
            val timer = TimerActions(viewModel::pauseTimer, viewModel::resumeTimer, viewModel::stopTimer)
            val checklistActions = ChecklistActions(viewModel::toggleChecklist, viewModel::dismissChecklist)
            TodayContent(
                it, focus, timer, viewModel::finishTask, viewModel::addOne, onOpenProject, onOpenEntry,
                padding, checklists, checklistActions,
            )
        }
    }
}

@Composable
fun TodayContent(
    state: TodayState,
    focus: FocusCard?,
    timer: TimerActions,
    onFinishTask: (String) -> Unit,
    onAddOne: (String) -> Unit,
    onOpenProject: (String) -> Unit,
    onOpenEntry: (String) -> Unit,
    padding: PaddingValues,
    checklists: List<ChecklistRun> = emptyList(),
    checklistActions: ChecklistActions = ChecklistActions({ _, _ -> }, {}),
) {
    LazyColumn(contentPadding = padding, modifier = Modifier.testTag("today")) {
        focus?.let { card ->
            item { FocusCardView(card, System::currentTimeMillis, timer.onPause, timer.onResume, timer.onStop) }
        }
        items(checklists, key = { "checklist-" + it.ruleId }) { ChecklistCard(it, checklistActions) }
        item { SectionTitle("Tasks due") }
        if (state.due.isEmpty()) item { EmptyText("Nothing due or overdue.") }
        items(state.due, key = { it.id }) { task ->
            val overdue = task.dueAt < state.bounds.start
            ListItem(
                headlineContent = { Text(task.title) },
                supportingContent = { Text(if (overdue) "${task.projectName} - overdue" else task.projectName) },
                leadingContent = { Checkbox(checked = false, onCheckedChange = { onFinishTask(task.id) }) },
                modifier = Modifier.clickable { onOpenProject(task.projectId) },
            )
        }

        item { SectionTitle("Continue") }
        if (state.continueItems.isEmpty()) item { EmptyText("Nothing in progress.") }
        items(state.continueItems, key = { it.id }) { entry ->
            ListItem(
                headlineContent = { Text(entry.title) },
                supportingContent = { Text("${entry.mediaType.label} - ${nextLabel(entry)}") },
                trailingContent = { AddOneButton(entry, onAddOne) },
                modifier = Modifier.clickable { onOpenEntry(entry.id) },
            )
        }

        item { SectionTitle("Active projects") }
        if (state.projects.isEmpty()) item { EmptyText("No active projects.") }
        items(state.projects, key = { it.project.id }) { summary ->
            ListItem(
                headlineContent = { Text(summary.project.name) },
                supportingContent = { Text("${summary.project.stage.label} - ${summary.tasksDone}/${summary.tasksTotal} tasks") },
                modifier = Modifier.clickable { onOpenProject(summary.project.id) },
            )
        }

        item { SectionTitle("Recent activity") }
        if (state.activity.isEmpty()) item { EmptyText("Nothing yet.") }
        items(state.activity, key = { it.id }) { row ->
            ListItem(
                headlineContent = { Text(row.summary) },
                supportingContent = { Text(timeAgo(row.at, state.now)) },
            )
        }
    }
}
