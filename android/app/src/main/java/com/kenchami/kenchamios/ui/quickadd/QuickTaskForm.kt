package com.kenchami.kenchamios.ui.quickadd

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.kenchami.kenchamios.core.RepeatRule
import com.kenchami.kenchamios.core.startOfDay
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.common.ChoiceChips
import com.kenchami.kenchamios.ui.common.formatDate
import com.kenchami.kenchamios.ui.common.label
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint
import java.time.Clock
import java.time.LocalDate

private val REPEAT_CHOICES = listOf(null, "daily:1", "weekly:1", "monthly:1", "weekdays:1")

/**
 * One line like "Pay rent friday !high #home every month", read as it is typed.
 * The chips set what the line leaves open; a task without a project goes to the inbox.
 */
@Composable
fun TaskForm(projects: List<ProjectSummary>, clock: Clock, initialText: String, onSubmit: (NewTask) -> Unit) {
    val tint = Area.Projects.tint
    var text by rememberSaveable { mutableStateOf(initialText) }
    var projectId by rememberSaveable { mutableStateOf(Inbox.ID) }
    var priority by rememberSaveable { mutableStateOf(Priority.Medium) }
    var due by rememberSaveable { mutableStateOf(DueChoice.None) }
    var pickedDate by rememberSaveable { mutableStateOf<Long?>(null) }
    var picking by rememberSaveable { mutableStateOf(false) }
    var repeat by rememberSaveable { mutableStateOf<String?>(null) }
    val today = LocalDate.now(clock)
    val parsed = parseQuickAdd(text, today)
    val tagged = parsed.project?.let { matchProject(it, projects) }
    val chosenDue = when (due) {
        DueChoice.None -> null
        DueChoice.Today -> today
        DueChoice.Tomorrow -> today.plusDays(1)
        DueChoice.Pick -> pickedDate?.let { LocalDate.ofEpochDay(it) }
    }
    val task = NewTask(
        projectId = tagged?.project?.id ?: projectId,
        title = parsed.title,
        priority = parsed.priority ?: priority,
        dueAt = (parsed.due ?: chosenDue)?.let { startOfDay(it, clock.zone) },
        recurrence = parsed.recurrence ?: repeat,
    )

    // the chips can outgrow a small screen
    FormColumn(Modifier.verticalScroll(rememberScrollState())) {
        AppTextField(text, { text = it }, "Task", Modifier.fillMaxWidth(), singleLine = true)
        ParsedPreview(parsed, tagged, clock)
        if (tagged == null) {
            Text("Project")
            ChoiceChips(
                listOf<ProjectSummary?>(null) + projects,
                projects.firstOrNull { it.project.id == projectId },
                { it?.project?.name ?: Inbox.NAME },
                { projectId = it?.project?.id ?: Inbox.ID },
                tint = tint,
            )
        }
        if (parsed.priority == null) {
            Text("Priority")
            ChoiceChips(Priority.entries, priority, { it.label }, { priority = it }, tint = tint)
        }
        if (parsed.due == null) {
            Text("Due")
            ChoiceChips(DueChoice.entries, due, { it.label }, tint = tint, onSelect = {
                due = it
                picking = it == DueChoice.Pick
            })
        }
        if (parsed.recurrence == null) {
            Text("Repeat")
            ChoiceChips(REPEAT_CHOICES, repeat, { rule ->
                rule?.let { RepeatRule.parse(it)?.label } ?: "Never"
            }, { repeat = it }, tint = tint)
        }
        if (picking) {
            DatePickerSheet(
                onPick = {
                    pickedDate = it
                    picking = false
                },
                onDismiss = {
                    picking = false
                    if (pickedDate == null) due = DueChoice.None
                },
            )
        }
        Button(onClick = { onSubmit(task) }, enabled = task.title.isNotBlank()) { Text("Add task") }
    }
}

// what the line was read as, so a typo shows before the task is added
@Composable
private fun ParsedPreview(parsed: QuickTask, tagged: ProjectSummary?, clock: Clock) {
    val found = listOfNotNull(
        parsed.due?.let { "Due ${formatDate(startOfDay(it, clock.zone), clock.zone)}" },
        parsed.priority?.let { "${it.label} priority" },
        parsed.recurrence?.let { RepeatRule.parse(it)?.label },
        tagged?.project?.name,
    )
    val muted = MaterialTheme.colorScheme.onSurfaceVariant
    val hint = "Type dates like tomorrow or friday, !high, #project and every week."
    Text(found.joinToString(" - ").ifEmpty { hint }, style = MaterialTheme.typography.bodySmall, color = muted)
    if (parsed.project != null && tagged == null) {
        Text("No project named #${parsed.project}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error)
    }
}
