package com.kenchami.kenchamios.ui.nav

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.Create
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.ui.graphics.vector.ImageVector

/** The four screens of the bottom bar. */
enum class Destination(val route: String, val label: String, val icon: ImageVector) {
    Today("today", "Today", Icons.Filled.Home),
    Projects("projects", "Projects", Icons.AutoMirrored.Filled.List),
    Prompts("prompts", "Prompts", Icons.Filled.Create),
    Media("media", "Media", Icons.Filled.PlayArrow),
}

object Routes {
    const val PROJECT = "project/{id}"
    const val ENTRY = "media/{entryId}"
    const val SYNC = "sync"
    const val RULES = "rules"
    const val RULE = "rules/{id}"
    const val RULES_SETTINGS = "rules-settings"

    fun project(id: String) = "project/$id"

    fun entry(id: String) = "media/$id"

    fun rule(id: String) = "rules/$id"
}
