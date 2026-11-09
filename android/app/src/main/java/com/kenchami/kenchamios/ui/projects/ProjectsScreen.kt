package com.kenchami.kenchamios.ui.projects

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ListItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.EmptyText
import com.kenchami.kenchamios.ui.common.QuickAddButton
import com.kenchami.kenchamios.ui.common.label
import com.kenchami.kenchamios.ui.common.timeAgo

@Composable
fun ProjectsScreen(onOpenProject: (String) -> Unit, onQuickAdd: () -> Unit, viewModel: ProjectsViewModel = hiltViewModel()) {
    val query by viewModel.query.collectAsStateWithLifecycle()
    val projects by viewModel.projects.collectAsStateWithLifecycle()
    AppScreen("Projects", actions = { QuickAddButton(onQuickAdd) }) { padding ->
        ProjectsContent(query, viewModel::setQuery, projects, System.currentTimeMillis(), onOpenProject, padding)
    }
}

@Composable
fun ProjectsContent(
    query: String,
    onQueryChange: (String) -> Unit,
    projects: List<ProjectSummary>,
    now: Long,
    onOpenProject: (String) -> Unit,
    padding: PaddingValues,
) {
    Column(Modifier.padding(padding)) {
        OutlinedTextField(
            value = query,
            onValueChange = onQueryChange,
            label = { Text("Search") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
        )
        if (projects.isEmpty()) EmptyText("No projects yet. Use + to add an idea.")
        LazyColumn(Modifier.testTag("projects")) {
            items(projects, key = { it.project.id }) { summary ->
                ListItem(
                    headlineContent = { Text(summary.project.name) },
                    supportingContent = {
                        Text(
                            "${summary.project.stage.label} - ${summary.tasksDone}/${summary.tasksTotal} tasks - ${timeAgo(
                                summary.lastActivityAt,
                                now,
                            )}",
                        )
                    },
                    modifier = Modifier.clickable { onOpenProject(summary.project.id) },
                )
            }
        }
    }
}
