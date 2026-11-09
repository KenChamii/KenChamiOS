package com.kenchami.kenchamios.ui.common

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextDecoration

// compose scales tweens by the system animator setting, so reduced motion shortens or skips them
const val MOTION_MS = 200

/** Rows fade in and out and slide into place when the list changes. */
fun Modifier.listMotion(scope: LazyItemScope): Modifier =
    with(scope) { this@listMotion.animateItem(fadeInSpec = tween(MOTION_MS), fadeOutSpec = tween(MOTION_MS)) }

/** A task title that fades and strikes through once it is ticked. */
@Composable
fun TaskTitle(title: String, done: Boolean) {
    val color by animateColorAsState(
        if (done) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface,
        tween(MOTION_MS),
        label = "task title",
    )
    Text(title, color = color, textDecoration = if (done) TextDecoration.LineThrough else null)
}
