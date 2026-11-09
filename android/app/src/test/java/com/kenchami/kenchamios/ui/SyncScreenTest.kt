package com.kenchami.kenchamios.ui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.sync.DesktopLink
import com.kenchami.kenchamios.sync.SyncSnapshot
import com.kenchami.kenchamios.ui.sync.SyncContent
import com.kenchami.kenchamios.ui.sync.SyncProgress
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class SyncScreenTest {
    @get:Rule
    val compose = createComposeRule()

    private val link = DesktopLink("192.168.1.20", 47821, "ab".repeat(32), "desktop")
    private val now = 10 * 60_000L

    private fun show(
        snapshot: SyncSnapshot,
        progress: SyncProgress = SyncProgress(),
        onSyncNow: () -> Unit = {},
        onPair: (String) -> Unit = {},
        onUnpair: () -> Unit = {},
    ) = compose.setContent {
        SyncContent(snapshot, progress, now, onSyncNow, onPair, {}, onUnpair, PaddingValues(0.dp))
    }

    @Test
    fun anUnpairedPhoneOffersPairingOnly() {
        show(SyncSnapshot(null, null, null))

        compose.onNodeWithText("Scan QR code").assertIsDisplayed()
        compose.onNodeWithText("Not paired", substring = true).assertIsDisplayed()
        compose.onNodeWithText("Sync now").assertDoesNotExist()
    }

    @Test
    fun aPairedPhoneShowsTheLastSyncAndSyncsOnDemand() {
        var syncs = 0
        show(SyncSnapshot(link, now - 5 * 60_000L, null), onSyncNow = { syncs++ })

        compose.onNodeWithText("Paired with 192.168.1.20:47821").assertIsDisplayed()
        compose.onNodeWithText("Last sync 5m ago").assertIsDisplayed()
        compose.onNodeWithText("Sync now").performClick()

        assertEquals(1, syncs)
    }

    @Test
    fun theLastErrorAndTheResultAreShown() {
        show(SyncSnapshot(link, null, "The desktop cannot be reached."), SyncProgress(message = "Pair again."))

        compose.onNodeWithText("Never synced").assertIsDisplayed()
        compose.onNodeWithText("Last problem: The desktop cannot be reached.").assertIsDisplayed()
        compose.onNodeWithText("Pair again.").assertIsDisplayed()
    }

    @Test
    fun syncNowIsDisabledWhileWorking() {
        show(SyncSnapshot(link, null, null), SyncProgress(busy = true))

        compose.onNodeWithText("Working...").assertIsNotEnabled()
    }

    @Test
    fun pastedTextIsHandedToThePairing() {
        var pasted = ""
        show(SyncSnapshot(null, null, null), onPair = { pasted = it })

        compose.onNodeWithText("Or paste the pairing text").performTextInput("{\"v\":1}")
        compose.onNodeWithText("Pair with pasted text").performClick()

        assertEquals("{\"v\":1}", pasted)
    }

    @Test
    fun unpairingAsksFirst() {
        var unpaired = false
        show(SyncSnapshot(link, null, null), onUnpair = { unpaired = true })

        compose.onNodeWithText("Unpair").performClick()
        compose.onNodeWithText("This phone stops syncing. Your data stays on both devices.").assertIsDisplayed()
        compose.onAllNodesWithText("Unpair")[1].performClick()

        assertEquals(true, unpaired)
    }
}
