package com.kenchami.kenchamios.widget

import android.content.Context
import androidx.glance.GlanceId
import androidx.glance.action.ActionParameters
import androidx.glance.appwidget.action.ActionCallback
import dagger.hilt.android.EntryPointAccessors

val EntryIdKey = ActionParameters.Key<String>("entry_id")

/** The +1 button of a continue row. */
class AddOneAction : ActionCallback {
    override suspend fun onAction(context: Context, glanceId: GlanceId, parameters: ActionParameters) {
        val entryId = parameters[EntryIdKey] ?: return
        EntryPointAccessors.fromApplication(context.applicationContext, WidgetEntryPoint::class.java)
            .media()
            .increment(entryId)
    }
}
