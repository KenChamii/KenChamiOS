package com.kenchami.kenchamios.sync

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.HttpUrl
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.io.IOException

private val JSON = "application/json".toMediaType()
private const val HTTP_UNAUTHORIZED = 401
private const val HTTP_FORBIDDEN = 403
private const val HTTP_NOT_FOUND = 404
private const val HTTP_UNPROCESSABLE = 422
private const val HTTP_TOO_MANY = 429
private const val MIN_TOKEN_LENGTH = 32
private const val MAX_TOKEN_LENGTH = 128

/** Why a call to the desktop did not work; the sync screen shows the message. */
sealed class SyncFailure(message: String, cause: Throwable? = null) : Exception(message, cause) {
    class Unreachable(cause: Throwable) : SyncFailure("The desktop cannot be reached on this network.", cause)

    class PinMismatch(cause: Throwable) : SyncFailure("The desktop certificate changed. Pair again.", cause)

    class AuthLost : SyncFailure("The desktop no longer knows this phone. Pair again.")

    class Throttled : SyncFailure("The desktop is blocking this phone for a minute after too many failures.")

    class WrongCode : SyncFailure("The pairing code is wrong.")

    class PairingClosed : SyncFailure("The desktop is not waiting for a new phone. Open the pairing window there.")

    class Refused(reason: String) : SyncFailure("The desktop refused the data: $reason")

    class BadAnswer(cause: Throwable? = null) : SyncFailure("The desktop sent an answer this app does not understand.", cause)
}

data class PairResult(val token: String, val desktopDeviceId: String)

/** The HTTPS calls of the sync protocol; [client] must be pinned to the desktop certificate. */
class SyncApi(private val client: OkHttpClient, host: String, port: Int) {
    private val base: HttpUrl = HttpUrl.Builder().scheme("https").host(host).port(port).build()

    suspend fun pair(code: String, deviceId: String, deviceName: String): PairResult {
        val body = JSONObject().put("code", code).put("device_id", deviceId).put("device_name", deviceName)
        val answer =
            call(Request.Builder().url(base.resolve("/pair") ?: throw SyncFailure.BadAnswer()).post(body.toString().toRequestBody(JSON)))
        return parse {
            val json = JSONObject(answer)
            val token = json.getString("token")
            val desktop = json.getString("server_device_id")
            if (token.length !in MIN_TOKEN_LENGTH..MAX_TOKEN_LENGTH || !isValidId(desktop)) throw JSONException("bad pairing answer")
            PairResult(token, desktop)
        }
    }

    suspend fun push(credential: String, changes: List<Change>): ApplyOutcome {
        val array = JSONArray()
        changes.forEach { array.put(JSONObject().put("table", it.table).put("row", it.row)) }
        val body = JSONObject().put("changes", array).toString().toRequestBody(JSON)
        val answer = call(authorized(credential, "/sync/push").post(body))
        return parse {
            val json = JSONObject(answer)
            ApplyOutcome(json.getInt("applied"), json.getInt("ignored"))
        }
    }

    suspend fun pull(credential: String, since: Long, limit: Int): ChangePage {
        val answer = call(authorized(credential, "/sync/pull?since=$since&limit=$limit").get())
        return parse { parsePull(JSONObject(answer)) }
    }

    private fun authorized(credential: String, path: String): Request.Builder =
        Request.Builder().url(base.resolve(path) ?: throw SyncFailure.BadAnswer()).header("Authorization", "Bearer $credential")

    private suspend fun call(request: Request.Builder): String = withContext(Dispatchers.IO) {
        try {
            client.newCall(request.build()).execute().use(::read)
        } catch (error: IOException) {
            throw if (hasPinMismatch(error)) SyncFailure.PinMismatch(error) else SyncFailure.Unreachable(error)
        }
    }

    private fun read(response: Response): String {
        val text = response.body.string()
        return when (response.code) {
            in 200..299 -> text
            HTTP_UNAUTHORIZED -> throw SyncFailure.AuthLost()
            HTTP_FORBIDDEN -> throw SyncFailure.WrongCode()
            HTTP_NOT_FOUND -> throw SyncFailure.PairingClosed()
            HTTP_TOO_MANY -> throw SyncFailure.Throttled()
            HTTP_UNPROCESSABLE -> throw SyncFailure.Refused(reasonOf(text))
            else -> throw SyncFailure.BadAnswer()
        }
    }

    private fun reasonOf(text: String): String = try {
        JSONObject(text).optString("error", "invalid data")
    } catch (_: JSONException) {
        "invalid data"
    }

    // a host with several addresses reports the failed handshake as a suppressed error of the last connect error
    private fun hasPinMismatch(error: Throwable): Boolean =
        error is PinMismatchException || error.cause?.let(::hasPinMismatch) == true || error.suppressed.any(::hasPinMismatch)

    private fun <T> parse(block: () -> T): T = try {
        block()
    } catch (error: JSONException) {
        throw SyncFailure.BadAnswer(error)
    }

    private fun parsePull(json: JSONObject): ChangePage {
        val array = json.getJSONArray("changes")
        val changes = List(array.length()) {
            val item = array.getJSONObject(it)
            Change(item.getString("table"), item.getJSONObject("row"))
        }
        return ChangePage(changes, json.getLong("next"), json.getBoolean("has_more"))
    }
}
