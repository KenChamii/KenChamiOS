package com.kenchami.kenchamios.sync

import okhttp3.OkHttpClient
import okhttp3.Protocol
import java.security.MessageDigest
import java.security.SecureRandom
import java.security.cert.CertificateException
import java.security.cert.X509Certificate
import java.util.concurrent.TimeUnit
import javax.net.ssl.SSLContext
import javax.net.ssl.X509TrustManager

private const val FINGERPRINT_HEX_LENGTH = 64
private const val TIMEOUT_S = 15L

/** The server certificate does not have the hash from the pairing QR code. */
class PinMismatchException : CertificateException("the desktop certificate does not match the pairing")

fun isValidFingerprint(value: String) = value.length == FINGERPRINT_HEX_LENGTH && value.all { it in '0'..'9' || it in 'a'..'f' }

/** Trusts exactly one certificate, identified by its SHA-256 hash, and no certificate authority. */
class PinnedTrustManager(fingerprintHex: String) : X509TrustManager {
    private val pinned: ByteArray

    init {
        require(isValidFingerprint(fingerprintHex)) { "the fingerprint is not 64 lowercase hex characters" }
        pinned = ByteArray(FINGERPRINT_HEX_LENGTH / 2) { fingerprintHex.substring(it * 2, it * 2 + 2).toInt(16).toByte() }
    }

    override fun checkServerTrusted(chain: Array<out X509Certificate>?, authType: String?) {
        val leaf = chain?.firstOrNull() ?: throw CertificateException("the server sent no certificate")
        leaf.checkValidity()
        val actual = MessageDigest.getInstance("SHA-256").digest(leaf.encoded)
        // constant time, so the comparison reveals nothing about the pin
        if (!MessageDigest.isEqual(actual, pinned)) throw PinMismatchException()
    }

    override fun checkClientTrusted(chain: Array<out X509Certificate>?, authType: String?): Unit =
        throw CertificateException("this app does not accept client certificates")

    override fun getAcceptedIssuers(): Array<X509Certificate> = emptyArray()
}

object PinnedTls {
    /** An HTTP client that only talks to the server whose certificate has the given hash. */
    fun client(fingerprintHex: String): OkHttpClient {
        val trustManager = PinnedTrustManager(fingerprintHex)
        val context = SSLContext.getInstance("TLS").apply { init(null, arrayOf(trustManager), SecureRandom()) }
        return OkHttpClient.Builder()
            .sslSocketFactory(context.socketFactory, trustManager)
            // the address is a LAN IP that changes, so the certificate hash is what identifies the desktop
            .hostnameVerifier { _, _ -> true }
            .protocols(listOf(Protocol.HTTP_1_1))
            .retryOnConnectionFailure(false)
            .connectTimeout(TIMEOUT_S, TimeUnit.SECONDS)
            .readTimeout(TIMEOUT_S, TimeUnit.SECONDS)
            .writeTimeout(TIMEOUT_S, TimeUnit.SECONDS)
            .build()
    }
}
