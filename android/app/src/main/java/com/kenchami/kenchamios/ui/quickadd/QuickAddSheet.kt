package com.kenchami.kenchamios.ui.quickadd

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.PrimaryTabRow
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle

private val TABS = listOf("Task", "Idea", "Prompt", "Media")

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QuickAddSheet(onDismiss: () -> Unit, viewModel: QuickAddViewModel = hiltViewModel()) {
    val projects by viewModel.projectChoices.collectAsStateWithLifecycle()
    var tab by rememberSaveable { mutableIntStateOf(0) }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
    ) {
        Column(Modifier.navigationBarsPadding().padding(bottom = 16.dp).testTag("quick-add")) {
            PrimaryTabRow(selectedTabIndex = tab) {
                TABS.forEachIndexed { index, title ->
                    Tab(selected = tab == index, onClick = { tab = index }, text = { Text(title) })
                }
            }
            when (tab) {
                0 -> TaskForm(projects, viewModel.clock) { projectId, title, priority, dueAt ->
                    viewModel.addTask(projectId, title, priority, dueAt)
                    onDismiss()
                }
                1 -> IdeaForm { name, description ->
                    viewModel.addIdea(name, description)
                    onDismiss()
                }
                2 -> PromptForm(projects) { title, body, projectId ->
                    viewModel.addPrompt(title, body, projectId)
                    onDismiss()
                }
                else -> MediaForm { type, title, status, total ->
                    viewModel.addMedia(type, title, status, total)
                    onDismiss()
                }
            }
        }
    }
}
