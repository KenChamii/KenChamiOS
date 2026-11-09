package com.kenchami.kenchamios.widget

import com.kenchami.kenchamios.data.repo.MediaRepository
import com.kenchami.kenchamios.data.repo.TaskRepository
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import java.time.Clock

/** Widgets are not Hilt components, so they reach the app graph through this. */
@EntryPoint
@InstallIn(SingletonComponent::class)
interface WidgetEntryPoint {
    fun tasks(): TaskRepository

    fun media(): MediaRepository

    fun clock(): Clock
}
