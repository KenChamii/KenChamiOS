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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle

enum class QuickAddTab { Task, Idea, Prompt, Media }

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QuickAddSheet(initialTab: QuickAddTab, onDismiss: () -> Unit, viewModel: QuickAddViewModel = hiltViewModel()) {
    val projects by viewModel.projectChoices.collectAsStateWithLifecycle()
    var tab by rememberSaveable { mutableStateOf(initialTab) }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
    ) {
        Column(Modifier.navigationBarsPadding().padding(bottom = 16.dp).testTag("quick-add")) {
            PrimaryTabRow(selectedTabIndex = tab.ordinal) {
                QuickAddTab.entries.forEach { entry ->
                    Tab(selected = tab == entry, onClick = { tab = entry }, text = { Text(entry.name) })
                }
            }
            when (tab) {
                QuickAddTab.Task -> TaskForm(projects, viewModel.clock, onAddIdea = {
                    tab = QuickAddTab.Idea
                }) { projectId, title, priority, dueAt ->
                    viewModel.addTask(projectId, title, priority, dueAt)
                    onDismiss()
                }
                QuickAddTab.Idea -> IdeaForm { name, description ->
                    viewModel.addIdea(name, description)
                    onDismiss()
                }
                QuickAddTab.Prompt -> PromptForm(projects) { title, body, projectId ->
                    viewModel.addPrompt(title, body, projectId)
                    onDismiss()
                }
                QuickAddTab.Media -> MediaForm { type, title, status, total ->
                    viewModel.addMedia(type, title, status, total)
                    onDismiss()
                }
            }
        }
    }
}
