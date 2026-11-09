package com.kenchami.kenchamios.sync

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class PairingPayloadTest {
    private fun valid() = SyncFixtures.obj("pair-payload")

    @Test
    fun theSharedPayloadIsParsed() {
        val payload = parsePairingPayload(SyncFixtures.text("pair-payload"))

        assertEquals(PairingPayload("192.168.1.20", 47821, "ab".repeat(32), "7K3M9QX2TV"), payload)
    }

    @Test
    fun surroundingWhitespaceFromPastingIsIgnored() {
        assertEquals(47821, parsePairingPayload("\n  ${SyncFixtures.text("pair-payload")}  \n").port)
    }

    @Test
    fun anythingUnexpectedIsRefused() {
        val broken = listOf(
            "not json",
            "[]",
            valid().put("v", 2).toString(),
            valid().put("extra", 1).toString(),
            valid().put("port", 0).toString(),
            valid().put("port", 70_000).toString(),
            valid().put("port", "47821").toString(),
            valid().put("host", "a b").toString(),
            valid().put("fp", "AB".repeat(32)).toString(),
            valid().put("code", "7K3M9QX2TI").toString(),
            valid().put("code", "short").toString(),
            JSONObject().put("v", 1).toString(),
        )
        for (text in broken) {
            assertThrows(text, InvalidPairingException::class.java) { parsePairingPayload(text) }
        }
    }
}
