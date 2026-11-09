package com.kenchami.kenchamios.ui.common

import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import javax.inject.Inject
import javax.inject.Singleton

class UndoOffer(val message: String, val undo: suspend () -> Unit)

/** Carries "done, undo?" messages from any screen to the one snackbar of the app scaffold. */
@Singleton
class UndoBus @Inject constructor() {
    private val flow = MutableSharedFlow<UndoOffer>(extraBufferCapacity = 1, onBufferOverflow = BufferOverflow.DROP_OLDEST)
    val offers: SharedFlow<UndoOffer> = flow

    fun offer(message: String, undo: suspend () -> Unit) {
        flow.tryEmit(UndoOffer(message, undo))
    }
}
