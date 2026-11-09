package com.kenchami.kenchamios.sync

import org.json.JSONException
import org.json.JSONObject

private val CODE_PATTERN = Regex("^[0-9A-HJKMNP-TV-Z]{10}$")
private val HOST_PATTERN = Regex("^[A-Za-z0-9.-]{1,64}$")
private const val MAX_PORT = 65_535
private val EXPECTED_KEYS = setOf("v", "host", "port", "fp", "code")

/** The text of the QR code was not a pairing offer from KenChamiOS. */
class InvalidPairingException : Exception("This is not a KenChamiOS pairing code.")

/** Reads the QR text (or the pasted text) strictly; anything unexpected is refused. */
fun parsePairingPayload(text: String): PairingPayload {
    try {
        val json = JSONObject(text.trim())
        val keys = json.keys().asSequence().toSet()
        val port = json.opt("port") as? Int ?: throw InvalidPairingException()
        val host = json.opt("host") as? String ?: throw InvalidPairingException()
        val fingerprint = json.opt("fp") as? String ?: throw InvalidPairingException()
        val code = json.opt("code") as? String ?: throw InvalidPairingException()
        val valid = keys == EXPECTED_KEYS &&
            json.opt("v") == 1 &&
            HOST_PATTERN.matches(host) &&
            port in 1..MAX_PORT &&
            isValidFingerprint(fingerprint) &&
            CODE_PATTERN.matches(code)
        if (!valid) throw InvalidPairingException()
        return PairingPayload(host, port, fingerprint, code)
    } catch (_: JSONException) {
        throw InvalidPairingException()
    }
}
