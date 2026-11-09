package com.kenchami.kenchamios.ui.sync

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.sync.InvalidPairingException
import com.kenchami.kenchamios.sync.PairingService
import com.kenchami.kenchamios.sync.SyncFailure
import com.kenchami.kenchamios.sync.SyncResult
import com.kenchami.kenchamios.sync.SyncRunner
import com.kenchami.kenchamios.sync.SyncSnapshot
import com.kenchami.kenchamios.sync.SyncState
import com.kenchami.kenchamios.sync.parsePairingPayload
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

/** What the sync screen shows besides the stored pairing. */
data class SyncProgress(val busy: Boolean = false, val message: String? = null)

@HiltViewModel
class SyncViewModel @Inject constructor(state: SyncState, private val runner: SyncRunner, private val pairing: PairingService) :
    ViewModel() {
    val snapshot: StateFlow<SyncSnapshot> = state.observe()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), SyncSnapshot(null, null, null))

    private val _progress = MutableStateFlow(SyncProgress())
    val progress: StateFlow<SyncProgress> = _progress

    fun syncNow() = run {
        when (val result = runner.sync()) {
            is SyncResult.Done -> "Synced: ${result.sent} sent, ${result.received} received."
            SyncResult.NotPaired -> "Pair with the desktop first."
            is SyncResult.Failed -> result.message
        }
    }

    /** [text] is what the QR code or the pasted text holds. */
    fun pair(text: String) = run {
        try {
            pairing.pair(parsePairingPayload(text))
            runner.sync()
            "Paired. The first sync is done."
        } catch (error: InvalidPairingException) {
            error.message
        } catch (error: SyncFailure) {
            error.message
        }
    }

    fun unpair() = run {
        pairing.unpair()
        "Unpaired. Remove this phone on the desktop too."
    }

    fun showMessage(message: String) = _progress.update { it.copy(message = message) }

    private fun run(block: suspend () -> String?) {
        if (_progress.value.busy) return
        _progress.value = SyncProgress(busy = true)
        viewModelScope.launch {
            val message = block()
            _progress.value = SyncProgress(busy = false, message = message)
        }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
