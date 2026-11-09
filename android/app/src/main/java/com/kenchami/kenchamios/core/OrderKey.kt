package com.kenchami.kenchamios.core

// fractional indexing: keys are base-62 strings that sort like fractions, so an
// item can always be placed between two neighbours without touching other rows
private const val DIGITS = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
private val BASE = DIGITS.length

object OrderKey {
    /**
     * Returns a key that sorts strictly between [before] and [after].
     * An empty [before] means the start of the list, a null [after] the end.
     */
    fun between(before: String, after: String?): String = midpoint(before, after)

    private fun midpoint(before: String, after: String?): String {
        if (after != null) {
            // missing digits in `before` count as zero
            val shared = after.indices.takeWhile { after[it] == (before.getOrNull(it) ?: DIGITS[0]) }.count()
            if (shared > 0) {
                return after.take(shared) + midpoint(before.drop(shared), after.drop(shared))
            }
        }

        val low = before.firstOrNull()?.let(::digitValue) ?: 0
        val high = after?.firstOrNull()?.let(::digitValue) ?: BASE
        if (high - low > 1) return DIGITS[(low + high) / 2].toString()
        if (after != null && after.length > 1) return after.take(1)
        return DIGITS[low] + midpoint(before.drop(1), null)
    }

    private fun digitValue(digit: Char): Int = DIGITS.indexOf(digit).coerceAtLeast(0)
}
