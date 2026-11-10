package com.kenchami.kenchamios.ui.notes

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalLocale
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.today.dateLabel
import java.time.LocalDate

private const val EDITOR_LINES = 10

/** What the notes screen reports. */
class NotesActions(val onSelect: (LocalDate) -> Unit, val onMonth: (Long) -> Unit, val onEdit: (String) -> Unit)

@Composable
fun DailyNotesScreen(onBack: () -> Unit, viewModel: DailyNotesViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val draft by viewModel.draft.collectAsStateWithLifecycle()
    AppScreen("Daily notes", onBack = onBack, area = Area.Focus) { padding ->
        state?.let { DailyNotesContent(it, draft, NotesActions(viewModel::select, viewModel::showMonth, viewModel::edit), padding) }
    }
}

@Composable
fun DailyNotesContent(state: DailyNotesState, draft: String?, actions: NotesActions, padding: PaddingValues) {
    Column(
        Modifier.padding(padding).verticalScroll(rememberScrollState()).padding(horizontal = 16.dp).testTag("daily-notes"),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        MonthCalendar(state.month, state.selected, state.today, state.noteDays, actions.onSelect, actions.onMonth)
        Text(dateLabel(state.selected, LocalLocale.current.platformLocale), style = MaterialTheme.typography.titleMedium)
        if (draft != null) {
            AppTextField(draft, actions.onEdit, "Note", Modifier.fillMaxWidth(), minLines = EDITOR_LINES)
            Text(
                "Saved as you type. Markdown shows formatted on the desktop.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
