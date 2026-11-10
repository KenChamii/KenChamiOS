package com.kenchami.kenchamios.ui.today

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material3.Checkbox
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.context.ChecklistRun
import com.kenchami.kenchamios.core.formatMinutes
import com.kenchami.kenchamios.data.db.DueTask
import com.kenchami.kenchamios.data.db.PlannedTask
import com.kenchami.kenchamios.data.repo.nextLabel
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.EmptyState
import com.kenchami.kenchamios.ui.common.HeroStat
import com.kenchami.kenchamios.ui.common.LabelValueRow
import com.kenchami.kenchamios.ui.common.QuickAddButton
import com.kenchami.kenchamios.ui.common.RulesButton
import com.kenchami.kenchamios.ui.common.SectionTitle
import com.kenchami.kenchamios.ui.common.SyncButton
import com.kenchami.kenchamios.ui.common.TaskTitle
import com.kenchami.kenchamios.ui.common.TintedIcon
import com.kenchami.kenchamios.ui.common.TonalCard
import com.kenchami.kenchamios.ui.common.icon
import com.kenchami.kenchamios.ui.common.label
import com.kenchami.kenchamios.ui.common.listMotion
import com.kenchami.kenchamios.ui.common.timeAgo
import com.kenchami.kenchamios.ui.media.AddOneButton
import com.kenchami.kenchamios.ui.quickadd.QuickAddTab
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint

/** Where Today can send the user. */
class TodayNav(
    val openProject: (String) -> Unit,
    val openEntry: (String) -> Unit,
    val quickAdd: (QuickAddTab) -> Unit,
    val openSync: () -> Unit,
    val openInbox: () -> Unit,
    val openHabits: () -> Unit,
    val openTrash: () -> Unit,
    val openPlan: () -> Unit,
    val openReview: () -> Unit,
    val openNotes: () -> Unit,
)

@Composable
fun TodayScreen(nav: TodayNav, onOpenRules: () -> Unit, viewModel: TodayViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val focus by viewModel.focus.collectAsStateWithLifecycle()
    val checklists by viewModel.checklists.collectAsStateWithLifecycle()
    AppScreen("Today", actions = {
        RulesButton(onOpenRules)
        SyncButton(nav.openSync)
        QuickAddButton { nav.quickAdd(QuickAddTab.Task) }
        MoreMenu(nav)
    }) { padding ->
        state?.let {
            val timer = TimerActions(viewModel::pauseTimer, viewModel::resumeTimer, viewModel::stopTimer)
            val checklistActions = ChecklistActions(viewModel::toggleChecklist, viewModel::dismissChecklist)
            TodayContent(it, focus, timer, viewModel::finishTask, viewModel::addOne, nav, padding, checklists, checklistActions)
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
    nav: TodayNav,
    padding: PaddingValues,
    checklists: List<ChecklistRun> = emptyList(),
    checklistActions: ChecklistActions = ChecklistActions({ _, _ -> }, {}),
) {
    val firstRun = state.setup.isNotEmpty()
    LazyColumn(contentPadding = padding, modifier = Modifier.testTag("today")) {
        item { Greeting(state) }
        if (firstRun) {
            item { SetupCard(state.setup) { onSetupStep(it, nav) } }
        } else {
            item { TodayHero(state) }
        }
        focus?.let { card ->
            item { FocusCardView(card, System::currentTimeMillis, timer.onPause, timer.onResume, timer.onStop) }
        }
        items(checklists, key = { "checklist-" + it.ruleId }) { ChecklistCard(it, checklistActions) }
        if (!firstRun) {
            item { DayActions(nav) }
            if (state.planned.isNotEmpty()) item { SectionTitle("Planned") }
            items(state.planned, key = { "planned-" + it.id }) { task -> PlannedTaskRow(task, onFinishTask, nav.openProject) }
            // a planned task shows once, in the plan
            val plannedIds = state.planned.map { it.id }.toSet()
            val due = state.due.filter { it.id !in plannedIds }
            item { SectionTitle("Tasks due") }
            if (due.isEmpty() && state.planned.isEmpty()) {
                item(key = "due-empty") {
                    EmptyState(Icons.Filled.CheckCircle, "Nothing due today.", "Add a task", { nav.quickAdd(QuickAddTab.Task) })
                }
            }
            items(due, key = { it.id }) { task -> DueTaskRow(task, state, onFinishTask, nav.openProject) }
        }

        if (state.continueItems.isNotEmpty()) item { SectionTitle("Continue") }
        items(state.continueItems, key = { it.id }) { entry ->
            ListItem(
                headlineContent = { Text(entry.title) },
                supportingContent = { Text("${entry.mediaType.label} - ${nextLabel(entry)}") },
                leadingContent = { TintedIcon(entry.mediaType.icon, Area.Media.tint) },
                trailingContent = { AddOneButton(entry, onAddOne) },
                modifier = Modifier.listMotion(this).clickable { nav.openEntry(entry.id) },
            )
        }

        if (state.projects.isNotEmpty()) item { SectionTitle("Active projects") }
        items(state.projects, key = { it.project.id }) { summary ->
            ListItem(
                headlineContent = { Text(summary.project.name) },
                supportingContent = { Text("${summary.project.stage.label} - ${summary.tasksDone}/${summary.tasksTotal} tasks") },
                leadingContent = { TintedIcon(summary.project.stage.icon, Area.Projects.tint) },
                modifier = Modifier.listMotion(this).clickable { nav.openProject(summary.project.id) },
            )
        }

        if (state.activity.isNotEmpty()) item { SectionTitle("Recent activity") }
        items(state.activity, key = { it.id }) { row ->
            ListItem(
                headlineContent = { Text(row.summary) },
                supportingContent = { Text(timeAgo(row.at, state.now)) },
                modifier = Modifier.listMotion(this),
            )
        }
    }
}

private fun onSetupStep(step: SetupStep, nav: TodayNav) = when (step) {
    SetupStep.CreateProject -> nav.quickAdd(QuickAddTab.Idea)
    SetupStep.AddTask -> nav.quickAdd(QuickAddTab.Task)
    SetupStep.AddMedia -> nav.quickAdd(QuickAddTab.Media)
    SetupStep.PairDesktop -> nav.openSync()
}

// the row keeps its tick for the moment it takes the list to drop the finished task
@Composable
private fun LazyItemScope.DueTaskRow(task: DueTask, state: TodayState, onFinishTask: (String) -> Unit, onOpenProject: (String) -> Unit) {
    var ticked by rememberSaveable(task.id) { mutableStateOf(false) }
    val overdue = task.dueAt < state.bounds.start
    ListItem(
        headlineContent = { TaskTitle(task.title, ticked) },
        supportingContent = { Text(if (overdue) "${task.projectName} - overdue" else task.projectName) },
        leadingContent = {
            Checkbox(checked = ticked, enabled = !ticked, onCheckedChange = {
                ticked = true
                onFinishTask(task.id)
            })
        },
        modifier = Modifier.listMotion(this).clickable { onOpenProject(task.projectId) },
    )
}

@Composable
private fun LazyItemScope.PlannedTaskRow(task: PlannedTask, onFinishTask: (String) -> Unit, onOpenProject: (String) -> Unit) {
    var ticked by rememberSaveable(task.id) { mutableStateOf(false) }
    ListItem(
        headlineContent = { TaskTitle(task.title, ticked) },
        supportingContent = { Text(listOfNotNull(task.projectName, task.durationMin?.let(::formatMinutes)).joinToString(" - ")) },
        leadingContent = {
            Checkbox(checked = ticked, enabled = !ticked, onCheckedChange = {
                ticked = true
                onFinishTask(task.id)
            })
        },
        modifier = Modifier.listMotion(this).clickable { onOpenProject(task.projectId) },
    )
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun DayActions(nav: TodayNav) {
    FlowRow(Modifier.padding(horizontal = 20.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        FilledTonalButton(onClick = nav.openPlan) { Text("Plan my day") }
        FilledTonalButton(onClick = nav.openReview) { Text("Review my day") }
        FilledTonalButton(onClick = nav.openNotes) { Text("Today's note") }
    }
}

@Composable
private fun Greeting(state: TodayState) {
    Column(Modifier.padding(start = 20.dp, end = 20.dp, top = 4.dp)) {
        Text(state.greeting, style = MaterialTheme.typography.headlineMedium)
        Text(state.date, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun TodayHero(state: TodayState) {
    val overdue = state.due.count { it.dueAt < state.bounds.start }
    Column(Modifier.padding(horizontal = 20.dp, vertical = 12.dp)) {
        HeroStat(state.due.size - overdue, "due today")
        TonalCard(Modifier.fillMaxWidth().padding(top = 16.dp)) {
            Column(Modifier.padding(horizontal = 20.dp, vertical = 8.dp)) {
                LabelValueRow("Overdue", overdue.toString())
                LabelValueRow("In progress", state.continueItems.size.toString())
                LabelValueRow("Active projects", state.projects.size.toString())
            }
        }
    }
}
