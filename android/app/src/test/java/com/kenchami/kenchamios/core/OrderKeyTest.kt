package com.kenchami.kenchamios.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class OrderKeyTest {
    @Test
    fun firstKeyIsInTheMiddle() {
        assertEquals("V", OrderKey.between("", null))
    }

    @Test
    fun matchesTheDesktopKeys() {
        assertEquals("k", OrderKey.between("V", null))
        assertEquals("aV", OrderKey.between("a", "b"))
    }

    @Test
    fun appendingKeepsIncreasing() {
        val keys = mutableListOf(OrderKey.between("", null))
        repeat(200) { keys += OrderKey.between(keys.last(), null) }
        assertEquals(keys.sorted(), keys)
        assertEquals(keys.size, keys.toSet().size)
    }

    @Test
    fun prependingKeepsDecreasing() {
        val keys = mutableListOf(OrderKey.between("", null))
        repeat(200) { keys.add(0, OrderKey.between("", keys.first())) }
        assertEquals(keys.sorted(), keys)
        assertEquals(keys.size, keys.toSet().size)
    }

    @Test
    fun insertingInTheSameGapNeverCollides() {
        val low = OrderKey.between("", null)
        var high = OrderKey.between(low, null)
        repeat(200) {
            val mid = OrderKey.between(low, high)
            assertTrue("$low < $mid < $high", low < mid && mid < high)
            high = mid
        }
    }

    @Test
    fun handlesAdjacentDigits() {
        val mid = OrderKey.between("a", "b")
        assertTrue("a" < mid && mid < "b")
    }
}
