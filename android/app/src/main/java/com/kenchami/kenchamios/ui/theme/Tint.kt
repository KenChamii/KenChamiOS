package com.kenchami.kenchamios.ui.theme

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color

/** The parts of the app that carry their own tint; surfaces stay neutral. */
enum class Area { Projects, Media, Focus, Workflow }

/** A soft background with a strong foreground on top of it. */
@Immutable
data class Tint(val container: Color, val content: Color)

val LightTints = mapOf(
    Area.Projects to Tint(Color(0xFFE2EAFF), Color(0xFF2B57D6)),
    Area.Media to Tint(Color(0xFFEDE4FF), Color(0xFF6D3FD4)),
    Area.Focus to Tint(Color(0xFFFFE9D9), Color(0xFFC0530C)),
    Area.Workflow to Tint(Color(0xFFDDF2E4), Color(0xFF1D7D45)),
)

val DarkTints = mapOf(
    Area.Projects to Tint(Color(0xFF1F2B4D), Color(0xFFA3BBFF)),
    Area.Media to Tint(Color(0xFF2D2347), Color(0xFFC9B1FF)),
    Area.Focus to Tint(Color(0xFF44291A), Color(0xFFFFB685)),
    Area.Workflow to Tint(Color(0xFF1B3526), Color(0xFF8ED8A8)),
)

val LocalTints = staticCompositionLocalOf { LightTints }

val Area.tint: Tint
    @Composable
    @ReadOnlyComposable
    get() = LocalTints.current.getValue(this)
