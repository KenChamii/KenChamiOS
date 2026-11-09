package com.kenchami.kenchamios.context.platform

import com.kenchami.kenchamios.context.ContextCoordinator
import com.kenchami.kenchamios.context.ContextRunner
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent

/** Workers are not Hilt components, so they reach the rules through this. */
@EntryPoint
@InstallIn(SingletonComponent::class)
interface ContextEntryPoint {
    fun contextRunner(): ContextRunner

    fun coordinator(): ContextCoordinator
}
