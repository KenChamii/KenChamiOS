package com.kenchami.kenchamios.ui.quickadd

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.core.startOfDay
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.ui.common.EmptyText
import com.kenchami.kenchamios.ui.common.label
import java.time.Clock
import java.time.LocalDate

@Composable
private fun FormColumn(content: @Composable () -> Unit) {
    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) { content() }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun <T> ChoiceChips(options: List<T>, selected: T?, label: (T) -> String, onSelect: (T) -> Unit) {
    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        options.forEach { option ->
            FilterChip(selected = selected == option, onClick = { onSelect(option) }, label = { Text(label(option)) })
        }
    }
}

@Composable
fun TaskForm(
    projects: List<ProjectSummary>,
    clock: Clock,
    onSubmit: (projectId: String, title: String, priority: Priority, dueAt: Long?) -> Unit,
) {
    if (projects.isEmpty()) {
        EmptyText("Add an idea first, a task needs a project.")
        return
    }
    var title by rememberSaveable { mutableStateOf("") }
    var projectId by rememberSaveable { mutableStateOf(projects.first().project.id) }
    var priority by rememberSaveable { mutableStateOf(Priority.Medium) }
    var due by rememberSaveable { mutableStateOf(DueChoice.None) }
    var pickedDate by rememberSaveable { mutableStateOf<Long?>(null) }
    var picking by rememberSaveable { mutableStateOf(false) }
    val today = LocalDate.now(clock)

    FormColumn {
        OutlinedTextField(title, { title = it }, label = { Text("Task") }, singleLine = true, modifier = Modifier.fillMaxWidth())
        Text("Project")
        ChoiceChips(projects, projects.firstOrNull { it.project.id == projectId }, { it.project.name }) { projectId = it.project.id }
        Text("Priority")
        ChoiceChips(Priority.entries, priority, { it.label }) { priority = it }
        Text("Due")
        ChoiceChips(DueChoice.entries, due, { it.label }) {
            due = it
            picking = it == DueChoice.Pick
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
        Button(
            onClick = {
                val date = when (due) {
                    DueChoice.None -> null
                    DueChoice.Today -> today
                    DueChoice.Tomorrow -> today.plusDays(1)
                    DueChoice.Pick -> pickedDate?.let { LocalDate.ofEpochDay(it) }
                }
                onSubmit(projectId, title, priority, date?.let { startOfDay(it, clock.zone) })
            },
            enabled = title.isNotBlank(),
        ) { Text("Add task") }
    }
}

@Composable
fun IdeaForm(onSubmit: (name: String, description: String) -> Unit) {
    var name by rememberSaveable { mutableStateOf("") }
    var description by rememberSaveable { mutableStateOf("") }
    FormColumn {
        OutlinedTextField(name, { name = it }, label = { Text("Idea") }, singleLine = true, modifier = Modifier.fillMaxWidth())
        OutlinedTextField(description, { description = it }, label = { Text("Description") }, modifier = Modifier.fillMaxWidth())
        Button(onClick = { onSubmit(name, description) }, enabled = name.isNotBlank()) { Text("Add idea") }
    }
}

@Composable
fun PromptForm(projects: List<ProjectSummary>, onSubmit: (title: String, body: String, projectId: String?) -> Unit) {
    var title by rememberSaveable { mutableStateOf("") }
    var body by rememberSaveable { mutableStateOf("") }
    var projectId by rememberSaveable { mutableStateOf<String?>(null) }
    FormColumn {
        OutlinedTextField(title, { title = it }, label = { Text("Title") }, singleLine = true, modifier = Modifier.fillMaxWidth())
        OutlinedTextField(body, { body = it }, label = { Text("Prompt") }, minLines = 3, modifier = Modifier.fillMaxWidth())
        Text("Project")
        ChoiceChips(listOf<ProjectSummary?>(null) + projects, projects.firstOrNull { it.project.id == projectId }, {
            it?.project?.name
                ?: "None"
        }) {
            projectId = it?.project?.id
        }
        Button(onClick = { onSubmit(title, body, projectId) }, enabled = title.isNotBlank()) { Text("Add prompt") }
    }
}

@Composable
fun MediaForm(onSubmit: (type: MediaType, title: String, status: MediaStatus, total: Int?) -> Unit) {
    var title by rememberSaveable { mutableStateOf("") }
    var type by rememberSaveable { mutableStateOf(MediaType.Anime) }
    var status by rememberSaveable { mutableStateOf(MediaStatus.Planned) }
    var total by rememberSaveable { mutableStateOf("") }
    val totalValue = total.trim().takeIf { it.isNotEmpty() }?.toIntOrNull()
    val totalValid = total.isBlank() || totalValue != null

    FormColumn {
        OutlinedTextField(title, { title = it }, label = { Text("Title") }, singleLine = true, modifier = Modifier.fillMaxWidth())
        ChoiceChips(MediaType.entries, type, { it.label }) { type = it }
        ChoiceChips(MediaStatus.entries, status, { it.label }) { status = it }
        OutlinedTextField(
            value = total,
            onValueChange = { total = it },
            label = { Text("Total episodes, chapters or hours (optional)") },
            singleLine = true,
            isError = !totalValid,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            modifier = Modifier.fillMaxWidth(),
        )
        Button(onClick = { onSubmit(type, title, status, totalValue) }, enabled = title.isNotBlank() && totalValid) {
            Text("Add to library")
        }
    }
}
