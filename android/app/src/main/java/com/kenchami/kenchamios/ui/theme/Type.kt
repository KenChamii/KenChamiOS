package com.kenchami.kenchamios.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.kenchami.kenchamios.R

private fun interAt(weight: FontWeight) =
    Font(R.font.inter_variable, weight, variationSettings = FontVariation.Settings(weight, FontStyle.Normal))

// one variable file serves every weight of the scale
val Inter = FontFamily(
    interAt(FontWeight.Normal),
    interAt(FontWeight.Medium),
    interAt(FontWeight.SemiBold),
    interAt(FontWeight.Bold),
)

private val base = Typography()

private fun TextStyle.inter(weight: FontWeight, tracking: Double) =
    copy(fontFamily = Inter, fontWeight = weight, letterSpacing = tracking.em)

val AppTypography = Typography(
    displayLarge = base.displayLarge.inter(FontWeight.Bold, -0.04).copy(fontSize = 88.sp, lineHeight = 88.sp),
    displayMedium = base.displayMedium.inter(FontWeight.Bold, -0.035).copy(fontSize = 64.sp, lineHeight = 64.sp),
    displaySmall = base.displaySmall.inter(FontWeight.Bold, -0.025).copy(fontSize = 44.sp, lineHeight = 48.sp),
    headlineLarge = base.headlineLarge.inter(FontWeight.Bold, -0.02),
    headlineMedium = base.headlineMedium.inter(FontWeight.Bold, -0.015),
    headlineSmall = base.headlineSmall.inter(FontWeight.Bold, -0.01),
    titleLarge = base.titleLarge.inter(FontWeight.Bold, -0.01),
    titleMedium = base.titleMedium.inter(FontWeight.SemiBold, 0.0),
    titleSmall = base.titleSmall.inter(FontWeight.SemiBold, 0.0),
    bodyLarge = base.bodyLarge.inter(FontWeight.Normal, 0.0),
    bodyMedium = base.bodyMedium.inter(FontWeight.Normal, 0.0),
    bodySmall = base.bodySmall.inter(FontWeight.Normal, 0.0),
    labelLarge = base.labelLarge.inter(FontWeight.SemiBold, 0.0),
    labelMedium = base.labelMedium.inter(FontWeight.Medium, 0.02).copy(fontSize = 12.sp, lineHeight = 16.sp),
    labelSmall = base.labelSmall.inter(FontWeight.Medium, 0.02),
)
