package com.kenchami.kenchamios.ui.common

import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.Stage
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.model.TrashKind

val Stage.label: String
    get() = when (this) {
        Stage.Idea -> "Idea"
        Stage.Plan -> "Plan"
        Stage.Build -> "Build"
        Stage.Review -> "Review"
        Stage.Done -> "Done"
    }

val TaskStatus.label: String
    get() = when (this) {
        TaskStatus.Todo -> "To do"
        TaskStatus.Doing -> "Doing"
        TaskStatus.Review -> "Review"
        TaskStatus.Done -> "Done"
    }

val Priority.label: String
    get() = when (this) {
        Priority.Low -> "Low"
        Priority.Medium -> "Medium"
        Priority.High -> "High"
    }

val MediaType.label: String
    get() = when (this) {
        MediaType.Anime -> "Anime"
        MediaType.Manga -> "Manga"
        MediaType.Game -> "Game"
        MediaType.Movie -> "Movie"
        MediaType.Show -> "Show"
        MediaType.Album -> "Album"
        MediaType.Artist -> "Artist"
    }

val MediaStatus.label: String
    get() = when (this) {
        MediaStatus.InProgress -> "In progress"
        MediaStatus.Planned -> "Planned"
        MediaStatus.Completed -> "Completed"
        MediaStatus.OnHold -> "On hold"
        MediaStatus.Dropped -> "Dropped"
    }

val TrashKind.label: String
    get() = when (this) {
        TrashKind.Project -> "Project"
        TrashKind.Task -> "Task"
        TrashKind.Prompt -> "Prompt"
        TrashKind.SavePoint -> "Save point"
        TrashKind.Media -> "Library entry"
        TrashKind.Habit -> "Habit"
    }
