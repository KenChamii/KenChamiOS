package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.core.DEFAULT_AVAILABLE_MIN
import com.kenchami.kenchamios.core.startOfDay
import com.kenchami.kenchamios.data.LocalSettings
import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.TaskStatus
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.time.LocalDate
import java.time.ZoneOffset

@RunWith(RobolectricTestRunner::class)
class PlannerRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var tasks: TaskRepository
    private lateinit var planner: PlannerRepository
    private lateinit var projectId: String

    // the test clock is 2023-11-14 in UTC
    private val today = LocalDate.of(2023, 11, 14)
    private val endOfToday = startOfDay(today.plusDays(1), ZoneOffset.UTC)

    @Before
    fun setUp() {
        env = TestDatabase()
        tasks = TaskRepository(env.database, env.stamper, env.activity, env.clock)
        planner = PlannerRepository(env.database, env.stamper, env.activity, LocalSettings(env.database.settingDao(), env.stamper))
        projectId = runBlocking { ProjectRepository(env.database, env.stamper, env.activity).create("P") }
    }

    @After
    fun tearDown() = env.close()

    private suspend fun candidates(query: String = "") = planner.observeCandidates(today, endOfToday, query).first().map { it.title }

    @Test
    fun planningEstimatingAndUnplanningATask() = runBlocking<Unit> {
        val id = tasks.create(projectId, "Write")

        planner.setPlan(id, TaskPlan(today, null, 45))
        val planned = planner.observePlanned(today).first().single()
        assertEquals(listOf("2023-11-14", null, 45), listOf(planned.plannedOn, planned.scheduledAt, planned.durationMin))

        planner.setPlan(id, TaskPlan(null, null, 45))
        assertEquals(emptyList<Any>(), planner.observePlanned(today).first())
        assertEquals(45, checkNotNull(tasks.get(id)).durationMin)

        planner.setPlan(id, TaskPlan.NONE)
        val task = checkNotNull(tasks.get(id))
        assertEquals(listOf(null, null, null), listOf(task.plannedOn, task.scheduledAt, task.durationMin))
        assertEquals(
            listOf("task.created", "task.planned", "task.unplanned", "task.unplanned"),
            env.database.activityDao().kindsFor(id),
        )
    }

    @Test
    fun badPlansAndMissingTasksAreRefused() = runBlocking<Unit> {
        val id = tasks.create(projectId, "Write")

        for (bad in listOf(TaskPlan(null, 5_000, 30), TaskPlan(today, -1, 30), TaskPlan(today, null, 0), TaskPlan(today, null, 1441))) {
            assertThrows(IllegalArgumentException::class.java) { runBlocking { planner.setPlan(id, bad) } }
        }
        assertThrows(IllegalStateException::class.java) { runBlocking { planner.setPlan("missing", TaskPlan.NONE) } }
    }

    @Test
    fun theDayListsTimeBlocksFirstThenTheRest() = runBlocking<Unit> {
        for ((title, at) in listOf("Loose" to null, "Late" to 9_000L, "Early" to 1_000L)) {
            planner.setPlan(tasks.create(projectId, title), TaskPlan(today, at, 30))
        }
        planner.setPlan(tasks.create(projectId, "Tomorrow"), TaskPlan(today.plusDays(1), null, 30))

        assertEquals(listOf("Early", "Late", "Loose"), planner.observePlanned(today).first().map { it.title })
    }

    @Test
    fun candidatesAreDueInTheInboxInProgressOrLeftOver() = runBlocking<Unit> {
        tasks.create(projectId, "Due later", dueAt = endOfToday + 1)
        tasks.create(projectId, "Someday")
        tasks.create(projectId, "Overdue", dueAt = 1_000)
        tasks.create(Inbox.ID, "Inboxed")
        tasks.setStatus(tasks.create(projectId, "Started"), TaskStatus.Doing)
        tasks.setStatus(tasks.create(projectId, "Finished", dueAt = 1_000), TaskStatus.Done)
        planner.setPlan(tasks.create(projectId, "Yesterday's"), TaskPlan(today.minusDays(1), null, 30))
        planner.setPlan(tasks.create(projectId, "Already today", dueAt = 1_000), TaskPlan(today, null, 30))

        assertEquals(listOf("Overdue", "Inboxed", "Started", "Yesterday's"), candidates())
    }

    @Test
    fun aSearchFindsAnyOpenTaskWithItsWildcardsTakenLiterally() = runBlocking<Unit> {
        tasks.create(projectId, "Someday report")
        tasks.create(projectId, "50% done")
        tasks.create(projectId, "500 lines")

        assertEquals(listOf("Someday report"), candidates(" report "))
        assertEquals(listOf("50% done"), candidates("50%"))
    }

    @Test
    fun theNextOccurrenceOfARepeatingTaskStartsUnplanned() = runBlocking<Unit> {
        val id = tasks.create(projectId, "Water", Priority.Low, dueAt = 1_000, recurrence = "daily:1")
        planner.setPlan(id, TaskPlan(today, 5_000, 30))

        val next = checkNotNull(tasks.get(checkNotNull(tasks.setStatus(id, TaskStatus.Done))))

        assertEquals(listOf(null, null, null), listOf(next.plannedOn, next.scheduledAt, next.durationMin))
        assertEquals("2023-11-14", checkNotNull(tasks.get(id)).plannedOn)
    }

    @Test
    fun availableHoursDefaultToSixAndKeepAValidChange() = runBlocking<Unit> {
        assertEquals(DEFAULT_AVAILABLE_MIN, planner.observeAvailableMinutes().first())

        planner.setAvailableMinutes(450)
        assertEquals(450, planner.observeAvailableMinutes().first())

        for (bad in listOf(29, 1441)) {
            assertThrows(IllegalArgumentException::class.java) { runBlocking { planner.setAvailableMinutes(bad) } }
        }
        assertEquals(0, env.count("SELECT COUNT(*) FROM change_log WHERE table_name = 'settings'"))
    }
}
