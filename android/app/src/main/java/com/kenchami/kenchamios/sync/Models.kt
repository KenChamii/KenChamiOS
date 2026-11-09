package com.kenchami.kenchamios.sync

import org.json.JSONObject

/** Most changes in one push or pull page, the same limit as the desktop. */
const val MAX_CHANGES = 500

/** One row of a synced table as it travels: the table name and the row as in `tables.schema.json`. */
data class Change(val table: String, val row: JSONObject) {
    val id: String get() = row.getString("id")
    val updatedAt: Long get() = row.getLong("updated_at")
}

data class ChangePage(val changes: List<Change>, val next: Long, val hasMore: Boolean)

data class ApplyOutcome(val applied: Int, val ignored: Int)

/** Content of the pairing QR code. */
data class PairingPayload(val host: String, val port: Int, val fingerprint: String, val code: String)

/** What the phone needs to talk to the paired desktop; the credential is kept apart. */
data class DesktopLink(val host: String, val port: Int, val fingerprint: String, val desktopDeviceId: String)

/** The sync data was refused before anything was stored. */
class RejectedException(message: String) : Exception(message)
