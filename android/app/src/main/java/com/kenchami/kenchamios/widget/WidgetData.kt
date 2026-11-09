package com.kenchami.kenchamios.widget

import com.kenchami.kenchamios.data.db.DueTask
import com.kenchami.kenchamios.data.db.LibraryEntry

/** What the widget shows: tasks due today or overdue, and what to continue. */
data class WidgetData(val due: List<DueTask> = emptyList(), val continuing: List<LibraryEntry> = emptyList())
