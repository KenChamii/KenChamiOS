package com.kenchami.kenchamios.ui.planner

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.data.db.PlannedTask
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.common.SectionTitle

/** What the evening review can do. */
class ReviewActions(
    val onDone: (PlannedTask) -> Unit,
    val onTomorrow: (PlannedTask) -> Unit,
    val onDrop: (PlannedTask) -> Unit,
    val onAddNote: (String) -> Unit,
    val onOpenNote: () -> Unit,
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReviewDaySheet(onDismiss: () -> Unit, onOpenNote: () -> Unit, viewModel: ReviewDayViewModel = hiltViewModel()) {
    val planned by viewModel.planned.collectAsStateWithLifecycle()
    ModalBottomSheet(onDismissRequest = onDismiss, sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)) {
        ReviewDayContent(
            planned,
            ReviewActions(viewModel::done, viewModel::tomorrow, viewModel::drop, viewModel::addNote) {
                onDismiss()
                onOpenNote()
            },
        )
    }
}

/** The evening review: settle today's plan and add a line to today's note. */
@Composable
fun ReviewDayContent(planned: List<PlannedTask>?, actions: ReviewActions) {
    var note by rememberSaveable { mutableStateOf("") }
    var added by rememberSaveable { mutableStateOf(false) }
    Column(Modifier.navigationBarsPadding().padding(bottom = 16.dp).verticalScroll(rememberScrollState()).testTag("review-day")) {
        Text("Review my day", style = MaterialTheme.typography.headlineSmall, modifier = Modifier.padding(horizontal = 20.dp))
        SectionTitle("Today's plan")
        if (planned?.isEmpty() == true) Hint("Nothing was planned for today.")
        planned.orEmpty().forEach { ReviewRow(it, actions) }
        SectionTitle("Note")
        AppTextField(
            note,
            { note = it },
            "How did today go?",
            Modifier.fillMaxWidth().padding(horizontal = 16.dp),
            minLines = 3,
        )
        Row(
            Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            FilledTonalButton(
                onClick = {
                    actions.onAddNote(note)
                    note = ""
                    added = true
                },
                enabled = note.isNotBlank(),
            ) { Text("Add to today's note") }
            if (added) Text("Added", color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        TextButton(onClick = actions.onOpenNote, modifier = Modifier.padding(horizontal = 8.dp)) { Text("Open today's note") }
    }
}

@Composable
private fun ReviewRow(task: PlannedTask, actions: ReviewActions) {
    Column(Modifier.fillMaxWidth().padding(start = 20.dp, end = 8.dp, top = 8.dp)) {
        Text(task.title, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Text(task.projectName, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (task.status == TaskStatus.Done) {
                Icon(Icons.Filled.Check, contentDescription = null)
                Text("Done", modifier = Modifier.padding(start = 4.dp, top = 8.dp, bottom = 8.dp))
            } else {
                ReviewButton("Done", "Mark ${task.title} done") { actions.onDone(task) }
                ReviewButton("Tomorrow", "Move ${task.title} to tomorrow") { actions.onTomorrow(task) }
                ReviewButton("Drop", "Drop ${task.title} from the plan") { actions.onDrop(task) }
            }
        }
    }
}

@Composable
private fun ReviewButton(label: String, description: String, onClick: () -> Unit) {
    TextButton(onClick = onClick, modifier = Modifier.semantics { contentDescription = description }) { Text(label) }
}
