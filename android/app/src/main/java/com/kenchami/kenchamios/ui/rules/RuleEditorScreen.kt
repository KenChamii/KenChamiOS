package com.kenchami.kenchamios.ui.rules

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
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
import com.kenchami.kenchamios.ui.common.AppScreen
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
        OutlinedTextField(form.name, { v -> onChange { it.copy(name = v) } }, label = { Text("Name") }, singleLine = true)
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Switch(checked = form.enabled, onCheckedChange = { v -> onChange { it.copy(enabled = v) } })
            Text("Enabled")
        }
        SectionTitle("When")
        Choices(TriggerKind.entries, form.trigger, { it.label }) { v -> onChange { it.copy(trigger = v) } }
        when (form.trigger) {
            TriggerKind.Place -> PlaceFields(form, onChange)
            TriggerKind.Time -> TimeFields(form, onChange)
            TriggerKind.Wifi -> WifiFields(form, onChange)
        }
        SectionTitle("Then")
        Choices(ActionKind.entries, form.action, { it.label }) { v -> onChange { it.copy(action = v) } }
        ActionFields(form, onChange)
        message?.let { Text(it, color = MaterialTheme.colorScheme.error, modifier = Modifier.testTag("rule-error")) }
        Button(onClick = onSave) { Text("Save") }
        if (!isNew) OutlinedButton(onClick = { confirmDelete = true }) { Text("Delete") }
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
    Choices(PlaceTransition.entries, form.placeTransition, { if (it == PlaceTransition.Leave) "Leave" else "Arrive" }) { v ->
        onChange { it.copy(placeTransition = v) }
    }
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
            FilterChip(
                selected = day in form.days,
                onClick = { onChange { it.copy(days = if (day in it.days) it.days - day else it.days + day) } },
                label = { Text(dayLabel(day).take(2)) },
            )
        }
    }
}

@Composable
private fun WifiFields(form: RuleForm, onChange: ((RuleForm) -> RuleForm) -> Unit) {
    Choices(WifiTransition.entries, form.wifiTransition, { if (it == WifiTransition.Connect) "Join Wi-Fi" else "Leave Wi-Fi" }) { v ->
        onChange { it.copy(wifiTransition = v) }
    }
    Text("Any Wi-Fi network counts. The network name is not read.", style = MaterialTheme.typography.bodySmall)
}

@Composable
private fun ActionFields(form: RuleForm, onChange: ((RuleForm) -> RuleForm) -> Unit) {
    when (form.action) {
        ActionKind.ShowTasks -> Text("A notification lists the tasks due today or overdue.", style = MaterialTheme.typography.bodySmall)
        ActionKind.Remind -> OutlinedTextField(form.text, { v -> onChange { it.copy(text = v) } }, label = { Text("Reminder text") })
        ActionKind.Checklist -> OutlinedTextField(
            value = form.items,
            onValueChange = { v -> onChange { it.copy(items = v) } },
            label = { Text("Items, one per line") },
            minLines = 3,
        )
    }
}

@Composable
private fun NumberField(label: String, value: String, decimal: Boolean, modifier: Modifier = Modifier, onValue: (String) -> Unit) {
    OutlinedTextField(
        value = value,
        onValueChange = onValue,
        label = { Text(label) },
        singleLine = true,
        keyboardOptions = KeyboardOptions(keyboardType = if (decimal) KeyboardType.Decimal else KeyboardType.Number),
        modifier = modifier,
    )
}

@Composable
private fun <T> Choices(options: List<T>, selected: T, label: (T) -> String, onSelect: (T) -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        options.forEach { option ->
            FilterChip(selected = option == selected, onClick = { onSelect(option) }, label = { Text(label(option)) })
        }
    }
}
