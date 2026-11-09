package com.kenchami.kenchamios.sync

import com.kenchami.kenchamios.data.Stamper
import javax.inject.Inject
import javax.inject.Named
import javax.inject.Singleton

/** Pairs this phone with the desktop shown in a QR code. */
@Singleton
class PairingService @Inject constructor(
    private val state: SyncState,
    private val secrets: SecretStore,
    private val apis: SyncApiFactory,
    private val stamper: Stamper,
    @Named("deviceName") private val deviceName: String,
) {
    /** The certificate in the QR code is pinned before the one-time code is sent. */
    suspend fun pair(payload: PairingPayload) {
        val api = apis.create(payload.host, payload.port, payload.fingerprint)
        val result = api.pair(payload.code, stamper.deviceId(), deviceName)
        secrets.set(result.token)
        state.savePairing(DesktopLink(payload.host, payload.port, payload.fingerprint, result.desktopDeviceId))
    }

    /** Forgets the desktop; it keeps listing this phone until the pairing is revoked there. */
    suspend fun unpair() {
        secrets.clear()
        state.clear()
    }
}
