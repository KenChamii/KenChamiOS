package com.kenchami.kenchamios.ui.theme

import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color

private val Accent = Color(0xFF3D6DF2)

val LightColors = lightColorScheme(
    primary = Accent,
    onPrimary = Color.White,
    primaryContainer = Color(0xFFE2E2E2),
    onPrimaryContainer = Color(0xFF1A1A1A),
    secondary = Color(0xFF5E5E5E),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFE6E6E6),
    onSecondaryContainer = Color(0xFF1A1A1A),
    tertiary = Color(0xFF5E5E5E),
    background = Color(0xFFF0F0F0),
    onBackground = Color(0xFF1A1A1A),
    surface = Color(0xFFF0F0F0),
    onSurface = Color(0xFF1A1A1A),
    surfaceVariant = Color(0xFFE6E6E6),
    onSurfaceVariant = Color(0xFF757575),
    surfaceContainerLowest = Color.White,
    surfaceContainerLow = Color(0xFFF5F5F5),
    surfaceContainer = Color(0xFFEAEAEA),
    surfaceContainerHigh = Color(0xFFE4E4E4),
    surfaceContainerHighest = Color(0xFFDEDEDE),
    outline = Color(0xFFB0B0B0),
    outlineVariant = Color(0xFFD8D8D8),
    error = Color(0xFFC62828),
)

val DarkColors = darkColorScheme(
    primary = Color(0xFF8FAAFF),
    onPrimary = Color(0xFF0B1B4D),
    primaryContainer = Color(0xFF3A3A3A),
    onPrimaryContainer = Color(0xFFF0F0F0),
    secondary = Color(0xFFB0B0B0),
    onSecondary = Color(0xFF1A1A1A),
    secondaryContainer = Color(0xFF363636),
    onSecondaryContainer = Color(0xFFF0F0F0),
    tertiary = Color(0xFFB0B0B0),
    background = Color(0xFF262626),
    onBackground = Color(0xFFF0F0F0),
    surface = Color(0xFF262626),
    onSurface = Color(0xFFF0F0F0),
    surfaceVariant = Color(0xFF333333),
    onSurfaceVariant = Color(0xFF9A9A9A),
    surfaceContainerLowest = Color(0xFF1C1C1C),
    surfaceContainerLow = Color(0xFF2B2B2B),
    surfaceContainer = Color(0xFF303030),
    surfaceContainerHigh = Color(0xFF363636),
    surfaceContainerHighest = Color(0xFF3D3D3D),
    outline = Color(0xFF6A6A6A),
    outlineVariant = Color(0xFF404040),
    error = Color(0xFFFF8A80),
)
