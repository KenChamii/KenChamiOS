package com.kenchami.kenchamios.ui.common

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.AddCircle
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Create
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material.icons.filled.Face
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.R
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Stage
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.Tint

val Area.icon: ImageVector
    @Composable
    get() = when (this) {
        Area.Projects -> Icons.AutoMirrored.Filled.List
        Area.Media -> Icons.Filled.PlayArrow
        Area.Focus -> ImageVector.vectorResource(R.drawable.ic_timer)
        Area.Workflow -> Icons.Filled.Create
    }

val Stage.icon: ImageVector
    get() = when (this) {
        Stage.Idea -> Icons.Filled.AddCircle
        Stage.Plan -> Icons.Filled.DateRange
        Stage.Build -> Icons.Filled.Build
        Stage.Review -> Icons.Filled.Search
        Stage.Done -> Icons.Filled.CheckCircle
    }

// the core icon set has no book, tv or disc, so these are the closest fits
val MediaType.icon: ImageVector
    get() = when (this) {
        MediaType.Anime -> Icons.Filled.Face
        MediaType.Manga -> Icons.AutoMirrored.Filled.List
        MediaType.Game -> Icons.Filled.Star
        MediaType.Movie -> Icons.Filled.PlayArrow
        MediaType.Show -> Icons.Filled.DateRange
        MediaType.Album -> Icons.Filled.Favorite
        MediaType.Artist -> Icons.Filled.Person
    }

/** An icon on a soft tinted square, the mark for areas, stages and media types. */
@Composable
fun TintedIcon(icon: ImageVector, tint: Tint, modifier: Modifier = Modifier, size: Dp = 40.dp, contentDescription: String? = null) {
    Box(
        modifier.size(size).clip(MaterialTheme.shapes.small).background(tint.container),
        contentAlignment = Alignment.Center,
    ) {
        Icon(icon, contentDescription, tint = tint.content, modifier = Modifier.size(size / 2))
    }
}

/** The tint for things that belong to no area. */
val neutralTint: Tint
    @Composable
    get() = Tint(MaterialTheme.colorScheme.surfaceContainerHigh, MaterialTheme.colorScheme.onSurfaceVariant)
