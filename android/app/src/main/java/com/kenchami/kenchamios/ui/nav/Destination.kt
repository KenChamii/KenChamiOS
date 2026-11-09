package com.kenchami.kenchamios.ui.nav

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.Create
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.ui.graphics.vector.ImageVector
import com.kenchami.kenchamios.ui.theme.Area

/** The four screens of the bottom bar; Today belongs to no area and stays neutral. */
enum class Destination(val route: String, val label: String, val icon: ImageVector, val area: Area?) {
    Today("today", "Today", Icons.Filled.Home, null),
    Projects("projects", "Projects", Icons.AutoMirrored.Filled.List, Area.Projects),
    Prompts("prompts", "Prompts", Icons.Filled.Create, Area.Workflow),
    Media("media", "Media", Icons.Filled.PlayArrow, Area.Media),
}

object Routes {
    const val PROJECT = "project/{id}"
    const val ENTRY = "media/{entryId}"
    const val SYNC = "sync"
    const val RULES = "rules"
    const val RULE = "rules/{id}"
    const val RULES_SETTINGS = "rules-settings"
    const val INBOX = "inbox"
    const val HABITS = "habits"
    const val TRASH = "trash"

    fun project(id: String) = "project/$id"

    fun entry(id: String) = "media/$id"

    fun rule(id: String) = "rules/$id"
}
