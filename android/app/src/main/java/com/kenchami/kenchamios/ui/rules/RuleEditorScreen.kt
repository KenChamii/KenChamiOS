package com.kenchami.kenchamios.ui.rules

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.context.PlaceTransition
import com.kenchami.kenchamios.context.WifiTransition
import com.kenchami.kenchamios.ui.common.AppChip
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.common.ChoiceChips
import com.kenchami.kenchamios.ui.common.ConfirmDialog
import com.kenchami.kenchamios.ui.common.SectionTitle
import java.time.DayOfWeek

@Composable
fun RuleEditorScreen(onBack: () -> Unit, viewModel: RuleEditorViewModel = hiltViewModel()) {
    val form by viewModel.form.collectAsStateWithLifecycle()
    val message by viewModel.message.collectAsStateWithLifecycle()
    AppScreen(if (viewModel.isNew) "New rule" else "Edit rule", onBack = onBack) { padding ->
        form?.let {
            RuleEditorContent(
                form = it,
                message = message,
                isNew = viewModel.isNew,
                onChange = viewModel::edit,
                onSave = { viewModel.save(onBack) },
                onDelete = { viewModel.delete(onBack) },
                padding = padding,
            )
        }
    }
}

@Composable
fun RuleEditorContent(
    form: RuleForm,
    message: String?,
    isNew: Boolean,
    onChange: ((RuleForm) -> RuleForm) -> Unit,
    onSave: () -> Unit,
    onDelete: () -> Unit,
    padding: PaddingValues,
) {
    var confirmDelete by rememberSaveable { mutableStateOf(false) }
    Column(
        Modifier.padding(padding).padding(horizontal = 16.dp).verticalScroll(rememberScrollState()).testTag("rule-editor"),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        AppTextField(form.name, { v -> onChange { it.copy(name = v) } }, "Name", Modifier.fillMaxWidth(), singleLine = true)
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Switch(checked = form.enabled, onCheckedChange = { v -> onChange { it.copy(enabled = v) } })
            Text("Enabled")
        }
        SectionTitle("When")
        ChoiceChips(TriggerKind.entries, form.trigger, { it.label }, onSelect = { v -> onChange { it.copy(trigger = v) } })
        when (form.trigger) {
            TriggerKind.Place -> PlaceFields(form, onChange)
            TriggerKind.Time -> TimeFields(form, onChange)
            TriggerKind.Wifi -> WifiFields(form, onChange)
        }
        SectionTitle("Then")
        ChoiceChips(ActionKind.entries, form.action, { it.label }, onSelect = { v -> onChange { it.copy(action = v) } })
        ActionFields(form, onChange)
        message?.let { Text(it, color = MaterialTheme.colorScheme.error, modifier = Modifier.testTag("rule-error")) }
        Button(onClick = onSave) { Text("Save") }
        if (!isNew) FilledTonalButton(onClick = { confirmDelete = true }) { Text("Delete") }
    }
    if (confirmDelete) {
        ConfirmDialog(
            title = "Delete rule?",
            text = "The rule and its checklist progress are removed from this phone.",
            confirmLabel = "Delete",
            onConfirm = onDelete,
            onDismiss = { confirmDelete = false },
        )
    }
}

@Composable
private fun PlaceFields(form: RuleForm, onChange: ((RuleForm) -> RuleForm) -> Unit) {
    ChoiceChips(
        PlaceTransition.entries,
        form.placeTransition,
        label = { if (it == PlaceTransition.Leave) "Leave" else "Arrive" },
        onSelect = { v -> onChange { it.copy(placeTransition = v) } },
    )
    NumberField("Latitude", form.latitude, decimal = true) { v -> onChange { it.copy(latitude = v) } }
    NumberField("Longitude", form.longitude, decimal = true) { v -> onChange { it.copy(longitude = v) } }
    NumberField("Radius in metres (100 to 5000)", form.radius, decimal = false) { v -> onChange { it.copy(radius = v) } }
    Text(
        "Copy the coordinates from any map app. They stay on this phone. Place rules need location access, see Rule settings.",
        style = MaterialTheme.typography.bodySmall,
    )
}

@Composable
private fun TimeFields(form: RuleForm, onChange: ((RuleForm) -> RuleForm) -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        NumberField("Hour (0-23)", form.hour, decimal = false, Modifier.weight(1f)) { v -> onChange { it.copy(hour = v) } }
        NumberField("Minute (0-59)", form.minute, decimal = false, Modifier.weight(1f)) { v -> onChange { it.copy(minute = v) } }
    }
    Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        DayOfWeek.entries.forEach { day ->
            AppChip(
                selected = day in form.days,
                onClick = { onChange { it.copy(days = if (day in it.days) it.days - day else it.days + day) } },
                label = dayLabel(day).take(2),
            )
        }
    }
}

@Composable
private fun WifiFields(form: RuleForm, onChange: ((RuleForm) -> RuleForm) -> Unit) {
    ChoiceChips(
        WifiTransition.entries,
        form.wifiTransition,
        label = { if (it == WifiTransition.Connect) "Join Wi-Fi" else "Leave Wi-Fi" },
        onSelect = { v -> onChange { it.copy(wifiTransition = v) } },
    )
    Text("Any Wi-Fi network counts. The network name is not read.", style = MaterialTheme.typography.bodySmall)
}

@Composable
private fun ActionFields(form: RuleForm, onChange: ((RuleForm) -> RuleForm) -> Unit) {
    when (form.action) {
        ActionKind.ShowTasks -> Text("A notification lists the tasks due today or overdue.", style = MaterialTheme.typography.bodySmall)
        ActionKind.Remind -> AppTextField(form.text, { v -> onChange { it.copy(text = v) } }, "Reminder text", Modifier.fillMaxWidth())
        ActionKind.Checklist -> AppTextField(
            value = form.items,
            onValueChange = { v -> onChange { it.copy(items = v) } },
            label = "Items, one per line",
            modifier = Modifier.fillMaxWidth(),
            minLines = 3,
        )
    }
}

@Composable
private fun NumberField(label: String, value: String, decimal: Boolean, modifier: Modifier = Modifier, onValue: (String) -> Unit) {
    AppTextField(
        value = value,
        onValueChange = onValue,
        label = label,
        singleLine = true,
        keyboardOptions = KeyboardOptions(keyboardType = if (decimal) KeyboardType.Decimal else KeyboardType.Number),
        modifier = modifier,
    )
}
